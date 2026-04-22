# Tauri 命令层

将业务逻辑层功能暴露给前端调用的接口层。

## 模块结构

```
src-tauri/src/commands/
├── mod.rs           # 模块入口
├── player.rs        # 玩家管理命令
├── score.rs         # 成绩管理命令
├── map.rs           # 谱面管理命令
├── playlist.rs      # 歌单管理命令
└── leaderboard.rs   # 排行榜查询命令
```

## 命令列表

### 玩家管理

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `add_player` | id, live_url, alias | PlayerInfo | 添加或更新玩家。已存在则刷新信息+增量拉新成绩，不存在则全量拉取 |
| `get_player_list` | - | Vec<PlayerInfo> | 获取玩家列表 |
| `get_player` | id | Option<PlayerInfo> | 获取单个玩家 |
| `delete_player` | id | - | 删除玩家 |
| `update_player` | player | - | 更新玩家信息 |

### 成绩管理

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `update_player_scores` | player_id | i64 | 增量更新成绩 |
| `get_player_scores` | player_id | Vec<PlayerScore> | 获取成绩列表 |
| `get_player_score_count` | player_id | i64 | 获取成绩数量 |

### 谱面管理

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `get_map_by_key` | key | Map | 通过 key 获取谱面 |
| `get_map_by_hash` | hash | Map | 通过 hash 获取谱面 |

### 歌单管理

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `add_playlist` | playlist_id | Playlist | 添加或更新歌单。已存在则刷新信息+重新拉取谱面，不存在则全量添加 |
| `get_playlist_list` | - | Vec<Playlist> | 获取歌单列表 |
| `get_playlist` | playlist_id | Option<Playlist> | 获取单个歌单 |
| `delete_playlist` | playlist_id | - | 删除歌单 |
| `get_playlist_maps` | playlist_id | Vec<PlaylistMap> | 获取歌单谱面列表 |

### 打谱查询

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `get_map_scores` | key, player_ids | MapScoresResponse | 查询单个谱面在指定玩家下的成绩，支持跨版本合并 |
| `get_playlist_scores` | playlist_id, page, player_ids | PlaylistScoresResponse | 查询歌单某页谱面在指定玩家下的成绩（每页100谱面） |

### 打谱查询响应类型

```typescript
interface MapScoresResponse {
  map: Map;                    // 谱面基本信息
  current_diffs: MapDiff[];    // 当前版本所有难度
  no_score_groups: NoScoreDiffGroup[];  // 无分数难度（按模式分组）
  scores: ScoreRowData[];      // 成绩数据行
}

interface ScoreRowData {
  mode: string;
  difficulty: string;
  is_current_version: boolean;
  scores: PlayerScoreCell[];   // 各玩家成绩
}

interface PlayerScoreCell {
  player_id: string;
  accuracy: number;
  is_current_version: boolean;
}

interface NoScoreDiffGroup {
  mode: string;
  difficulties: string[];      // 该模式下无分数的难度列表
}

interface PlaylistScoresResponse {
  playlist: Playlist;
  page: number;
  total_pages: number;
  maps: MapScoresResponse[];
}
```

### 排行榜查询

| 命令 | 参数 | 返回 | 说明 |
|------|------|------|------|
| `get_leaderboard` | leaderboard_id | LeaderboardResponse | 获取排行榜详情 |

## 设计要点

### 数据库连接获取

通过 `AppHandle` 和 `Manager` trait 获取数据库连接：

```rust
let state = app.try_state::<Mutex<Connection>>()
    .ok_or("Database not initialized")?;
let guard = state.lock().map_err(|e| e.to_string())?;
```

### 异步命令处理

由于 `rusqlite::Connection` 不是 `Send + Sync`，不能在异步上下文中跨 await 持有数据库锁。采用分离模式：

```rust
// 1. API 调用（不持有锁）
let response = api.get_xxx().await?;

// 2. 数据库操作（获取锁，执行，释放锁）
{
    let state = app.try_state::<Mutex<Connection>>()?;
    let guard = state.lock()?;
    db::insert_xxx(&guard, &data)?;
}
```

### 错误处理

所有错误统一转换为字符串，便于前端处理：

```rust
.map_err(|e| e.to_string())
```

## 前端调用

使用 `@tauri-apps/api/core` 的 `invoke` 函数：

```typescript
import { invoke } from "@tauri-apps/api/core";

const player = await invoke("get_player", { id: "123" });
```

注意：`invoke` 只能在 Tauri WebView 中使用，普通浏览器无法调用。

## 进度事件

耗时命令（`add_player`、`add_playlist`）在执行过程中通过 Tauri 事件系统向前端推送实时进度。

### 事件

| 事件名 | Payload 类型 | 说明 |
|--------|-------------|------|
| `progress-update` | `ProgressPayload` | 进度更新 |

### ProgressPayload 结构

```rust
pub struct ProgressPayload {
    pub task_type: String,        // "add_player" 或 "add_playlist"
    pub task_id: String,          // UUID，区分不同任务
    pub stage: String,            // 当前阶段标识
    pub progress: f64,            // 0.0 - 100.0
    pub total: Option<u64>,       // 总数
    pub current: Option<u64>,     // 当前完成数
    pub message: Option<String>,  // 状态消息
    pub player_name: Option<String>,       // 玩家昵称（add_player）
    pub player_avatar_url: Option<String>, // 玩家头像（add_player）
}
```

### 前端使用

```typescript
const unlisten = await listen('progress-update', (event) => {
  const { task_type, task_id, stage, progress, message } = event.payload;
  // 更新 UI 进度显示
});
// 操作完成后清理
unlisten();
```