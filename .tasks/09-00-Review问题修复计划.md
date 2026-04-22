# 09 系列 - Review 问题修复计划

基于 08 系列全量 Review 的结果，按优先级和依赖关系制定修复计划。

## 问题分级

| 级别 | 含义 | 处理策略 |
|------|------|----------|
| **P0** | 数据不一致、崩溃风险、并发 bug | 立即修复，阻断后续工作 |
| **P1** | 类型对齐、稳定性、内存泄漏 | 紧随 P0 之后修复 |
| **P2** | 代码重复、性能、样式统一 | P0/P1 完成后并行修复 |
| **P3** | 清理、配置完善、优化 | 收尾阶段 |

## 依赖关系

```
09-01 (后端 P0) ──→ 09-03 (类型对齐)
09-02 (前端 P0/P1) ──→ 09-03 (类型对齐)
                                    │
09-04 (代码去重) ←──────────────────┘
09-05 (性能)     ←──────────────────┘
09-06 (样式统一) ────────────────────→ 09-07 (A11y/UX)
09-08 (清理收尾) ←───────────────────┘
```

## Task 列表

| 编号 | 文件 | 内容 | 优先级 |
|------|------|------|--------|
| 09-01 | 09-01-后端安全与数据完整性.md | Rust 端 P0：事务、外键、序列化、panic 修复 | P0 |
| 09-02 | 09-02-前端稳定性修复.md | 前端 P0/P1：进度并发、loading 丢失、Error Boundary、Audio 泄漏 | P0-P1 |
| 09-03 | 09-03-前后端类型对齐.md | Rust ↔ TS 类型对齐、UI 状态分离 | P1 |
| 09-04 | 09-04-代码重复提取.md | Rust 命令层/前端组件去重、公共函数提取 | P2 |
| 09-05 | 09-05-性能优化.md | useMemo、批量接口、N+1 修复、Audio 池化 | P2 |
| 09-06 | 09-06-样式与主题统一.md | 难度颜色统一、主题色体系替换、硬编码清理 | P2 |
| 09-07 | 09-07-无障碍与用户体验.md | Modal A11y、表单验证、UX 改进 | P2 |
| 09-08 | 09-08-清理与配置完善.md | 废弃文件删除、CSP、版本号动态化、依赖锁定 | P3 |

## 执行策略

采用「一会话一批次」模式。每批次在独立会话中执行，完成后在本文件末尾追加**批次交接记录**，记录关键变更以供后续批次参考。

| 批次 | Task | 会话 | 与前批依赖 |
|------|------|------|-----------|
| 1 | 09-01 | 独立 | 无 |
| 2 | 09-02 | 独立 | 无 |
| 3 | 09-03 | 独立 | 需 09-01 合并（Rust 类型变更） |
| 4 | 09-04 + 09-05 | 独立 | 需 09-03 合并（类型已对齐） |
| 5 | 09-06 | 独立 | 需 09-04+05 合并（文件重叠） |
| 6 | 09-07 + 09-08 | 独立 | 无强依赖，可并行 |

### 新会话开始清单

1. 读取本文件，查看**批次交接记录**了解前序变更
2. 读取当前批次对应的 Task 文件
3. `git status` + `git diff` 确认上一批次已合并
4. 开始执行

### 批次完成验证（每批完成后必须执行）

> ⚠️ **每批任务完成后，必须实际运行程序验证，不可跳过。**

1. `cargo build` + `npm run build` — 确认无编译错误
2. `npm run tauri dev` — 确认应用正常启动
3. 走查受影响的页面/功能 — 检查行为是否符合预期
4. 检查未修改的页面/功能 — 确认无回归
5. 在本文件**批次交接记录**中写入验证结果（通过/失败 + 具体问题）

---

# 批次交接记录

> 每批次完成后追加变更记录。后续批次执行前必须阅读此部分。

## 09-01 — 后端安全与数据完整性

**状态：已完成**

### 操作罗列

1. **数据库事务保护**
   - `delete_player`：包裹在 BEGIN/COMMIT/ROLLBACK 显式事务中
   - `delete_playlist`：同上
   - 新增 `save_playlist_map_entry`：Map + MapDiff + PlaylistMap 三操作合一事务
   - `insert_score`：改为原子化 UPDATE-then-INSERT 模式，消除 SELECT-then-UPDATE/INSERT 并发竞争

2. **外键约束启用**
   - `init_database` 中创建连接后执行 `PRAGMA foreign_keys = ON`

3. **序列化安全（serde default）**
   - `Metadata`：所有字段添加 `#[serde(default)]`
   - `PlayerResponse`：`avatar` 添加 `#[serde(default)]`
   - `CompactLeaderboard`：所有非主键字段添加 `#[serde(default)]`
   - `SongResponse`：已有 `#[serde(default)]` 的 `author`、`mapper` 不变
   - `DifficultyResponse`：`mode`、`mode_name`、`difficulty_name` 添加 `#[serde(default)]`

4. **API 客户端 panic 修复**
   - `ApiClient::new()` 改为 `Result<Self, String>` 返回
   - `BeatLeaderApi::new()` 和 `BeatSaverApi::new()` 同步改为 `Result`
   - 所有调用点（player.rs, score.rs, map.rs, playlist.rs, leaderboard.rs）添加 `.map_err(|e| e.to_string())?`

5. **重试机制改进**
   - `request_with_retry`：超时和 429 分别独立计数
   - 改为 `for _attempt in 0..=max_retries` 模式

6. **数据库索引优化**
   - `idx_playerscore_player` 替换为 `idx_playerscore_player_hash` 复合索引

7. **边界情况修复**
   - `parse_leaderboard_id`：使用 `rsplit_once('x')` 替代 `split('x')`，增加空字符串/单字符错误返回
   - 删除未使用的 `save_map_with_diffs` 和 `insert_playlist_map` 函数

### 涉及文件变更
- `src-tauri/src/database/operations.rs`：事务、原子化 insert_score、parse_leaderboard_id、新组合函数
- `src-tauri/src/database/mod.rs`：PRAGMA foreign_keys
- `src-tauri/src/database/schema.rs`：复合索引替换
- `src-tauri/src/api/types.rs`：Metadata serde default
- `src-tauri/src/api/beatleader.rs`：serde default、Result 返回
- `src-tauri/src/api/beatsaver.rs`：Result 返回
- `src-tauri/src/api/client.rs`：Result 返回、重试独立计数器
- `src-tauri/src/commands/player.rs`：API 构造 Result 处理、delete_player 返回值适配
- `src-tauri/src/commands/playlist.rs`：save_playlist_map_entry 替代三独立操作、API 构造 Result 处理
- `src-tauri/src/commands/score.rs`：API 构造 Result 处理
- `src-tauri/src/commands/map.rs`：API 构造 Result 处理
- `src-tauri/src/commands/leaderboard.rs`：API 构造 Result 处理

### 验证结果
- `cargo check`：编译通过，零警告
- `cargo build`：编译通过
- `npm run build`：编译通过
- `npm run tauri dev`：应用正常启动，数据库初始化成功，无 panic、无报错
- 受影响的命令层（player/playlist/score/map/leaderboard）：API 构造改为 Result 后调用链适配正确
- 外键 PRAGMA 已在 init_database 中注入，无冲突
- **CDP 实操测试**（通过 `--remote-debugging-port=9222` + agent-browser 连接 Tauri WebView2）：
  - 主界面（打谱查询）：正常渲染，输入框联动、按钮状态切换正常
  - 玩家管理：6→7 个玩家数据完整显示，头像/成绩数正确
  - 添加玩家弹窗：表单字段正常，关闭/取消正常
  - 添加玩家全流程：填写 ID → 提交 → API 拉取 → 新玩家 "Nao-Nao" 成功出现在列表，头像正确，0 成绩
  - 歌单管理：3 个歌单正常显示，查看/删除按钮可点击
  - 歌单谱面详情：75 首谱面列表滚动正常，成绩百分比显示正确
  - **单谱面成绩查询**：输入 Key "425d2" → 查询 → 成绩表格正确显示谱面信息、难度标签、玩家成绩百分比
  - **多人对比**：通过 "+" 按钮添加 Estrella、Nao-Nao，成绩表格动态增加列，无成绩玩家显示空白
  - **歌单成绩查询**：选择 "BSR Low-Tech (75 首)" → 75 首谱面成绩表格完整加载，双玩家成绩对比正确，"!" 图标标记无成绩难度
  - 控制台无 JS 报错

## 09-02 — 前端稳定性修复

_（待完成）_

## 09-03 — 前后端类型对齐

_（待完成）_

## 09-04/05 — 代码重复提取 + 性能优化

_（待完成）_

## 09-06 — 样式与主题统一

_（待完成）_

## 09-07/08 — 无障碍与用户体验 + 清理与配置完善

_（待完成）_

---

# 本次任务完成记录

> 以下仅记录**计划制定阶段**的操作，实际修复工作尚未开始。

## 操作罗列

1. 读取 08-01 至 08-10 共 10 个 Review Task 文件，汇总所有发现的问题
2. 按 P0-P3 优先级分类，识别依赖关系
3. 制定 09 系列 8 个 Task 文档的分工与执行顺序
4. 编写 09-00 主索引文档 + 09-01 至 09-08 共 8 个分 Task 文档

## 流程归纳

1. 从 10 个 Review 文件中提取 ~30 个 CRITICAL 和 ~50 个建议改进项
2. 按模块相关性而非纯优先级分组
3. 识别跨 Task 的依赖关系
4. 确定一会话一批次的执行策略

## 后续

实际修复从 **09-01（后端安全与数据完整性）** 开始，在独立会话中执行。
