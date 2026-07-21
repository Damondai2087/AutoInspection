//! 巡检核心管理器
//! 负责释放内置脚本、创建并管控子进程、实时推送日志、读取/导出报告、
//! 管理历史记录与设置、检测系统类型与权限、选择目录等。

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

use crate::convert;
use crate::models::*;

/// 全局可克隆的巡检管理器
#[derive(Clone)]
pub struct InspectionManager {
    #[allow(dead_code)]
    pub app_data_dir: PathBuf,
    pub scripts_dir: PathBuf,
    pub reports_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl InspectionManager {
    /// 基于 AppHandle 初始化目录结构
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let base = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("无法获取应用数据目录: {}", e))?;
        let scripts_dir = base.join("scripts");
        let reports_dir = base.join("reports");
        let data_dir = base.join("data");
        for d in [&base, &scripts_dir, &reports_dir, &data_dir] {
            fs::create_dir_all(d).map_err(|e| e.to_string())?;
        }
        Ok(Self {
            app_data_dir: base,
            scripts_dir,
            reports_dir,
            data_dir,
        })
    }

    /// 将内置巡检脚本释放到可写工作目录（仅在缺失或版本不一致时复制）
    pub fn release_scripts(&self, app: &AppHandle) -> Result<(), String> {
        let src_dirs = script_source_dirs(app);
        let files = [
            ("linux_inspect.sh", "linux_inspect.sh"),
            ("win_inspection_html.py", "win_inspection_html.py"),
        ];
        for (src, dst) in files {
            let mut found: Option<PathBuf> = None;
            for dir in &src_dirs {
                let candidate = dir.join(src);
                if candidate.exists() {
                    found = Some(candidate);
                    break;
                }
            }
            let s = match found {
                Some(p) => p,
                None => continue,
            };
            let d = self.scripts_dir.join(dst);
            let src_len = fs::metadata(&s).map(|m| m.len()).unwrap_or(0);
            let dst_len = fs::metadata(&d).map(|m| m.len()).unwrap_or(0);
            if !d.exists() || src_len != dst_len {
                fs::copy(&s, &d).map_err(|e| format!("释放脚本失败 {}: {}", dst, e))?;
            }
        }
        Ok(())
    }

    /// 解析实际使用的脚本路径（自定义目录优先，否则用内置释放版本）
    pub fn resolve_script(&self, target: &str, settings: &Settings) -> PathBuf {
        let name = if target == "windows" {
            "win_inspection_html.py"
        } else {
            "linux_inspect.sh"
        };
        if !settings.custom_script_dir.is_empty() {
            let p = Path::new(&settings.custom_script_dir).join(name);
            if p.exists() {
                return p;
            }
        }
        self.scripts_dir.join(name)
    }

    /// 默认输出目录
    pub fn default_output_dir(&self, settings: &Settings) -> PathBuf {
        if !settings.output_dir.is_empty() {
            PathBuf::from(&settings.output_dir)
        } else {
            self.reports_dir.clone()
        }
    }

    /// 校验请求合法性（本机巡检必须匹配宿主系统）
    fn validate(&self, req: &InspectionRequest) -> Result<(), String> {
        if req.target_system == "macos" {
            return Err("macOS 巡检脚本即将支持，敬请期待".into());
        }
        if req.mode == "local" {
            let host = self.os_info().platform;
            if req.target_system != host {
                return Err(format!(
                    "本机巡检需在 {} 系统上运行，当前系统为 {}。请选择与本机匹配的目标系统，或使用「远程巡检」。",
                    req.target_system, host
                ));
            }
        }
        Ok(())
    }

    /// 启动巡检：释放脚本、准备目录、派生工作线程执行
    pub fn start(
        &self,
        app: AppHandle,
        running: Arc<Mutex<Option<Child>>>,
        req: InspectionRequest,
    ) -> Result<String, String> {
        self.release_scripts(&app)?;
        let settings = self.get_settings();
        let output_dir = if let Some(d) = &req.output_dir {
            if !d.is_empty() {
                PathBuf::from(d)
            } else {
                self.default_output_dir(&settings)
            }
        } else {
            self.default_output_dir(&settings)
        };
        fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;
        self.validate(&req)?;

        let mgr = self.clone();
        let appc = app.clone();
        let reqc = req.clone();
        std::thread::spawn(move || {
            mgr.run_worker(appc, running, reqc, output_dir);
        });
        Ok("started".into())
    }

    /// 工作线程：根据模式分发执行，完成后推送结果事件并记录历史
    fn run_worker(
        &self,
        app: AppHandle,
        running: Arc<Mutex<Option<Child>>>,
        req: InspectionRequest,
        output_dir: PathBuf,
    ) {
        let target_name = match req.target_system.as_str() {
            "windows" => "Windows",
            "linux" => "Linux",
            "macos" => "macOS",
            _ => "未知",
        };
        let mode_name = if req.mode == "remote" { "远程" } else { "本机" };
        let _ = app.emit(
            "inspection-log",
            LogLine {
                level: "info".to_string(),
                text: format!("▶ 开始{}巡检 · 目标系统: {}", mode_name, target_name),
            },
        );

        let outcome: Result<(PathBuf, String), String> = match (
            req.mode.as_str(),
            req.target_system.as_str(),
        ) {
            ("local", "linux") => self.run_local_linux(&app, running.clone(), &req, &output_dir),
            ("local", "windows") => {
                self.run_local_windows(&app, running.clone(), &req, &output_dir)
            }
            ("local", "macos") => Err("macOS 巡检脚本即将支持，敬请期待".into()),
            ("remote", "linux") => self.run_remote_linux(&app, running.clone(), &req, &output_dir),
            ("remote", "windows") => Err("Windows 远程巡检暂不支持，敬请期待".into()),
            ("remote", "macos") => Err("macOS 远程巡检暂不支持，敬请期待".into()),
            _ => Err("未知的巡检模式或目标系统".into()),
        };

        let result = match outcome {
            Ok((report, status)) => {
                let msg = format!("✅ 巡检完成 · 报告: {}", report.display());
                let _ = app.emit(
                    "inspection-log",
                    LogLine {
                        level: "info".to_string(),
                        text: msg.clone(),
                    },
                );
                InspectionResult {
                    success: true,
                    report_path: Some(report.to_string_lossy().to_string()),
                    report_format: "html".into(),
                    message: msg,
                    status,
                }
            }
            Err(e) => {
                let _ = app.emit(
                    "inspection-log",
                    LogLine {
                        level: "error".to_string(),
                        text: format!("❌ 巡检失败: {}", e),
                    },
                );
                InspectionResult {
                    success: false,
                    report_path: None,
                    report_format: "html".into(),
                    message: e,
                    status: "failed".into(),
                }
            }
        };

        let _ = self.add_history_from_result(&req, &result);
        let _ = app.emit("inspection-done", &result);
    }

    /// 本机 Linux 巡检
    fn run_local_linux(
        &self,
        app: &AppHandle,
        running: Arc<Mutex<Option<Child>>>,
        req: &InspectionRequest,
        output_dir: &Path,
    ) -> Result<(PathBuf, String), String> {
        let settings = self.get_settings();
        let script = self.resolve_script("linux", &settings);
        if !script.exists() {
            return Err(format!("未找到 Linux 巡检脚本: {}", script.display()));
        }
        let mut cmd = Command::new("bash");
        cmd.arg(&script);
        if req.fast {
            cmd.arg("--fast");
        }
        apply_linux_module_flags(&mut cmd, &req.modules);
        cmd.env("INSPECT_REPORT_DIR", output_dir);
        cmd.current_dir(output_dir);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let code = self.run_command(app, running, &mut cmd, &[], false)?;
        let report = find_latest_report(output_dir, "html");
        Ok((report, status_from_exit("linux", code)))
    }

    /// 本机 Windows 巡检
    fn run_local_windows(
        &self,
        app: &AppHandle,
        running: Arc<Mutex<Option<Child>>>,
        req: &InspectionRequest,
        output_dir: &Path,
    ) -> Result<(PathBuf, String), String> {
        let settings = self.get_settings();
        let script = self.resolve_script("windows", &settings);
        if !script.exists() {
            return Err(format!("未找到 Windows 巡检脚本: {}", script.display()));
        }
        // 优先使用随应用内置的便携 Python，用户零安装；缺失时回退系统 Python
        let (py, py_source) = match bundled_python(app) {
            Some(p) => (clean_path_for_display(&p), "内置便携版"),
            None => match detect_python() {
                Some(p) => (p, "系统"),
                None => {
                    return Err(
                        "未检测到可用的 Python 解释器。应用应内置便携 Python，请确认安装完整；也可自行安装 Python 3.6+ 后重试"
                            .into(),
                    )
                }
            },
        };
        Self::emit_log(app, "info", &format!("Python 解释器（{}）: {}", py_source, py));
        Self::emit_log(app, "info", &format!("巡检脚本: {}", script.display()));
        Self::emit_log(app, "info", &format!("工作目录: {}", output_dir.display()));
        let mut cmd = Command::new(&py);
        // -u：关闭标准输出缓冲，让日志实时回显（否则会在进程退出时一次性刷出）
        cmd.arg("-u");
        cmd.arg(&script);
        if req.fast {
            cmd.arg("--fast");
        }
        // 强制 UTF-8 标准输出，避免中文 Windows（GBK/cp936 控制台）导致日志乱码。
        // 脚本写报告文件时已显式使用 utf-8，故不受影响。
        cmd.env("PYTHONIOENCODING", "utf-8");
        cmd.env("PYTHONUTF8", "1");
        cmd.current_dir(output_dir);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let code = self.run_command(app, running, &mut cmd, &[], false)?;
        Self::emit_log(
            app,
            "info",
            &format!("Python 进程退出，退出码: {:?}", code),
        );
        let report = find_latest_report(output_dir, "html");
        if report.to_string_lossy().contains("未找到") {
            Self::emit_log(
                app,
                "error",
                &format!("未在 {} 找到报告文件，请检查脚本是否完整执行", output_dir.display()),
            );
        } else {
            Self::emit_log(
                app,
                "info",
                &format!("报告已生成: {}", report.display()),
            );
        }
        Ok((report, status_from_exit("windows", code)))
    }

    /// 远程 Linux 巡检（ssh + scp，密钥或 sshpass 密码认证）
    fn run_remote_linux(
        &self,
        app: &AppHandle,
        running: Arc<Mutex<Option<Child>>>,
        req: &InspectionRequest,
        output_dir: &Path,
    ) -> Result<(PathBuf, String), String> {
        let rc = &req.remote;
        if rc.host.trim().is_empty() || rc.username.trim().is_empty() {
            return Err("远程巡检请填写主机 IP 和登录账号".into());
        }
        let port = if rc.port == 0 { 22 } else { rc.port };
        let settings = self.get_settings();
        let script = self.resolve_script("linux", &settings);
        let script_content =
            fs::read(&script).map_err(|e| format!("读取脚本失败: {}", e))?;
        let use_pass = rc.auth == "password";
        let key = rc.key_path.clone();

        let _ = app.emit(
            "inspection-log",
            LogLine {
                level: "info".to_string(),
                text: format!("上传巡检脚本到 {}@{}:{} ...", rc.username, rc.host, port),
            },
        );
        let mut up = build_scp(
            use_pass,
            &rc.password,
            &rc.username,
            rc.host.as_str(),
            port,
            &key,
            &script.to_string_lossy(),
            "/tmp/autoinspect_inspect.sh",
            true,
        );
        self.run_command(app, running.clone(), &mut up, &[], true)?;

        let _ = app.emit(
            "inspection-log",
            LogLine {
                level: "info".to_string(),
                text: "在远程主机执行巡检...".into(),
            },
        );
        let mut ssh = build_ssh_inspect(
            use_pass,
            &rc.password,
            &rc.username,
            rc.host.as_str(),
            port,
            &key,
        );
        self.run_command(app, running.clone(), &mut ssh, &script_content, true)?;

        let _ = app.emit(
            "inspection-log",
            LogLine {
                level: "info".to_string(),
                text: "拉取巡检报告...".into(),
            },
        );
        let mut down = build_scp(
            use_pass,
            &rc.password,
            &rc.username,
            rc.host.as_str(),
            port,
            &key,
            "/tmp/autoinspect_report/*.html",
            &output_dir.to_string_lossy(),
            false,
        );
        self.run_command(app, running.clone(), &mut down, &[], true)?;

        let report = find_latest_report(output_dir, "html");
        Ok((report, "ok".into()))
    }

    /// 通用命令执行：写入 stdin、流式推送 stdout/stderr、等待结束
    /// 返回退出码；fail_on_nonzero 为 true 时非零退出视为错误
    fn run_command(
        &self,
        app: &AppHandle,
        running: Arc<Mutex<Option<Child>>>,
        cmd: &mut Command,
        stdin_data: &[u8],
        fail_on_nonzero: bool,
    ) -> Result<Option<i32>, String> {
        let mut child = cmd.spawn().map_err(|e| {
            Self::emit_log(
                app,
                "error",
                &format!("启动进程失败: {}（请确认相关命令可用）", e),
            );
            format!("启动进程失败: {}（请确认相关命令可用）", e)
        })?;
        // 打印实际执行的命令，便于排查
        let cmd_str = format!(
            "{} {}",
            clean_path_for_display(Path::new(cmd.get_program())),
            cmd.get_args()
                .map(|a| a.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(" ")
        );
        Self::emit_log(app, "info", &format!("执行命令: {}", cmd_str));
        if let Some(mut stdin) = child.stdin.take() {
            let data = stdin_data.to_vec();
            std::thread::spawn(move || {
                let _ = stdin.write_all(&data);
                let _ = stdin.flush();
            });
        }
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        *running.lock().unwrap() = Some(child);
        if let Some(out) = stdout {
            Self::spawn_reader(app.clone(), out, "info");
        }
        if let Some(err) = stderr {
            Self::spawn_reader(app.clone(), err, "error");
        }
        let code = {
            let mut g = running.lock().unwrap();
            if let Some(mut c) = g.take() {
                drop(g);
                c.wait().ok().and_then(|s| s.code())
            } else {
                None
            }
        };
        if fail_on_nonzero && code != Some(0) {
            return Err(format!("远程命令返回非零状态，退出码: {:?}", code));
        }
        Ok(code)
    }

    /// 终止正在运行的巡检进程（含子进程树）
    pub fn stop(&self, running: Arc<Mutex<Option<Child>>>) -> Result<String, String> {
        let mut g = running.lock().unwrap();
        if let Some(child) = g.as_mut() {
            kill_tree(child);
            Ok("已发送终止信号，进程正在退出".into())
        } else {
            Ok("当前没有运行中的巡检任务".into())
        }
    }

    /// 读取报告文件内容
    pub fn read_report(&self, path: &str) -> Result<String, String> {
        fs::read_to_string(path).map_err(|e| format!("读取报告失败: {}", e))
    }

    /// 导出报告为指定格式到目标目录
    pub fn export_report(
        &self,
        src: &str,
        format: &str,
        dest_dir: &str,
    ) -> Result<String, String> {
        let src_path = Path::new(src);
        if !src_path.exists() {
            return Err(format!("源报告不存在: {}", src));
        }
        let stem = src_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("report");
        let ext = match format {
            "md" => "md",
            "txt" => "txt",
            _ => "html",
        };
        let mut out_path = PathBuf::from(dest_dir);
        if out_path.as_os_str().is_empty() {
            out_path = self.reports_dir.clone();
        }
        fs::create_dir_all(&out_path).map_err(|e| e.to_string())?;
        let out_file = out_path.join(format!("{}.{}", stem, ext));
        match format {
            "txt" => {
                let html = fs::read_to_string(src_path).map_err(|e| e.to_string())?;
                let txt = convert::html_to_text(&html);
                fs::write(&out_file, txt).map_err(|e| e.to_string())?;
            }
            "md" => {
                let html = fs::read_to_string(src_path).map_err(|e| e.to_string())?;
                let md = convert::html_to_markdown(&html);
                fs::write(&out_file, md).map_err(|e| e.to_string())?;
            }
            _ => {
                fs::copy(src_path, &out_file).map_err(|e| e.to_string())?;
            }
        }
        Ok(out_file.to_string_lossy().to_string())
    }

    // ------------------------- 历史记录 -------------------------

    fn history_path(&self) -> PathBuf {
        self.data_dir.join("history.json")
    }

    pub fn delete_history_item(&self, id: &str) -> Result<(), String> {
        let mut v = self.get_history().unwrap_or_default();
        let before = v.len();
        v.retain(|item| item.id != id);
        if v.len() == before {
            return Err("未找到对应的历史记录".into());
        }
        let s = serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?;
        fs::write(self.history_path(), s).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_history(&self) -> Result<Vec<HistoryItem>, String> {
        let p = self.history_path();
        if !p.exists() {
            return Ok(vec![]);
        }
        let s = fs::read_to_string(&p).map_err(|e| e.to_string())?;
        let v: Vec<HistoryItem> = serde_json::from_str(&s).unwrap_or_default();
        Ok(v)
    }

    fn add_history_from_result(
        &self,
        req: &InspectionRequest,
        result: &InspectionResult,
    ) -> Result<(), String> {
        let mut v = self.get_history().unwrap_or_default();
        let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let target = if req.mode == "remote" {
            format!("{}@{}", req.remote.username, req.remote.host)
        } else {
            "本机".into()
        };
        let item = HistoryItem {
            id: format!("{}-{}", ts, req.target_system),
            timestamp: ts,
            target_system: req.target_system.clone(),
            mode: req.mode.clone(),
            target,
            report_path: result.report_path.clone().unwrap_or_default(),
            status: result.status.clone(),
            summary: if result.success {
                "巡检完成".into()
            } else {
                result.message.clone()
            },
        };
        v.push(item);
        if v.len() > 200 {
            v.drain(0..v.len() - 200);
        }
        let s = serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?;
        fs::write(self.history_path(), s).map_err(|e| e.to_string())?;
        Ok(())
    }

    // ------------------------- 设置 -------------------------

    fn settings_path(&self) -> PathBuf {
        self.data_dir.join("settings.json")
    }

    pub fn get_settings(&self) -> Settings {
        let p = self.settings_path();
        if let Ok(s) = fs::read_to_string(&p) {
            serde_json::from_str(&s).unwrap_or_else(|_| default_settings(&self.reports_dir))
        } else {
            default_settings(&self.reports_dir)
        }
    }

    pub fn save_settings(&self, s: &Settings) -> Result<(), String> {
        let p = self.settings_path();
        let s = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
        fs::write(&p, s).map_err(|e| e.to_string())?;
        Ok(())
    }

    // ------------------------- 系统与权限 -------------------------

    pub fn os_info(&self) -> OsInfo {
        let platform = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            "linux"
        };
        let name = match platform {
            "windows" => "Windows",
            "macos" => "macOS",
            _ => "Linux",
        }
        .to_string();
        OsInfo {
            platform: platform.to_string(),
            name,
        }
    }

    pub fn check_permission(&self) -> PermissionInfo {
        #[cfg(windows)]
        {
            let out = Command::new("powershell").args([
                "-NoProfile",
                "-Command",
                "([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)",
            ]).output();
            if let Ok(o) = out {
                let s = String::from_utf8_lossy(&o.stdout).to_lowercase();
                let elevated = s.contains("true");
                return PermissionInfo {
                    is_elevated: elevated,
                    method: "Windows Token".into(),
                    message: if elevated {
                        "已以管理员身份运行".into()
                    } else {
                        "建议以管理员身份运行，否则部分检查项无法采集".into()
                    },
                };
            }
            PermissionInfo {
                is_elevated: false,
                method: "unknown".into(),
                message: "无法检测权限状态".into(),
            }
        }
        #[cfg(not(windows))]
        {
            if let Ok(o) = Command::new("id").arg("-u").output() {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if s == "0" {
                    return PermissionInfo {
                        is_elevated: true,
                        method: "root".into(),
                        message: "已以 root 身份运行".into(),
                    };
                }
            }
            if let Ok(o) = Command::new("sudo").args(["-n", "true"]).output() {
                if o.status.success() {
                    return PermissionInfo {
                        is_elevated: true,
                        method: "sudo".into(),
                        message: "当前用户拥有免密 sudo 权限".into(),
                    };
                }
            }
            PermissionInfo {
                is_elevated: false,
                method: "user".into(),
                message: "建议以 root 或 sudo 运行，否则部分检查项无法采集".into(),
            }
        }
    }

    /// 原生文件夹选择对话框
    pub fn pick_dir(&self, app: &AppHandle) -> Option<String> {
        let picked = app.dialog().file().blocking_pick_folder()?;
        Some(picked.to_string())
    }

    /// 用系统默认程序打开文件或目录
    pub fn open_path(&self, app: &AppHandle, path: &str) -> Result<(), String> {
        app.opener()
            .open_path(path, None::<&str>)
            .map_err(|e| format!("打开失败: {}", e))
    }

    // ------------------------- 内部工具 -------------------------

    /// 统一推送一条日志
    fn emit_log(app: &AppHandle, level: &str, text: &str) {
        let _ = app.emit(
            "inspection-log",
            LogLine {
                level: level.to_string(),
                text: text.to_string(),
            },
        );
    }

    /// 把一行原始字节解码为字符串：优先 UTF-8，失败时退化为 lossy，
    /// 避免中文 Windows（GBK 等本地编码）导致整行被丢弃、日志面板空白。
    fn decode_line(buf: &[u8]) -> String {
        if let Ok(s) = std::str::from_utf8(buf) {
            s.to_string()
        } else {
            String::from_utf8_lossy(buf).to_string()
        }
    }

    /// 派生线程按字节读取管道并实时推送日志（不依赖 UTF-8 行读取）
    fn spawn_reader<R: Read + Send + 'static>(app: AppHandle, pipe: R, level: &'static str) {
        std::thread::spawn(move || {
            let mut reader = pipe.bytes();
            let mut buf: Vec<u8> = Vec::with_capacity(256);
            while let Some(Ok(b)) = reader.next() {
                if b == b'\n' {
                    let line = Self::decode_line(&buf);
                    if !line.is_empty() {
                        Self::emit_log(&app, level, &line);
                    }
                    buf.clear();
                } else if b != b'\r' {
                    buf.push(b);
                }
            }
            let line = Self::decode_line(&buf);
            if !line.is_empty() {
                Self::emit_log(&app, level, &line);
            }
        });
    }
}

/// 清理 Windows 路径显示：去除 \\?\ 长路径前缀、规范化分隔符
fn clean_path_for_display(p: &Path) -> String {
    let s = p.to_string_lossy();
    let s = s.strip_prefix("\\\\?\\").unwrap_or(&s);
    s.replace('\\', "/")
}

/// 默认设置
fn default_settings(reports_dir: &Path) -> Settings {
    Settings {
        output_dir: reports_dir.to_string_lossy().to_string(),
        theme: "dark".into(),
        custom_script_dir: String::new(),
    }
}

/// 将前端勾选的可选模块映射为 Linux 脚本的 skip 参数
fn apply_linux_module_flags(cmd: &mut Command, modules: &[String]) {
    if !modules.iter().any(|m| m == "largefile") {
        cmd.arg("--skip-large-file-scan");
    }
    if !modules.iter().any(|m| m == "update") {
        cmd.arg("--skip-update-check");
    }
    if !modules.iter().any(|m| m == "ssl") {
        cmd.arg("--skip-ssl-check");
    }
}

/// 根据退出码推导巡检状态
fn status_from_exit(target: &str, code: Option<i32>) -> String {
    match target {
        "linux" => match code {
            Some(0) => "ok".to_string(),
            Some(1) => "warn".to_string(),
            Some(2) => "critical".to_string(),
            _ => "failed".to_string(),
        },
        _ => match code {
            Some(0) => "ok".to_string(),
            _ => "failed".to_string(),
        },
    }
}

/// 查找目录下最新修改的指定扩展名文件
fn find_latest_report(dir: &Path, ext: &str) -> PathBuf {
    if let Ok(entries) = fs::read_dir(dir) {
        let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some(ext) {
                if let Ok(meta) = e.metadata() {
                    if let Ok(m) = meta.modified() {
                        if best.as_ref().map(|(t, _)| m > *t).unwrap_or(true) {
                            best = Some((m, p));
                        }
                    }
                }
            }
        }
        if let Some((_, p)) = best {
            return p;
        }
    }
    dir.join(format!("未找到.{} 报告", ext))
}

/// 计算内置脚本可能的源目录（兼容 dev 与 release 两种环境）
fn script_source_dirs(app: &AppHandle) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(res) = app.path().resource_dir() {
        dirs.push(res.join("resources").join("scripts"));
    }
    // dev 回退：可执行文件通常位于 target/debug 或 target/release 下，
    // 资源脚本位于 src-tauri/resources/scripts
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            if let Some(p) = exe_dir.parent().and_then(|p| p.parent()) {
                dirs.push(p.join("resources").join("scripts"));
            }
        }
    }
    dirs
}

/// 查找随应用打包的便携 Python（resources/python/python.exe）
/// 兼容 release（resource_dir）与 dev（可执行文件回溯到 src-tauri）两种布局
#[cfg(windows)]
fn bundled_python(app: &AppHandle) -> Option<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Ok(res) = app.path().resource_dir() {
        dirs.push(res.join("resources").join("python").join("python.exe"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            if let Some(p) = exe_dir.parent().and_then(|p| p.parent()) {
                dirs.push(p.join("resources").join("python").join("python.exe"));
            }
        }
    }
    dirs.into_iter().find(|p| p.exists())
}

/// 非 Windows 平台无需内置 Python
#[cfg(not(windows))]
fn bundled_python(_app: &AppHandle) -> Option<PathBuf> {
    None
}

/// 检测可用的 Python 解释器
fn detect_python() -> Option<String> {
    for cand in ["python", "python3", "py"] {
        if Command::new(cand)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Some(cand.to_string());
        }
    }
    None
}

/// 终止进程树（Windows 用 taskkill，Unix 用 pkill 子进程）
fn kill_tree(child: &mut Child) {
    let pid = child.id();
    let _ = child.kill();
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .output();
    }
    #[cfg(not(windows))]
    {
        let _ = Command::new("pkill")
            .args(["-TERM", "-P", &pid.to_string()])
            .output();
    }
}

/// 构造 scp 命令（上传/下载）
fn build_scp(
    use_pass: bool,
    password: &str,
    user: &str,
    host: &str,
    port: u16,
    key: &str,
    src: &str,
    dst: &str,
    upload: bool,
) -> Command {
    let remote = format!("{}@{}:{}", user, host, if upload { dst } else { src });
    let local = if upload { src } else { dst };
    let mut cmd = if use_pass {
        let mut c = Command::new("sshpass");
        c.args(["-p", password, "scp"]);
        c
    } else {
        Command::new("scp")
    };
    cmd.args([
        "-P",
        &port.to_string(),
        "-o",
        "StrictHostKeyChecking=no",
        "-o",
        "UserKnownHostsFile=/dev/null",
    ]);
    if !use_pass && !key.is_empty() {
        cmd.args(["-i", key]);
    }
    if upload {
        cmd.arg(local).arg(remote);
    } else {
        cmd.arg(remote).arg(local);
    }
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd
}

/// 构造 ssh 执行巡检命令（脚本通过 stdin 传入）
fn build_ssh_inspect(
    use_pass: bool,
    password: &str,
    user: &str,
    host: &str,
    port: u16,
    key: &str,
) -> Command {
    let target = format!("{}@{}", user, host);
    let remote_cmd = "INSPECT_REPORT_DIR=/tmp/autoinspect_report bash -s -- --fast";
    let mut cmd = if use_pass {
        let mut c = Command::new("sshpass");
        c.args(["-p", password, "ssh"]);
        c
    } else {
        Command::new("ssh")
    };
    cmd.args([
        "-p",
        &port.to_string(),
        "-o",
        "StrictHostKeyChecking=no",
        "-o",
        "UserKnownHostsFile=/dev/null",
    ]);
    if !use_pass && !key.is_empty() {
        cmd.args(["-i", key]);
    }
    cmd.arg(&target).arg(remote_cmd);
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd
}
