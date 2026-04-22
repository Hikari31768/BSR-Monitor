# 08-04 - 进度事件系统 Review

## Review 范围

[src-tauri/src/progress.rs](src-tauri/src/progress.rs)

## Review 维度

### 1. 文档对齐

- 对照 [.project/docs/Progress.md](.project/docs/Progress.md) 检查 `ProgressPayload` 字段是否一致
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

## 发现的问题

### 1. 文档对齐

- [x] `ProgressPayload` 字段与 `.project/docs/Progress.md` 文档一致
- [x] Builder 方法签名与文档描述一致
- [x] 进度阶段在命令层正确发射（add_player: 10%/20%/30%-100%/100%；add_playlist: 5%/10%/10%-100%/100%）

### 2. 代码质量

- [x] Builder 模式链式调用设计合理，符合 Rust 惯用法
- [x] `with_count` 中 total 为 0 时已做防护：`if total > 0 { ... } else { 0.0 }` (progress.rs:42)
- [ ] ⚠️ CRITICAL: `emit_progress_event` 返回 `Result<(), Box<dyn std::error::Error>>`，错误类型过于宽泛。`app.emit()` 本身返回 `Result<()>`（tauri 的 `tauri::Result`），通过 `?` 传播后包装成 `Box<dyn Error>` 丢失了具体错误信息。建议直接返回 `tauri::Result<()>` 或定义具体的 `ProgressError` 枚举。
- [ ] `with_progress` 和 `with_count` 方法都接受 `stage: String`，但两者都会覆盖 stage。如果先调用 `with_count` 再调用 `with_progress`（或反过来），stage 会被后调用者覆盖，可能导致语义不一致。建议审查调用方是否有混用场景（当前代码中未发现混用，但 API 设计允许）。

### 3. 边界情况

- [ ] ⚠️ CRITICAL: `with_player_info` 强制同时设置 `player_name` 和 `player_avatar_url`，无法只设置其中一个。如果 API 返回缺失头像 URL（虽然当前代码中 `player_response.avatar` 是 String 非 Option，不太可能为空），调用方没有单独设置 name 的途径。建议拆分为 `with_player_name` 和 `with_player_avatar_url` 两个方法，或让参数接受 `Option<String>`。
- [ ] ⚠️ CRITICAL: `task_type` 使用 `String` 而非枚举。当前在 `player.rs` 和 `playlist.rs` 中硬编码了 `"add_player"` 和 `"add_playlist"` 字符串，容易拼写错误且编译器无法检查。建议定义 `TaskType` 枚举（如 `enum TaskType { AddPlayer, AddPlaylist }`）并实现 `Serialize` 和 `Display`。
- [ ] ⚠️ CRITICAL: 前端 Players.tsx:94 和 Playlists.tsx:93 仅通过 `task_type` 匹配进度事件，**未使用 `task_id` 过滤**。虽然 `add_player` 通过 `isProcessing` 检查阻止了同玩家并发，但如果同时添加不同玩家（如两个模态框或快速点击），所有 `add_player` 类型的进度事件会同时更新所有匹配 `p.id === addForm.id` 的条目，而 `addForm.id` 是闭包捕获的旧值。更关键的是，由于事件监听是全局的（`listen('progress-update')` 接收所有事件），如果有多个同类型任务并行，前端无法区分哪个进度属于哪个任务实例。虽然后端每次 invoke 都生成了唯一的 `task_id`（Uuid::new_v4()），但前端完全没有利用这个字段。
- [ ] `emit_progress_event` 使用 `app.emit()` — 如果前端未监听该事件，消息不会堆积。Tauri 的 `emit` 是 fire-and-forget 模式，没有监听器时消息直接丢弃。这点无需担心。

### 4. 前端使用

- [ ] ⚠️ CRITICAL: **前端 Players.tsx:94 过滤条件仅检查 `task_type === 'add_player'`，未校验 `task_id`**。当存在多个并发的 add_player 操作时，A 任务的进度事件可能被 B 任务的前端监听器捕获。虽然当前 UI 通过 `isProcessing` 阻止了同一玩家重复操作，但并不阻止对不同玩家同时发起操作。如果用户快速添加多个不同玩家，各任务的 `unlisten` 回调都会收到所有 `add_player` 事件，导致进度互相串扰。
- [ ] ⚠️ CRITICAL: **前端 Playlists.tsx:93 存在同样问题** — 仅检查 `task_type === 'add_playlist'`，未校验 `task_id`。多个歌单同时添加时进度会互相干扰。
- [ ] 前端 Players.tsx 中 `listen` 返回的 `unlisten` 在 finally 块中正确清理（Players.tsx:129），这一点没问题。
- [ ] 前端 Playlists.tsx 中 `unlisten` 同样在 finally 块中正确清理（Playlists.tsx:122），这一点没问题。
- [ ] 前端通过闭包捕获 `addForm.id`（Players.tsx:96）和 `playlistId`（Playlists.tsx:95）来匹配进度，这在单一任务场景下可行，但如前所述，并发场景下 `task_type` 过滤会引入多余事件处理。如果改为同时校验 `task_id`（需要前端在 invoke 前获取后端生成的 task_id，或后端通过返回值/单独事件返回 task_id），可以彻底解决并发干扰问题。

### 5. 其他发现

- [ ] `progress.rs` 中的 `ProgressPayload::new` 初始 stage 设为 `"initial"`，但命令层从未发射过 stage 为 `"initial"` 的事件，第一个事件总是 `"fetching_player_info"` (10%) 或 `"fetching_playlist"` (5%)。`"initial"` 阶段实际上不会被用到。
- [ ] `save_playlist_maps_with_progress` 函数中，`total_pages` 参数传入的是 `current_page as u64 + 1`（playlist.rs:101, playlist.rs:211），这意味着随着分页递增，total_pages 也在动态变化，进度百分比可能出现回退（如第1页/共1页=100%，然后第2页/共2页=50%）。不过这个函数只在 `add_new_playlist` 和 `update_existing_playlist` 的循环中调用，而循环外另有最终完成事件，所以实际影响有限。
- [ ] `processing_maps` 阶段的进度计算（playlist.rs:303-304）使用了 `current_page` 和 `total_pages`，但在 `save_playlist_maps_with_progress` 被调用时，这两个值相等（playlist.rs:101, 211），导致 `(current_page as f64 - 1.0) / total_pages as f64 * 100.0` 这一项始终为 0（当 current_page=total_page=1 时），后续页面的占比也会随 total_pages 动态变化而波动。
