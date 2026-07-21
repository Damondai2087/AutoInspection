//! 前后端共享的数据结构

use serde::{Deserialize, Serialize};

/// 巡检请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionRequest {
    /// 目标系统: "windows" | "linux" | "macos"
    pub target_system: String,
    /// 巡检模式: "local" | "remote"
    pub mode: String,
    /// 勾选的巡检模块 key 列表
    pub modules: Vec<String>,
    /// 快速模式（跳过耗时检查）
    pub fast: bool,
    /// 远程连接配置（mode=remote 时使用）
    #[serde(default)]
    pub remote: RemoteConfig,
    /// 本次输出目录覆盖（可选）
    #[serde(default)]
    pub output_dir: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RemoteConfig {
    #[serde(default)]
    pub host: String,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub username: String,
    /// 认证方式: "password" | "key"
    #[serde(default)]
    pub auth: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub key_path: String,
}

/// 巡检结果（完成时回传前端）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionResult {
    pub success: bool,
    pub report_path: Option<String>,
    pub report_format: String,
    pub message: String,
    /// 退出状态: ok | warn | critical | failed
    #[serde(default)]
    pub status: String,
}

/// 实时日志行
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLine {
    pub level: String, // "info" | "error" | "warn"
    pub text: String,
}

/// 历史记录项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    pub id: String,
    pub timestamp: String,
    pub target_system: String,
    pub mode: String,
    pub target: String,
    pub report_path: String,
    /// ok | warn | critical | failed
    pub status: String,
    pub summary: String,
}

/// 软件设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub output_dir: String,
    /// "dark" | "light"
    #[serde(default)]
    pub theme: String,
    /// 自定义脚本目录（为空则用内置脚本）
    #[serde(default)]
    pub custom_script_dir: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            output_dir: String::new(),
            theme: "dark".to_string(),
            custom_script_dir: String::new(),
        }
    }
}

/// 操作系统信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    pub platform: String, // windows | linux | macos
    pub name: String,
}

/// 权限检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionInfo {
    pub is_elevated: bool,
    pub method: String,
    pub message: String,
}
