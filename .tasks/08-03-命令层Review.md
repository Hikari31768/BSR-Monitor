# 08-03 - 命令层 Review

## Review 范围

`src-tauri/src/commands/` 目录下所有文件：
- [mod.rs](src-tauri/src/commands/mod.rs)
- [player.rs](src-tauri/src/commands/player.rs) — 玩家管理命令
- [score.rs](src-tauri/src/commands/score.rs) — 成绩管理 + 打谱查询命令
- [map.rs](src-tauri/src/commands/map.rs) — 谱面管理命令
- [playlist.rs](src-tauri/src/commands/playlist.rs) — 歌单管理命令
- [leaderboard.rs](src-tauri/src/commands/leaderboard.rs) — 排行榜查询命令

## Review 维度

### 1. 文档对齐

- 对照 [.project/Commands.md](.project/Commands.md) 检查命令签名、参数、返回值是否一致
- 验证进度事件发射是否与 [.project/Progress.md](.project/Progress.md) 描述一致
- 检查 `MapScoresResponse` 和 `PlaylistScoresResponse` 结构是否与文档定义匹配
  - 注意：文档中 `ScoreRowData` 有 `scores: PlayerScoreCell[]`，`PlayerScoreCell` 有 `is_current_version` 字段，但代码中 `PlayerScoreCell` 没有此字段

### 2. 代码质量（Rust Best Practices）

- `map.rs` 和 `playlist.rs` 中都有 `convert_map_detail` 函数 — 重复代码应提取到公共模块
- `player.rs` 中 `add_new_player` 和 `update_existing_player` 有大量相似逻辑 — 是否可以合并
- `playlist.rs` 中 `add_new_playlist` 和 `update_existing_playlist` 几乎完全相同 — 差异仅在第一页前的"已存在"检查，应重构
- `score.rs` 中 `get_map_scores` 和 `get_playlist_scores` 的成绩聚合逻辑大量重复
- 每个命令中重复的 `app.try_state::<Mutex<rusqlite::Connection>>().ok_or(...)?; state.lock().map_err(...)?;` 模式 — 是否可以封装为辅助函数

### 3. 异步模式（Rust Async Patterns）

- `add_player`、`add_playlist` 等异步命令中，API 调用和数据库操作交替进行 — 是否合理
- `update_player_scores` 中每条成绩都单独获取数据库锁 — 是否应批量处理减少锁竞争
- `BeatLeaderApi::new()` 和 `BeatSaverApi::new()` 在每个命令中重新创建 — 是否应通过 Tauri state 共享

### 4. 边界情况

- `get_map_by_hash` 不检查数据库是否已有该 hash 对应的谱面，每次都调用 API — 是否符合预期
- `get_map_scores` 中如果 `key` 不存在于数据库，返回 `Err("Map not found: ...")` — 是否应自动触发 API 获取
- `add_player` 的增量更新逻辑：如果 `live_url` 和 `alias` 为 `None`，会覆盖已有值 — 是否应保留旧值
- `add_playlist` 中 `update_existing_playlist` 会删除旧谱面关联再重新拉取 — 如果 API 调用中途失败，歌单的谱面关联会丢失
- `score.rs` 中 `PlayerScoreCell` 缺少 `is_current_version` 字段（文档中有定义），前端依赖此字段标记非当前版本分数

### 5. 错误处理

- 所有错误统一转换为 `String` — 是否丢失了足够的错误上下文
- `emit_progress_event` 失败时 `.map_err(|e| e.to_string())` — 进度发射失败是否应中止任务
- `println!` 调试日志散落在 `playlist.rs` 中 — 是否应使用 `tracing` 等结构化日志
