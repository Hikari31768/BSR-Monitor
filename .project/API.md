# API 模块设计

封装 BeatLeader 和 BeatSaver API 调用，为业务层提供统一的数据获取接口。

## 模块结构

```
src/api/
├── mod.rs          # 模块入口，导出公共类型
├── client.rs       # ApiClient (HTTP 客户端，重试逻辑)
├── error.rs        # ApiError (错误类型)
├── types.rs        # 共享类型 (Metadata)
├── beatleader.rs   # BeatLeaderApi + 响应数据结构
├── beatsaver.rs    # BeatSaverApi + 响应数据结构
├── tests.rs        # 单元测试
```

## HTTP 客户端

### ApiClient

基础 HTTP 客户端，配置：
- 超时：30 秒
- User-Agent：`BSR-Monitor/0.1.0`

### 错误处理

`ApiError` 枚举：

| 类型 | 说明 |
|------|------|
| Http | reqwest HTTP 错误 |
| JsonParse | JSON 反序列化失败 |
| ApiStatus | API 返回非成功状态码 |
| RateLimited | 429 限流 |
| Timeout | 请求超时 |
| NotFound | 资源不存在 (404) |

### 重试机制

`request_with_retry` 方法：
- 最大重试次数：3 次
- 超时重试：延迟 1 秒
- 429 限流重试：延迟 5 秒

## BeatLeader API

基础 URL：`https://api.beatleader.xyz`

### 接口封装

| 方法 | API | 说明 |
|------|-----|------|
| `get_player(id)` | GET `/player/{id}` | 获取玩家信息 |
| `get_player_scores(id, page, count)` | POST `/player/{id}/scores/compact` | 获取玩家成绩 |
| `get_leaderboard(id)` | GET `/leaderboard/{id}` | 获取排行榜详情 |

### 数据结构

#### PlayerResponse

玩家基本信息。

| 字段 | 类型 | 说明 |
|------|------|------|
| id | String | Steam ID |
| name | String | 玩家名 |
| avatar | String | 头像 URL |
| country | String | 国家代码 |
| alias | Option<String> | 别名 |

#### CompactScoreResponseWithMetadata

成绩列表响应。

| 字段 | 类型 | 说明 |
|------|------|------|
| metadata | Metadata | 分页信息 |
| data | Vec<CompactScoreResponse> | 成绩列表 |

#### CompactScoreResponse

单条成绩信息。

| 字段 | 类型 | 说明 |
|------|------|------|
| score | CompactScore | 成绩详情 |
| leaderboard | CompactLeaderboard | 排行榜信息 |

#### CompactScore

成绩详情。

| 字段 | 类型 | 说明 |
|------|------|------|
| id | i32 | 成绩 ID |
| accuracy | f32 | 精度 (0-1) |
| epoch_time | i64 | 成绩时间戳 |

#### CompactLeaderboard

排行榜紧凑信息。

| 字段 | 类型 | 说明 |
|------|------|------|
| id | String | leaderboardId (如 `4b302xx91`) |
| song_hash | String | 谱面 hash |
| mode_name | String | 模式名 |
| difficulty | i32 | 难度值 |

**难度值映射**: 1→Easy, 3→Normal, 5→Hard, 7→Expert, 9→ExpertPlus

## BeatSaver API

基础 URL：`https://api.beatsaver.com`

### 接口封装

| 方法 | API | 说明 |
|------|-----|------|
| `get_map_by_key(key)` | GET `/maps/id/{id}` | 通过 key 获取谱面 |
| `get_map_by_hash(hash)` | GET `/maps/hash/{hash}` | 通过 hash 获取谱面 |
| `get_playlist(id, page)` | GET `/playlists/id/{id}/{page}` | 获取歌单详情 |

### 数据结构

#### MapDetail

谱面详细信息。

| 字段 | 类型 | 说明 |
|------|------|------|
| id | String | 谱面 key |
| name | String | 谱面名 |
| metadata | MapDetailMetadata | 元数据 |
| stats | Option<MapStats> | 统计信息 |
| versions | Vec<MapVersion> | 版本列表 |

#### MapDetailMetadata

谱面元数据。

| 字段 | 类型 | 说明 |
|------|------|------|
| song_name | String | 歌名 |
| song_author_name | String | 歌曲作者 |
| level_author_name | String | 谱师 |
| bpm | f32 | BPM |
| duration | i32 | 时长 (秒) |

#### MapStats

谱面统计。

| 字段 | 类型 | 说明 |
|------|------|------|
| score | f32 | 评分 |
| downloads | i32 | 下载量 |
| plays | i32 | 播放量 |
| upvotes | i32 | 赞数 |
| downvotes | i32 | 踩数 |

#### MapVersion

谱面版本。

| 字段 | 类型 | 说明 |
|------|------|------|
| hash | String | 版本 hash |
| cover_url | String | 封面 URL |
| preview_url | String | 音频预览 URL |
| download_url | String | 下载 URL |
| diffs | Vec<MapDifficulty> | 难度列表 |

#### MapDifficulty

难度信息。

| 字段 | 类型 | 说明 |
|------|------|------|
| characteristic | String | 模式 |
| difficulty | String | 难度名 |
| nps | f32 | 每秒音符数 |

#### PlaylistPage

歌单详情。

| 字段 | 类型 | 说明 |
|------|------|------|
| playlist | PlaylistFull | 歌单元数据 |
| maps | Vec<MapDetailWithOrder> | 谱面列表 |

## 字段命名约定

API 响应使用 camelCase，Rust 结构体使用 snake_case，通过 `#[serde(rename = "...")]` 映射。

**BeatLeader 示例**：
- API: `songHash` → Rust: `song_hash`
- API: `modeName` → Rust: `mode_name`

**BeatSaver 示例**：
- API: `songName` → Rust: `song_name`
- API: `playlistId` → Rust: `playlist_id`