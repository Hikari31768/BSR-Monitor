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

---

## Review 结果

### 1. Tauri v2 规范

- [x] `tauri::Builder::default()` 模式符合 Tauri v2 最佳实践
- [x] `generate_context!()` 宏使用正确
- [x] `setup` 中初始化数据库的模式正确（使用 `app.path().app_data_dir()` 获取路径，通过 `app.manage()` 存储连接）
- [x] `invoke_handler` 中注册的命令列表与 `commands/` 目录下的模块一致，无遗漏
- [ ] `tauri_plugin_shell` 插件被注册且 `shell:allow-open` 权限被赋予，但在 Rust commands 和前端代码中均未发现任何使用。如果没有计划使用，建议移除该插件和对应权限
- [x] `lib.rs` 中 `#[cfg_attr(mobile, tauri::mobile_entry_point)]` 移动端入口点配置正确

### 2. 安全性

- [x] 使用了 Tauri v2 capabilities 系统管理权限
- [x] capabilities 权限最小化做得较好：仅配置了 `core:default` 和 `shell:allow-open`
- [x] `main.rs` 中 `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` 正确，生产环境不会显示控制台窗口
- [ ] `capabilities/default.json` 中 `windows: ["main"]` 引用了窗口标签 `"main"`，但 `tauri.conf.json` 中窗口配置没有显式设置 `label` 字段。虽然 Tauri v2 默认第一个窗口的 label 为 `"main"`，但建议显式设置以避免隐式依赖
- [x] `core:default` 包含 `allow-internal-toggle-devtools`，这是默认行为，devtools 仅在 debug 构建中可用
- [ ] 未配置远程 CDP 调试端口（`remoteDevtoolsUrl`），这是正确的——生产环境不应暴露调试端口
- [ ] `tauri.conf.json` 中未显式配置 `withGlobalTauri`（默认 false）。如果前端不需要通过 `window.__TAURI__` 访问 Tauri API，则无需开启；如果需要则必须开启。当前前端代码中未发现 `__TAURI__` 相关引用

### 3. 依赖与 Feature

- [x] `rusqlite` 启用 `bundled` feature，无需系统安装 SQLite，正确
- [x] `reqwest` 启用 `json` feature，支持 JSON 序列化/反序列化，正确
- [x] `tokio` 启用 `rt-multi-thread` 和 `macros` feature，支持异步运行时和 `#[tokio::main]`，正确
- [x] `thiserror` 在 `src/api/error.rs` 中使用
- [x] `uuid` 在 `commands/playlist.rs` 和 `commands/player.rs` 中使用（v4 + serde feature）
- [x] 所有依赖均有使用，未发现未使用的依赖
- [ ] `tauri-build`、`tauri`、`tauri-plugin-shell` 均使用大版本号 `"2"`，未锁定小版本。建议在生产环境中锁定到具体小版本（如 `"2.0"` 或 `"2.1"`）以避免意外的 breaking changes

### 4. 配置

- [x] 窗口大小 1200x800 合理
- [x] `resizable: true`、`fullscreen: false` 合理
- [ ] **CSP 未配置**（`"csp": null`）。建议至少设置一个基础的 CSP，如 `"default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"`，以防止 XSS 等攻击
- [x] `devUrl: "http://localhost:1420"` 端口配置合理
- [x] `beforeDevCommand` 和 `beforeBuildCommand` 配置正确
- [ ] 缺少开发/生产环境差异化配置：没有 `tauri.conf.json` 的覆盖文件（如 `tauri.development.json`）或环境变量来区分 dev/prod 的窗口行为（如 dev 模式下可以保持 resizable，prod 模式可能需要固定大小等）
- [ ] `$schema` URL 不一致：`tauri.conf.json` 使用 `https://schema.tauri.app/config/2`，而 `capabilities/default.json` 使用 `https://schema.tauri.app/config/2.0.0`。建议统一为同一版本格式

### 5. 构建

- [x] `build.rs` 仅调用 `tauri_build::build()`，符合标准
- [x] `[profile.release]` 中 `strip = true` 和 `lto = true` 有助于减小二进制体积，配置合理
- [x] 图标配置引用了 5 个文件（32x32.png、128x128.png、128x128@2x.png、icon.icns、icon.ico），在 icons 目录中均存在
- [ ] icons 目录中存在未被 bundle 配置引用的图标文件：`64x64.png`、`icon.png`、`StoreLogo.png`、多个 `Square*Logo.png`（Windows Store 相关）。这些可能是为 Windows 应用商店准备但未在 bundle 中配置，或者是冗余文件。如果不需要可以清理

### 6. 其他

- [ ] `Cargo.toml` 中 `authors = ["you"]` 为占位符，建议更新为实际作者信息
- [ ] `tauri.conf.json` 中 `version` 字段为 `"0.1.0"`，这是 Tauri v2 中应用的版本号。建议确认是否需要与 `Cargo.toml` 中的 version 保持一致或使用统一的版本管理策略
- [x] `identifier: "com.bsr-monitor.app"` 格式正确（反向域名风格）

