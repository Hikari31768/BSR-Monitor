# 类型映射文档

本文档记录 Rust 后端与 TypeScript 前端之间的类型对应关系。

## 数据模型类型

### PlayerInfo

| Rust (`database::operations::PlayerInfo`) | TypeScript (`types/index.ts::PlayerInfo`) | 说明 |
|-------------------------------------------|-------------------------------------------|------|
| `id: String` | `id: string` | 玩家 ID |
| `name: String` | `name: string` | 玩家名称 |
| `avatar_url: Option<String>` | `avatar_url?: string` | 头像 URL |
| `live_url: Option<String>` | `live_url?: string` | 直播间 URL |
| `alias: Option<String>` | `alias?: string` | 备注名 |

### PlayerUIState

**仅存在于前端**

| TypeScript (`types/index.ts::PlayerUIState`) | 说明 |
|----------------------------------------------|------|
| `loading: boolean` | 是否正在处理中 |
| `progress: number` | 进度百分比 (0-100) |
| `statusMessage: string` | 状态消息 |

### PlayerWithScoreCount

**仅存在于前端** - 继承 PlayerInfo

| TypeScript (`types/index.ts::PlayerWithScoreCount`) | 说明 |
|------------------------------------------------------|------|
| `score_count: number` | 成绩数量 |

### PlayerScore

| Rust (`database::operations::PlayerScore`) | TypeScript (`types/index.ts::PlayerScore`) | 说明 |
|--------------------------------------------|-------------------------------------------|------|
| `player_id: String` | `player_id: string` | 玩家 ID |
| `score_id: i64` | `score_id: number` | 成绩 ID |
| `leaderboard_id: String` | `leaderboard_id: string` | 排行榜 ID |
| `map_key: Option<String>` | `map_key?: string` | 谱面 Key |
| `hash: String` | `hash: string` | 谱面 Hash |
| `mode: String` | `mode: string` | 模式 |
| `difficulty: String` | `difficulty: string` | 难度 |
| `accuracy: f32` | `accuracy: number` | 准确度 |
| `epoch_time: i64` | `epoch_time: number` | 时间戳 |

### Map

| Rust (`database::operations::Map`) | TypeScript (`types/index.ts::Map`) | 说明 |
|-----------------------------------|-----------------------------------|------|
| `key: String` | `key: string` | 谱面 Key |
| `song_name: String` | `song_name: string` | 歌曲名称 |
| `song_author_name: String` | `song_author_name: string` | 歌曲作者 |
| `level_author_name: String` | `level_author_name: string` | 谱面作者 |
| `rating: Option<f32>` | `rating?: number` | 评分 |
| `cover_url: Option<String>` | `cover_url?: string` | 封面 URL |
| `preview_url: Option<String>` | `preview_url?: string` | 预览 URL |

### MapDiff

| Rust (`database::operations::MapDiff`) | TypeScript (`types/index.ts::MapDiff`) | 说明 |
|-----------------------------------------|----------------------------------------|------|
| `key: String` | `key: string` | 谱面 Key |
| `hash: String` | `hash: string` | Hash |
| `is_current: bool` | `is_current: boolean` | 是否为当前版本 |
| `mode: String` | `mode: string` | 模式 |
| `difficulty: String` | `difficulty: string` | 难度 |

### Playlist

| Rust (`database::operations::Playlist`) | TypeScript (`types/index.ts::Playlist`) | 说明 |
|----------------------------------------|------------------------------------------|------|
| `playlist_id: i64` | `playlist_id: number` | 歌单 ID |
| `name: String` | `name: string` | 歌单名称 |
| `description: Option<String>` | `description?: string` | 描述 |
| `playlist_image: Option<String>` | `playlist_image?: string` | 封面 URL |
| `owner_id: Option<i64>` | `owner_id?: number` | 所有者 ID |
| `owner_name: Option<String>` | `owner_name?: string` | 所有者名称 |
| `total_maps: Option<i32>` | `total_maps?: number` | 谱面总数 |
| `created_at: Option<String>` | `created_at?: string` | 创建时间 |
| `songs_changed_at: Option<String>` | `songs_changed_at?: string` | 歌曲更新时间 |
| `download_url: Option<String>` | `download_url?: string` | 下载 URL |

### PlaylistUIState

**仅存在于前端**

| TypeScript (`types/index.ts::PlaylistUIState`) | 说明 |
|-----------------------------------------------|------|
| `loading: boolean` | 是否正在处理中 |
| `progress: number` | 进度百分比 (0-100) |
| `statusMessage: string` | 状态消息 |

### PlaylistMap

| Rust (`database::operations::PlaylistMap`) | TypeScript (`types/index.ts::PlaylistMap`) | 说明 |
|--------------------------------------------|-------------------------------------------|------|
| `playlist_id: i64` | `playlist_id: number` | 歌单 ID |
| `map_key: String` | `map_key: string` | 谱面 Key |
| `map_order: f32` | `map_order: number` | 顺序 |
| `map: Option<Map>` | `map?: Map` | 谱面信息 |

## 命令层返回类型

### ScoreRowData

| Rust (`commands/score.rs::ScoreRowData`) | TypeScript (`types/index.ts::ScoreRowData`) | 说明 |
|------------------------------------------|---------------------------------------------|------|
| `mode: String` | `mode: string` | 模式 |
| `difficulty: String` | `difficulty: string` | 难度 |
| `is_current_version: bool` | `is_current_version: boolean` | 是否当前版本 |
| `scores: Vec<PlayerScoreCell>` | `scores: PlayerScoreCell[]` | 玩家成绩单元格列表 |

### PlayerScoreCell

| Rust (`commands/score.rs::PlayerScoreCell`) | TypeScript (`types/index.ts::PlayerScoreCell`) | 说明 |
|--------------------------------------------|------------------------------------------------|------|
| `player_id: String` | `player_id: string` | 玩家 ID |
| `accuracy: f32` | `accuracy: number` | 准确度 |

### CurrentDiffInfo

| Rust (`commands/score.rs::CurrentDiffInfo`) | TypeScript (`types/index.ts::CurrentDiffInfo`) | 说明 |
|--------------------------------------------|-----------------------------------------------|------|
| `mode: String` | `mode: string` | 模式 |
| `difficulty: String` | `difficulty: string` | 难度 |
| `hash: String` | `hash: string` | Hash |

### MapScoresResponse

| Rust (`commands/score.rs::MapScoresResponse`) | TypeScript (`types/index.ts::MapScoresResponse`) | 说明 |
|------------------------------------------------|--------------------------------------------------|------|
| `map: db::Map` | `map: Map` | 谱面信息 |
| `current_diffs: Vec<CurrentDiffInfo>` | `current_diffs: CurrentDiffInfo[]` | 当前版本难度列表 |
| `scores: Vec<ScoreRowData>` | `scores: ScoreRowData[]` | 成绩行数据列表 |

### PlaylistScoresResponse

| Rust (`commands/score.rs::PlaylistScoresResponse`) | TypeScript (`types/index.ts::PlaylistScoresResponse`) | 说明 |
|----------------------------------------------------|------------------------------------------------------|------|
| `playlist: db::Playlist` | `playlist: Playlist` | 歌单信息 |
| `page: i32` | `page: number` | 当前页码 |
| `total_pages: i32` | `total_pages: number` | 总页数 |
| `maps: Vec<MapScoresResponse>` | `maps: MapScoresResponse[]` | 谱面成绩列表 |

## 序列化说明

### Serde 配置

Rust 端使用 `serde` 进行序列化，默认使用 snake_case 字段名。由于 TypeScript 端也使用 snake_case（与 API 返回格式一致），因此不需要额外的 `#[serde(rename_all)]` 属性。

### 可选字段

Rust 端使用 `Option<T>` 表示可选字段，TypeScript 端使用 `?:` 可选属性。这些字段在序列化时会自动处理：
- `Some(value)` → 字段包含在 JSON 中
- `None` → 字段被省略

### 特殊处理

- `i64` 类型在 JavaScript 中可能丢失精度，但目前的数据范围在 safe integer 范围内
- `f32` 类型对应 TypeScript 的 `number`
