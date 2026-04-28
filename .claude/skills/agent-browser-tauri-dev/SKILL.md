---
name: agent-browser-tauri-dev
description: 通过 agent-browser 连接本项目的 Tauri 应用前端（开发模式）。当需要查看或操作 BSR Monitor 的界面、执行 UI 测试、调试前端表现时使用。触发词包括：看一下前端、操作一下界面、连上前端、看看程序跑起来什么样、测试一下 UI、在界面上点点看。不要自己开浏览器访问 localhost——必须通过 CDP 连接 Tauri 内嵌的 WebView2，否则 Tauri API 不会生效。
---

# agent-browser 连接 Tauri 开发模式前端

本项目的 Tauri 应用在开发模式下通过 `--remote-debugging-port=9222` 暴露了 CDP 调试端口，使得 agent-browser 可以直接连接到内嵌的 WebView2 窗口，而非启动独立的 Chrome。这样能调用完整的 Tauri 后端 API，看到真实功能的界面。

## 核心连接流程

```bash
# 1. 读取 agent-browser 核心文档（必须）
agent-browser skills get core

# 2. 关闭可能残留的旧会话
agent-browser close

# 3. 通过 CDP 连接 + 导航到目标页面（一步到位）
agent-browser --cdp 9222 open http://localhost:1420

# 4. 截图或快照
agent-browser screenshot
agent-browser snapshot -i
```

**关键**：第 3 步的 `--cdp 9222 open <url>` 组合命令同时完成连接和导航。不要分开用 `connect` 再 `navigate`。

## 为什么这样连

- Tauri 的 WebView2 在开发模式下暴露了 CDP 端口 9222（见 [package.json](package.json) 的 `dev` 脚本）
- `--cdp 9222` 告诉 agent-browser 不要启动新的 Chrome，而是连接到已有浏览器
- `open http://localhost:1420` 在同一会话中导航到 Tauri 前端 URL
- 如果只用 `connect` 命令，只建立裸连接，不创建页面上下文，后续截图/快照都会报 "Session not found"

## 常见陷阱

### 不要用 `connect` 裸连

```bash
# 错误：只建连接，没有页面上下文
agent-browser connect 9222
agent-browser screenshot          # → Session not found

# 正确：--cdp + open 一步到位
agent-browser --cdp 9222 open http://localhost:1420
agent-browser screenshot          # ✓
```

### 不要传 `localhost:9222` 给 connect

```bash
# 错误：port 参数不接受 URL 格式
agent-browser connect localhost:9222   # → invalid value

# 正确：只传端口号（但不推荐用 connect）
agent-browser connect 9222
```

### 不要用不存在的 `disconnect`

```bash
# 错误：没有 disconnect 命令
agent-browser disconnect    # → Unknown command

# 正确：用 close 关闭会话
agent-browser close
```

### 不要用不带 --cdp 的 open

```bash
# 错误：会启动独立的 Chrome，不是 Tauri WebView2
agent-browser open http://localhost:1420    # → 只看到空壳页面，Tauri API 不工作

# 正确：通过 CDP 连接已有 WebView2
agent-browser --cdp 9222 open http://localhost:1420
```

### 执行任何 agent-browser 命令前，先读文档

```bash
agent-browser skills get core
```

这是 agent-browser SKILL.md 的要求。跳过这一步会导致试错命令、产生残留会话、启动多余 Chrome 实例。

## 连接参数速查

| 参数 | 说明 |
|------|------|
| CDP 端口 | `9222`（由 package.json 的 `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` 配置） |
| 前端 URL | `http://localhost:1420`（Tauri dev 服务器地址） |
| 连接命令 | `agent-browser --cdp 9222 open http://localhost:1420` |
| 清理命令 | `agent-browser close` |

## 典型操作序列

```bash
# 连接并截图
agent-browser skills get core
agent-browser close
agent-browser --cdp 9222 open http://localhost:1420
agent-browser screenshot

# 查看可交互元素
agent-browser snapshot -i

# 点击、填写等操作
agent-browser click @e1
agent-browser fill @e2 "some text"

# 操作后重新快照（refs 已失效）
agent-browser snapshot -i
```
