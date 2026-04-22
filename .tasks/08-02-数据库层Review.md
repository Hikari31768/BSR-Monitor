# 08-02 - 数据库层 Review

## Review 范围

`src-tauri/src/database/` 目录下所有文件：
- [mod.rs](src-tauri/src/database/mod.rs) — 数据库初始化
- [schema.rs](src-tauri/src/database/schema.rs) — 表结构创建
- [operations.rs](src-tauri/src/database/operations.rs) — CRUD 操作 + 类型定义 + 工具函数

## Review 维度

### 1. 文档对齐

- 对照 [.project/Database.md](.project/Database.md) 逐表检查字段是否一致
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
