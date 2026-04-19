# BSR Monitor 项目

BeatSaber 玩家成绩监控应用。

## 技术栈

- **框架**: Tauri 2
- **前端**: TypeScript + React（UI 展示）
- **后端**: Rust（数据库操作、核心业务逻辑、API 调用）
- **数据库**: SQLite (rusqlite)
- **HTTP 客户端**: reqwest (带 json feature)
- **异步运行时**: tokio (rt-multi-thread, macros)

## 核心模块

- [数据库设计](./Database.md)
- [API 模块](./API.md)

## 业务流程

### 玩家成绩管理

1. 添加玩家时，全量拉取该玩家所有成绩
2. 后续增量更新：按时间倒序拉取，遇到已存在的成绩则停止
3. 每个玩家对每个 `(hash, mode, difficulty)` 只保留最高分

### 谱面信息获取

- 通过 BeatSaver `/maps/hash/{hash}` 或 `/maps/id/{key}` 获取
- 通过 BeatLeader `/leaderboard/{id}` 可获取各版本（包括旧版本）的 hash

## leaderboardId 解析

BL 的 leaderboardId 格式为 `{key}{版本段}{难度}{模式}`：

- **版本段**: 空=首次上传，`x`=第二次，`xx`=第三次...
- **难度**: 1=Easy, 3=Normal, 5=Hard, 7=Expert, 9=ExpertPlus
- **模式**: 0=Legacy, 1=Standard, 2=OneSaber, 3=NoArrows, 4=90Degree, 5=360Degree, 6=Lightshow, 7=Lawless

解析规则：
1. 若存在版本段（含有一个或多个 `x`）：以这些 `x` 为锚点，向前取 key，向后依次取难度值（1位）和模式值（剩余位）
2. 若版本段为空：从末尾取1位作为模式值，再取1位作为难度值，剩余部分为 key

注意：版本段为空且模式值超过1位（非常规模式）时，会导致 key 解析不准确，但 hash 仍正确。