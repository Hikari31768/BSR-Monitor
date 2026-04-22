# 08-05 - Tauri 主配置 Review

## Review 范围

- [src-tauri/src/main.rs](src-tauri/src/main.rs)
- [src-tauri/src/lib.rs](src-tauri/src/lib.rs)
- `src-tauri/tauri.conf.json`（如存在）
- `src-tauri/Cargo.toml`（依赖与 feature 配置）
- `src-tauri/build.rs`（如存在）
- `src-tauri/capabilities/` 目录（Tauri 2 权限配置）
- `src-tauri/icons/` 目录

## Review 维度

### 1. Tauri v2 规范

- 使用 `tauri::Builder::default()` 模式是否符合 Tauri 2 最佳实践
- 插件注册：`tauri_plugin_shell` — 是否有未使用的插件
- `invoke_handler` 中注册的命令列表是否完整，有无遗漏或多余
- `setup` 中初始化数据库的模式是否正确
- `generate_context!()` 宏的使用

### 2. 安全性

- Tauri 2 使用 capabilities 系统管理权限 — 是否正确配置
- 是否有不必要的 `fs`、`shell` 等权限暴露
- `main.rs` 中 `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` 是否正确

### 3. 依赖与 Feature

- `Cargo.toml` 中 `rusqlite`、`reqwest`、`tokio`、`tauri` 等依赖的版本和 feature 是否合理
- `reqwest` 是否启用了 `json` feature
- `tokio` 是否启用了 `rt-multi-thread` 和 `macros`
- `rusqlite` 是否启用了必要的 feature（如 `bundled`）
- 是否有未使用的依赖

### 4. 配置

- `tauri.conf.json` 中的窗口大小、标题、URL 配置是否合理
- `withGlobalTauri` 是否开启（影响前端能否访问 `window.__TAURI__`）
- CSP（Content Security Policy）配置
- 开发/生产环境差异配置

### 5. 构建

- 打包配置（[.project/DevGuide.md](.project/DevGuide.md) 提到 `npm run tauri build`）
- CDP 调试端口在生产构建中是否被正确禁用
- 图标资源是否完整
