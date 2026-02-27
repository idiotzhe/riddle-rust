# 元宵猜灯谜管理系统

基于 Bun (Node.js) / Rust (Axum) 后端，Vue 3 管理后台，以及 Tauri 桌面端包装的跨平台应用。

## 技术栈

- **前端管理后台**: Vue 3 + Element Plus + Vite
- **后端 API**: Rust + Axum + SQLx (SQLite) / Bun + Hono (可选)
- **桌面端**: Tauri 2.0
- **数据库**: SQLite

## 开发与编译 (跨平台)

本项目已适配 Windows 和 macOS 系统。

### 环境准备

1. 安装 [Bun](https://bun.sh/)
2. 安装 [Rust](https://rustup.rs/)
3. 安装 Tauri 依赖 (详见 [Tauri 文档](https://tauri.app/v1/guides/getting-started/prerequisites))

### 编译打包

可以使用统一的跨平台编译脚本 `build.js`：

#### 1. 打包 Tauri 桌面应用 (Windows/macOS)

```bash
bun build.js
```
编译产物位于 `dist-desktop` 目录。

#### 2. 打包 Standalone Rust 后端 (不含 Tauri 壳)

```bash
bun build.js --standalone
```
编译产物位于 `dist` 目录。

### 老版本脚本 (仅限 Windows)

- `build_tauri_app.bat`: 打包 Tauri 应用
- `build_rust_app.bat`: 打包 Standalone 版本的 Rust 应用

## 项目结构

- `admin/`: Vue 3 管理后台源代码
- `src-tauri/`: Tauri 项目配置与集成后端源代码
- `backend-rust/`: 独立 Rust 后端源代码
- `template/`: 前端展示页面 (手机端/展示端)
- `app.js`: Bun 后端实现 (参考用)

## 注意事项

- **数据库文件**: 应用运行时会在系统的 AppData (Windows) 或 Application Support (macOS) 目录下创建 `lantern.db`。
- **静态资源**: 管理后台编译后会自动放入 `template/admin` 并通过后端服务。
