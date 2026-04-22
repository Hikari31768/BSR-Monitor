# 08-04 - 进度事件系统 Review

## Review 范围

[src-tauri/src/progress.rs](src-tauri/src/progress.rs)

## Review 维度

### 1. 文档对齐

- 对照 [.project/Progress.md](.project/Progress.md) 检查 `ProgressPayload` 字段是否一致
- 验证 Builder 方法签名是否与文档描述一致
- 检查进度阶段（add_player: 10%/20%/30%-100%/100%；add_playlist: 5%/10%/10%-100%/100%）是否在命令层正确发射

### 2. 代码质量

- `ProgressPayload` 使用 Builder 模式（with_xxx 链式调用）— 设计是否合理
- `with_count` 中进度计算公式 `(current as f64 / total as f64) * 100.0` — 当 total 为 0 时已处理
- `emit_progress_event` 返回 `Result<(), Box<dyn std::error::Error>>` — 错误类型是否过于宽泛

### 3. 边界情况

- `with_player_info` 同时设置 `player_name` 和 `player_avatar_url` — 如果只想设置其中一个怎么办
- `ProgressPayload` 的 `task_type` 是 `String` — 是否应使用枚举（`AddPlayer` / `AddPlaylist`）避免拼写错误
- 前端通过 `task_type` 字符串匹配来过滤事件 — 如果有多个同类型任务同时进行，仅靠 `task_type` 无法区分
- `emit_progress_event` 使用 `app.emit()` — 如果前端未监听该事件，消息是否会堆积

### 4. 前端使用

- 前端在 [Players.tsx:80-107](src/pages/Players.tsx#L80-L107) 和 [Playlists.tsx:81-103](src/pages/Playlists.tsx#L81-L103) 中监听进度
- 前端通过 `task_type` 匹配，但没有使用 `task_id` — 是否可能导致多个任务进度互相干扰
