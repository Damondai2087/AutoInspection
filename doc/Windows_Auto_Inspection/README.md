# Windows 系统自动巡检工具

一键运行，自动采集 Windows 系统 27 项巡检数据，生成专业 HTML 报告，可直接交付给领导或客户。

## 报告效果

- 深灰色封面标题 + 元信息表格
- 十章节目录导航（点击跳转）
- 事件日志自动聚合分析（内置 22 种常见事件知识库）
- 自动风险评估 + 修复建议（含命令）

## 巡检项目（27 项）

| 章节 | 内容 |
|------|------|
| 一、目标主机基本信息 | 主机名、操作系统、架构、域、BIOS、许可证、运行时间 |
| 二、硬件资源状态 | CPU、内存、磁盘使用率、物理磁盘健康、GPU/驱动 |
| 三、网络配置与连接 | 适配器、IP 分配、网关/DNS、监听端口（含说明）、共享文件夹 |
| 四、安全配置审计 | 防火墙、RDP、BitLocker、Defender、密码策略、补丁、时间同步 |
| 五、用户与权限 | 本地用户、管理员组 |
| 六、进程与服务分析 | 进程数、内存 Top 10 |
| 七、启动项与计划任务 | 注册表启动项、非微软计划任务 |
| 八、已安装软件 | 软件名、版本、发布者、安装日期 |
| 九、事件日志分析 | 系统/应用日志聚合、严重程度、知识库说明 |
| 十、风险评估与建议 | 自动评级、问题清单、修复命令 |

## 使用方法

### 1. 环境要求

- **Windows 10 / 11 / Server 2016+**
- **Python 3.6+**（大部分 Windows 已预装，没有的话看下面安装步骤）

### 2. 检查 Python 是否已安装

按 `Win + R`，输入 `cmd`，回车，然后输入：

```
python --version
```

如果显示 `Python 3.x.x` 就可以直接用。如果提示"不是内部命令"，需要先安装 Python。

### 3. 安装 Python（如果没有）

1. 打开浏览器访问 https://www.python.org/downloads/
2. 点击 **Download Python 3.x.x**
3. 运行安装程序，**务必勾选底部的 "Add Python to PATH"**
4. 点击 Install Now，等待完成
5. 重新打开 cmd，输入 `python --version` 确认

### 4. 下载巡检脚本

**方式一：直接下载**

1. 打开 https://github.com/Aidan-996/Windows_Auto_Inspection
2. 点击绿色 **Code** 按钮 → **Download ZIP**
3. 解压到任意目录

**方式二：Git 克隆**

```
git clone https://github.com/Aidan-996/Windows_Auto_Inspection.git
```

### 5. 运行巡检

双击运行或在 cmd 中执行：

```
python win_inspection_html.py
```

等待约 30 秒，完成后会在**当前目录**生成报告文件：

```
System_Inspection_Report_20260407_112813.html
```

**双击这个 .html 文件即可在浏览器中查看报告。**

### 6. 以管理员身份运行（推荐）

部分巡检项（审计策略、BitLocker、Defender、系统还原点）需要管理员权限才能完整采集。

1. 在开始菜单搜索 **cmd**
2. 右键 → **以管理员身份运行**
3. `cd` 到脚本所在目录
4. 运行 `python win_inspection_html.py`

## 常见问题

**Q: 提示 "python 不是内部命令"**
> 安装 Python 时没有勾选 "Add to PATH"。重新运行安装程序，选 Modify，勾选 PATH 选项。或者直接用完整路径运行：`C:\Users\你的用户名\AppData\Local\Programs\Python\Python3xx\python.exe win_inspection_html.py`

**Q: 报告里部分数据显示"无法获取"**
> 以管理员身份运行 cmd 再试一次。

**Q: 报告中文乱码**
> 脚本已处理 GBK/UTF-8 编码兼容。如果仍有问题，确保用 Chrome / Edge 打开 .html 文件（不要用记事本）。

**Q: 可以在服务器上定时运行吗？**
> 可以。用 Windows 任务计划程序添加任务，操作设为 `python C:\路径\win_inspection_html.py`，触发器设为每天/每周。

## 报告输出示例

```
Windows 系统巡检报告
DESKTOP-J44NOEQ / Windows 11 (v10.0.26200)

巡检日期    2026-04-07
日志范围    2026-03-30 ~ 2026-04-06
巡检人员    DESKTOP-J44NOEQ\Admin
综合风险    高

一、目标主机基本信息
  计算机名      DESKTOP-J44NOEQ
  操作系统      Windows 11 (v10.0.26200)
  许可证        已授权 (KMS)
  运行时间      2天 10时 50分
  ...

十、风险评估与建议
  [高] NTFS 文件系统损坏 → chkdsk /f
  [高] 非正常关机 3 次 → 排查电源/蓝屏
  [中] 密码最短长度为 0 → net accounts /minpwlen:8
```

## Contributing | 参与贡献

欢迎提交 Issue 和 Pull Request！详见 [CONTRIBUTING.md](CONTRIBUTING.md)

```
Fork → Clone → Branch → Commit → Push → Pull Request
```

## 版权声明

Copyright (c) 2026 [Aidan-996](https://github.com/Aidan-996)

本项目采用 MIT License 开源许可证。

**请尊重原创，转载或二次开发请保留原作者信息。**

- 作者：[Aidan-996](https://github.com/Aidan-996)
- 仓库：[Windows_Auto_Inspection](https://github.com/Aidan-996/Windows_Auto_Inspection)
- 如有问题或建议，欢迎提交 [Issue](https://github.com/Aidan-996/Windows_Auto_Inspection/issues)
