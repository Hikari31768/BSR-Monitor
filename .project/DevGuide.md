# 开发调试指南

本目录下的开发调试相关文档：

## 应用启动

```powershell
npm run tauri dev
```

## 打包

```powershell
npm run tauri build
```

## 文档索引

| 文档 | 概述 |
|------|------|
| [CDP调试](dev/CDP调试.md) | Windows WebView2 平台的 Chrome DevTools Protocol 远程调试：启用方式、agent-browser 命令、平台限制、安全注意事项 |
| [Subagent使用策略](dev/Subagent使用策略.md) | 三种 subagent 类型（general-purpose/Explore/Plan）的能力差异、上下文隔离约束 |
| [任务编排模式](dev/任务编排模式.md) | 多批次 Task 的「一会话一批次」执行策略、总-分两级文档结构、批次间信息传递机制、完成验证流程、适用场景 |
