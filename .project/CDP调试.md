# CDP 远程调试

Windows 平台的 Tauri 2 使用 WebView2（基于 Chromium），支持通过 Chrome DevTools Protocol (CDP) 连接前端页面，实现自动化调试和 UI 交互。

## 启用 CDP

启动前设置环境变量，开启 9222 端口：

```powershell
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9222"
npm run tauri dev
```

验证 CDP 是否可用：

```powershell
curl http://localhost:9222/json/version
```

返回 JSON 表示已启用。

## 连接 agent-browser

```bash
# 连接
agent-browser connect 9222

# 获取页面快照（包含元素引用，如 @e1, @e2）
agent-browser snapshot

# 常用操作
agent-browser get title        # 页面标题
agent-browser get url          # 页面 URL
agent-browser click @e5        # 点击元素
agent-browser fill @e2 "text"  # 填充输入框
agent-browser screenshot out.png    # 截图
agent-browser eval "document.title" # 执行 JS
agent-browser wait @e1         # 等待元素可见
agent-browser close            # 断开连接
```

> 注意：在 git bash 等 Unix 风格 shell 中，`agent-browser` 命令可能不在 PATH 中，需使用 `npx agent-browser`。

## 平台限制

| 平台 | WebView | CDP 支持 |
|------|---------|----------|
| Windows | WebView2 (Chromium) | 支持 |
| macOS | WebKit | 不支持 CDP |
| Linux | WebKitGTK | 不支持 CDP |

## 安全

CDP 端口暴露后，任何本地进程都可以连接并控制应用。仅在开发环境启用，生产构建时务必移除该参数。

## 多窗口

如果应用有多个窗口，CDP 会列出所有调试目标：

```bash
curl http://localhost:9222/json/list
```
