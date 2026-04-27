# 开发调试指南

本目录下的开发调试相关文档：

## 开发调试命令

| 命令 | 行为 |
|------|------|
| `npm run dev` | 启动开发环境（等价于 `npm run tauri dev`） |
| `npm run dev:stop` | 停止开发环境（交互式确认） |
| `npm run dev:force` | 强制停止开发环境 |
| `npm run dev:restart` | 重启开发环境（先强制杀，再启动） |

## 构建与打包

| 命令 | 行为 |
|------|------|
| `npm run build` | 构建前端（tsc + vite build） |
| `npm run build:tauri` | 构建完整 Tauri 应用 |
| `npm run preview` | 预览前端构建结果 |

## CDP 远程调试

`npm run dev` 启动时自动注入 `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9222"`，程序运行期间可随时通过 CDP 连接 9222 端口，可用于：
- 确认 UI 设计与布局是否符合预期
- 验证交互功能是否正常触发
- 检查 DOM 结构与样式状态
- 通过 `http://localhost:9222/json` 获取页面列表，通过 `http://localhost:9222/json/version` 获取浏览器信息

详见 [CDP调试](dev/CDP调试.md)。

## 文档索引

| 文档 | 概述 |
|------|------|
| [CDP调试](dev/CDP调试.md) | Windows WebView2 平台的 Chrome DevTools Protocol 远程调试：启用方式、agent-browser 命令、平台限制、安全注意事项 |
| [Subagent使用策略](dev/Subagent使用策略.md) | 三种 subagent 类型（general-purpose/Explore/Plan）的能力差异、上下文隔离约束 |
| [任务编排模式](dev/任务编排模式.md) | 多批次 Task 的「一会话一批次」执行策略、总-分两级文档结构、批次间信息传递机制、完成验证流程、适用场景 |
