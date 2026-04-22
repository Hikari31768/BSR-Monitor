# 08-02 - 数据库层 Review

## Review 范围

`src-tauri/src/database/` 目录下所有文件：
- [mod.rs](src-tauri/src/database/mod.rs) — 数据库初始化
- [schema.rs](src-tauri/src/database/schema.rs) — 表结构创建
- [operations.rs](src-tauri/src/database/operations.rs) — CRUD 操作 + 类型定义 + 工具函数

## Review 维度

### 1. 文档对齐

- 对照 [.project/docs/Database.md](.project/docs/Database.md) 逐表检查字段是否一致
- 检查复合主键定义是否与文档描述一致
- 验证索引创建是否与查询模式匹配
- 检查 `PlayerScore` 去重策略（按 epochTime 判断更新）是否按文档实现

### 2. 代码质量（Rust Best Practices）

- `operations.rs` 文件过大（600+行），类型定义、工具函数、CRUD 混在一起 — 是否应拆分
- 各 `query_row` + `query_map` 模式中重复的 row.get() 映射代码 — 是否可利用 `FromRow` trait 减少重复
- `insert_score` 中先查后更新的逻辑 — 是否可以利用 `INSERT OR REPLACE` + 触发器简化
- `get_scores_by_hashes_and_player` 中动态 SQL 拼接的安全性（虽为内部参数，但是否有注入风险）

### 3. 数据库设计

- 外键约束是否真正生效（rusqlite 默认不启用 PRAGMA foreign_keys）
- `PlayerScore` 表的 `mapKey` 字段为 `Option<String>` — 是否在所有写入路径中都有值
- `PlaylistMap` 的 `map` 字段（`Option<Map>`）是在查询时 JOIN 填充的，但 `insert_playlist_map` 不设置 — 是否符合预期
- 索引覆盖：`idx_playerscore_hash` 用于 `get_scores_by_hashes_and_player`，但查询条件还包含 `playerId` — 是否需要复合索引 `(playerId, hash)`

### 4. 边界情况

- `parse_leaderboard_id` 在异常输入（空字符串、纯数字、单个字符）下的行为
- `difficulty_from_value` 对非标准值返回 "Unknown" — 数据库写入时是否有校验
- `get_playlist_maps_paginated` 中使用 `format!` 拼接 `ORDER BY` 子句 — 虽然参数是内部 bool，但模式不规范
- `insert_map_diffs_batch` 中先将同 key 下所有记录的 `isCurrent` 改为 false，再插入新记录 — 如果同一 hash 在歌单中多次出现（不同页面），是否会误标记

### 5. 并发安全

- 数据库连接通过 `Mutex<Connection>` 管理 — 在高并发命令下是否会产生锁竞争
- `add_player` 等异步命令中，多次获取/释放数据库锁的模式是否正确
- 事务边界：`insert_map` + `insert_map_diffs_batch` 在多个命令中配对使用，但没有包裹在事务中 — 如果中间失败会导致不一致

---

## 发现的问题

### 1. 文档对齐

- [x] Map 表字段与 Database.md 一致（key/songName/songAuthorName/levelAuthorName/rating/coverURL/previewURL）
- [x] MapDiff 复合主键 `(key, hash, mode, difficulty)` 与文档一致
- [x] PlayerInfo 表字段与文档一致
- [x] PlayerScore 复合主键 `(playerId, hash, mode, difficulty)` 与文档一致
- [x] PlayerScore 去重策略：`insert_score` 先查后比较 epochTime，逻辑符合文档要求
- [x] Playlist 表字段与文档一致
- [x] PlaylistMap 复合主键 `(playlistId, mapKey)` 与文档一致

### 2. 代码质量

- [ ] operations.rs 文件 637 行，类型定义、工具函数、CRUD 操作混在一个文件中，建议拆分为 types.rs / utils.rs / map_ops.rs / player_ops.rs / playlist_ops.rs 等模块
- [ ] `get_map_by_key`、`get_player_by_id`、`get_playlist_by_id`、`get_score_by_pk`、`get_scores_by_player`、`get_scores_by_hashes_and_player` 等函数中存在大量重复的 `row.get()` 映射代码，建议实现 `FromRow` trait 统一处理
- [ ] `get_playlist_maps_paginated` 中使用 `format!` 拼接 `ORDER BY` 子句（`ASC`/`DESC`），虽然参数来源于内部 bool，但不符合参数化查询的最佳实践，建议用枚举定义安全的排序常量

### 3. 数据库设计

- [ ] ⚠️ CRITICAL: `mod.rs` 中 `init_database` 创建连接后未执行 `PRAGMA foreign_keys = ON`，rusqlite 默认不启用外键约束，导致 schema.rs 中定义的 `FOREIGN KEY` 形同虚设。当前代码通过 `delete_player` 和 `delete_playlist` 手动级联删除来弥补，但如果未来有其他写入路径依赖外键约束，会产生数据不一致
- [ ] `idx_playerscore_hash` 索引对 `get_scores_by_hashes_and_player` 查询（WHERE playerId = ? AND hash IN (...)）覆盖不足，建议创建复合索引 `(playerId, hash)` 以提升查询效率
- [ ] `idx_playerscore_player` 索引与 `idx_playerscore_hash` 索引存在部分重叠，若新增 `(playerId, hash)` 复合索引后，单独的 `idx_playerscore_player` 索引价值降低

### 4. 边界情况

- [ ] ⚠️ CRITICAL: `parse_leaderboard_id` 对包含多个 'x' 的输入（如 "4b302xx91"）处理异常：`split('x')` 会产生 `["4b302", "", "91"]`，`parts[0]` = "4b302"，suffix = "91"，但实际的 key 可能是 "4b302x"。应使用 `split_once('x')` 或 `rsplit_once('x')` 只按最后一个 'x' 分割
- [ ] `parse_leaderboard_id` 对空字符串输入 `""` 返回 `(None, 0, 0)`，对单字符输入 `"a"` 也返回 `(None, 0, 0)`，虽然调用方可能不会传入这类值，但函数缺乏显式的输入校验和错误返回（当前返回类型无 `Result`）
- [ ] `parse_leaderboard_id` 在 `else` 分支（不含 'x' 的情况）使用 `leaderboard_id.len() - 1` 计算索引，当输入长度为 1 时会访问 `leaderboard_id[0..0]` 得到空字符串，`parse::<i32>()` 失败返回 0 — 这个逻辑虽然不会 panic，但行为不明确
- [ ] `difficulty_from_value` 对非标准值返回 "Unknown" 字符串，数据库写入时没有校验，可能导致无效数据进入 MapDiff/PlayerScore 表
- [ ] `insert_map_diffs_batch` 中先将同 key 下所有记录的 `isCurrent` 改为 false，再插入新记录。如果同一 key 对应多个版本分布在歌单不同页面，第一次调用设置版本 A 的 `isCurrent = true`，第二次调用处理版本 B 时会先将版本 A 的 `isCurrent` 也改为 false。虽然最终最后处理的版本为 true 符合预期，但中间状态可能导致数据短暂不一致（建议用事务包裹）

### 5. 并发安全与事务边界

- [ ] ⚠️ CRITICAL: `delete_player` 先删除 PlayerScore 再删除 PlayerInfo，两条 `execute` 语句各自是独立事务（rusqlite 默认 autocommit）。如果第一条成功后第二条失败，会导致 orphaned PlayerScore 记录（虽然外键未启用所以不会约束失败）。应包裹在显式事务中
- [ ] ⚠️ CRITICAL: `delete_playlist` 先删除 PlaylistMap 再删除 Playlist，同样缺乏事务保护，存在与 `delete_player` 相同的风险
- [ ] ⚠️ CRITICAL: 调用方在 `add_player`、`add_playlist` 等命令中分别调用 `insert_map` + `insert_map_diffs_batch` + `insert_playlist_map` 等操作，这些写入操作没有被包裹在统一的事务中。如果中间某个操作失败，会导致 Map 已插入但 MapDiff 未插入等不一致状态
- [ ] `insert_score` 采用 SELECT-then-UPDATE/INSERT 模式实现去重策略，在高并发场景下（虽然 Tauri 桌面应用通常单线程访问），存在时间窗口竞争：两个线程可能同时 SELECT 到 None，然后都执行 INSERT，导致主键冲突。建议改用单条 SQL（如 `INSERT ... WHERE NOT EXISTS` 子查询）或在事务中执行
- [ ] 数据库连接通过 `Mutex<Connection>` 管理，在异步命令中每次获取锁执行一条语句后立即释放，多个操作之间没有原子性保证。对于需要多步原子性的操作，应由调用方显式管理事务（通过 `conn.transaction()` 获取事务对象，在事务持有期间锁不会释放）

### 6. 其他发现

- [ ] `PlaylistMap` 结构体中的 `map: Option<Map>` 字段在写入时不设置（`insert_playlist_map` 只写入 playlistId/mapKey/mapOrder），这是查询时通过 JOIN 填充的派生字段，符合预期，但建议添加 `#[serde(skip_serializing_if = "Option::is_none")]` 或文档注释说明
- [ ] `PlayerScore` 的 `mapKey` 字段为 `Option<String>`，在 `insert_score` 中直接写入调用方传入的值。如果 `mapKey` 为 `None`，数据库会存储 NULL，这可能导致后续通过 mapKey 关联查询时遗漏数据。建议在文档中明确该字段可为 NULL 的场景
