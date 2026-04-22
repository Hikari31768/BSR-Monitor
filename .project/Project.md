# BSR Monitor 项目

BeatSaber 玩家成绩监控应用。

## 技术栈

- **框架**: Tauri 2
- **前端**: TypeScript + React（UI 展示）
- **后端**: Rust（数据库操作、核心业务逻辑、API 调用）
- **数据库**: SQLite (rusqlite)
- **HTTP 客户端**: reqwest (带 json feature)
- **异步运行时**: tokio (rt-multi-thread, macros)

## 模块文档

- [用户功能概览](docs/UserFeatures.md)
- [业务流程](docs/BusinessFlow.md)
- [数据库设计](docs/Database.md)
- [API 模块](docs/API.md)
- [命令层](docs/Commands.md)
- [进度事件系统](docs/Progress.md)
