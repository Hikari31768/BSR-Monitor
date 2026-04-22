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

## 发现的问题

### 1. 文档对齐

- [x] `PlayerScoreCell` 缺少 `is_current_version` 字段 — 文档定义中有此字段，前端 TS 类型 `ScoreRow` 有 `is_current_version`，但 `PlayerScoreCell` 在 Rust 端（`score.rs:30-33`）和前端 TS（`index.ts:110-113`）均未定义 `is_current_version`。文档 `PlayerScoreCell` 接口定义了 `is_current_version: boolean`。代码中 `is_current_version` 放在了 `ScoreRowData` 层级而非 `PlayerScoreCell` 层级，两者结构不一致。
- [x] `MapScoresResponse` 中字段名 `no_score_diffs` vs 文档中 `no_score_groups` — 文档定义使用 `no_score_groups: NoScoreDiffGroup[]`，而代码使用 `no_score_diffs: Vec<NoScoreDiffGroup>`，字段名不一致。
- [x] `get_playlist_scores` 命令签名多了一个 `reverse: bool` 参数 — 文档中该命令只有 `playlist_id, page, player_ids` 三个参数，代码中额外多了 `reverse` 参数。

### 2. 重复代码

- [x] `convert_map_detail` 函数在 `map.rs:100-114` 和 `playlist.rs:368-382` 中完全重复定义，应提取到公共模块（如 `commands/common.rs` 或 `database/operations.rs`）。
- [x] `map.rs` 和 `playlist.rs` 中 MapDiff 批量插入逻辑完全重复（`map.rs:41-52` / `map.rs:82-93` / `playlist.rs:272-285`），三段代码结构相同，应封装为辅助函数。
- [x] `score.rs` 中 `get_map_scores` 和 `get_playlist_scores` 的成绩聚合逻辑大量重复（构建 `current_diffs`、`all_hashes`、`scores_by_diff`、`score_rows`、`no_score_diffs` 的流程完全一致），应提取为公共函数 `build_map_scores_response`。
- [x] `player.rs` 中 `add_new_player` 和 `update_existing_player` 的成绩拉取循环逻辑高度相似，仅增量判断条件不同。
- [x] `playlist.rs` 中 `add_new_playlist` 和 `update_existing_playlist` 几乎完全相同（约 80 行代码重复），唯一区别是 `update_existing_playlist` 在开始前多了一步 `delete_playlist_maps`。应合并为一个函数，用标志位控制是否先删除。
- [x] 数据库连接获取模式 `app.try_state::<Mutex<rusqlite::Connection>>().ok_or(...)?; state.lock().map_err(...)?;` 在每个命令中重复出现 20+ 次，应封装为辅助函数如 `fn get_db(app: &AppHandle) -> Result<MutexGuard<Connection>, String>`。

### 3. 异步模式 / API 客户端

- [x] `BeatLeaderApi::new()` 在 `player.rs:19`, `score.rs:436`, `leaderboard.rs:7` 中每次都重新创建；`BeatSaverApi::new()` 在 `map.rs:11/61`, `playlist.rs:14` 中每次都重新创建。若 API 客户端内部包含 HTTP client 等可共享资源，应通过 Tauri state 注入共享。
- [x] `update_player_scores` 中每条成绩都单独获取/释放数据库锁（`score.rs:474-508`），循环内每条成绩都调用 `get_score_by_pk` 然后 `insert_score`，各有一次锁获取。应改为批量查询已存在记录、批量插入，减少锁竞争。

### 4. 边界情况

- [x] `get_map_by_hash` 不检查数据库是否已有该 hash 对应的谱面，每次都调用 API。如果 hash 对应的谱面已在数据库中（但通过 key 查询未命中），会重复插入。且 `insert_map` 如果是 INSERT 而非 UPSERT，可能导致主键冲突。
- [x] `get_map_scores` 中如果 `key` 不存在于数据库，返回 `Err("Map not found: ...")` 而不自动触发 API 获取。这与 `get_map_by_key` 的行为（查不到则调 API）不一致，前端需要先调用 `get_map_by_key` 确保谱面存在。
- [x] `add_player` 增量更新中，如果 `live_url` 和 `alias` 为 `None`，会覆盖已有值。`update_existing_player` 函数（`player.rs:186-192`）构建 `PlayerInfo` 时直接使用传入的 `live_url`/`alias`（即 `None`），而非从数据库读取保留旧值。
- [x] `update_existing_playlist` 先删除所有谱面关联（`playlist.rs:185`），然后逐页重新拉取。如果 API 调用中途失败（网络问题等），歌单将丢失所有谱面关联而无回滚机制。
- [x] `update_player_scores` 的增量判断逻辑：比较的是 `score.epoch_time <= existing_score.epoch_time`，其中 `existing_score` 是通过 `get_score_by_pk` 查到的同 `(player_id, hash, mode, difficulty)` 的记录。但 API 返回的成绩按时间倒序排列，如果遇到一条已存在的记录但其 epoch_time 比本地新，会错误地继续拉取而非停止。

### 5. 错误处理

- [x] 所有错误统一转换为 `String`，丢失了错误类型信息，前端无法区分不同类型的错误（如网络错误、数据库错误、参数错误）。
- [x] `emit_progress_event` 失败时 `.map_err(|e| e.to_string())` 会导致整个任务中止。进度发射失败（如前端已断开）不应中止后台任务，建议改为 `.ok()` 忽略发射失败。
- [x] `playlist.rs` 中大量使用 `println!` 调试日志（第 53, 87, 94, 104, 108, 143, 156, 197, 204, 214, 218 行），应使用 `tracing` 等结构化日志框架。

### 6. 其他

- [x] `add_new_player` 中全量拉取成绩时，逐条 `insert_score` 而非批量插入，在数据库锁内循环（`player.rs:128-130`），效率较低。
- [x] `save_playlist_maps_with_progress` 中，每个谱面的 Map、MapDiff、PlaylistMap 分别获取/释放数据库锁（`playlist.rs:263-298`），三次独立的锁获取可合并为一次。
- [x] `add_playlist` 命令参数 `playlist_id: i32` 与 `get_playlist` 命令参数 `playlist_id: i64` 类型不一致，前端调用时需注意类型转换。
