# AutoInspection 项目记忆

## 项目概况
- 跨平台主机自动巡检桌面客户端，Rust + Tauri v2 + Vue3 + Element Plus
- 整合 Linux_Auto_Inspection（Bash）和 Windows_Auto_Inspection（Python）开源巡检脚本
- 目标：可视化界面替代命令行操作，支持本机/远程巡检，自动生成报告
- 后续兼容 macOS 巡检

## 技术栈
- 前端：Vue3 + Vite + Element Plus，深色主题蓝色主色调
- 后端：Rust，Tauri v2 IPC 通信
- 巡检脚本内置打包，运行时释放到工作目录

## 项目状态
- 当前为 Tauri 模板初始状态（2026-07-21）
- README 已编写完成
- 开发计划见 README 中的 checklist

## 关键文件路径
- README.md：项目说明文档
- doc/Linux_Auto_Inspection/：Linux 巡检脚本及文档
- doc/Windows_Auto_Inspection/：Windows 巡检脚本及文档
- src-tauri/src/lib.rs：Rust 后端入口（目前为模板代码）
- src/App.vue：前端入口（目前为模板代码）

## 依赖的开源项目
- https://github.com/Aidan-996/Linux_Auto_Inspection
- https://github.com/Aidan-996/Windows_Auto_Inspection
