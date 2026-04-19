# 数据库设计

## Map 表

谱面基本信息。

| Column | Type | 说明 |
|--------|------|------|
| key | VARCHAR(16) | **主键**，BeatSaver 谱面 key |
| songName | TEXT | 歌名 |
| songAuthorName | TEXT | 歌曲作者 |
| levelAuthorName | TEXT | 谱师 |
| rating | FLOAT | 评分 (stats.score) |
| coverURL | TEXT | 封面 URL |
| previewURL | TEXT | 音频预览 URL |

### 数据填充

**来源**: BeatSaver `/maps/id/{key}` 或 `/maps/hash/{hash}`

**时机**: 
- 用户主动查询谱面（输入 key 或 hash）
- 添加歌单时批量获取歌单内谱面信息

**字段映射** (BeatSaver MapDetail → Map 表):

| API 字段 | 表字段 |
|----------|--------|
| `id` | `key` |
| `metadata.songName` | `songName` |
| `metadata.songAuthorName` | `songAuthorName` |
| `metadata.levelAuthorName` | `levelAuthorName` |
| `stats.score` | `rating` |
| `versions[0].coverURL` | `coverURL` |
| `versions[0].previewURL` | `previewURL` |

**注意**: BeatSaver versions 数组只包含当前版本，直接取 `versions[0]` 获取封面和预览链接。

## MapDiff 表

谱面各版本的难度信息。一个 key 下可能有多个版本（hash），每个版本下有多个难度-模式组合。

| Column | Type | 说明 |
|--------|------|------|
| key | VARCHAR(16) | 外键，关联 Map 表 |
| hash | CHAR(40) | 版本 hash |
| isCurrent | BOOLEAN | 是否当前版本 |
| mode | VARCHAR(16) | 模式 |
| difficulty | VARCHAR(16) | 难度 |

**复合主键**: `(key, hash, mode, difficulty)`

模式列表：Standard, OneSaber, NoArrows, 90Degree, 360Degree, Lightshow, Lawless, Legacy

难度列表：Easy, Normal, Hard, Expert, ExpertPlus

### 数据填充

**来源**: BeatSaver MapDetail 的 `versions` 字段

**时机**: 获取谱面信息时同步填充

**字段映射** (BeatSaver MapDetail.versions → MapDiff 表):

| API 字段 | 表字段 |
|----------|--------|
| 父级 `id` | `key` |
| `hash` | `hash` |
| `true`（新插入的当前版本） | `isCurrent` |
| `diffs[].characteristic` | `mode` |
| `diffs[].difficulty` | `difficulty` |

**填充策略**: 
1. 遍历 versions 数组，对每个版本的 diffs 数组逐条插入
2. 插入前，将该 key 下所有旧记录的 `isCurrent` 改为 false
3. 新插入的记录 `isCurrent` 设为 true

同一 hash 下可能有多个难度-模式组合。versions 数组只含当前版本，其他版本的 `isCurrent` 保持 false（可能由之前查询时遗留）。

**注意**: BeatSaver versions 数组只包含当前 Published 版本，直接取 `versions[0]` 即可。旧版本 hash 无法通过 BeatSaver 获取难度列表。

## PlayerInfo 表

玩家基本信息。

| Column | Type | 说明 |
|--------|------|------|
| id | VARCHAR(40) | **主键**，BL 玩家 ID（Steam ID 格式） |
| name | TEXT | 玩家名 |
| avatarURL | TEXT | 头像 URL |
| liveURL | TEXT | 直播间地址 |
| alias | TEXT | 备注名 |

### 数据填充

**来源**: BL `/player/{id}`

**时机**: 用户添加玩家时

**字段映射** (BL PlayerResponse → PlayerInfo 表):

| API 字段 | 表字段 |
|----------|--------|
| `id` | `id` |
| `name` | `name` |
| `avatar` | `avatarURL` |
| 用户输入 | `liveURL` |
| 用户输入 | `alias` |

`liveURL` 和 `alias` 由用户手动填写，非 API 提供。

## PlayerScore 表

玩家成绩记录。每个玩家对每个 `(hash, mode, difficulty)` 只保留最高分。

| Column | Type | 说明 |
|--------|------|------|
| playerId | VARCHAR(40) | 外键，关联 PlayerInfo 表 |
| scoreId | BIGINT | BL 成绩唯一 ID |
| leaderboardId | VARCHAR(64) | BL 排行榜 ID（原始值，如 `4b302xx91`） |
| mapKey | VARCHAR(16) | 从 leaderboardId 解析得到的 key（可能不准确） |
| hash | CHAR(40) | 版本 hash |
| mode | VARCHAR(16) | 模式 |
| difficulty | VARCHAR(16) | 难度 |
| accuracy | FLOAT | 精度 |
| epochTime | BIGINT | 成绩时间（epoch 秒） |

**复合主键**: `(playerId, hash, mode, difficulty)`

### 数据填充

**来源**: BL `/player/{id}/scores/compact` (POST, count=100)

**时机**: 
- 添加玩家时：全量拉取所有成绩
- 后续更新：增量拉取（按时间倒序，遇到已存在成绩则停止）

**字段映射** (BL CompactScoreResponse.data[] → PlayerScore 表):

| API 字段 | 表字段 |
|----------|--------|
| 父级 `playerId`（请求参数） | `playerId` |
| `score.id` | `scoreId` |
| `leaderboard.id` | `leaderboardId` |
| 解析 `leaderboard.id` | `mapKey` |
| `leaderboard.songHash` | `hash` |
| `leaderboard.modeName` | `mode` |
| 难度值转换 `leaderboard.difficulty` | `difficulty` |
| `score.accuracy` | `accuracy` |
| `score.epochTime` | `epochTime` |

**难度值转换**: 1→Easy, 3→Normal, 5→Hard, 7→Expert, 9→ExpertPlus

**去重策略**: 使用复合主键 `(playerId, hash, mode, difficulty)`，插入时若已存在：
- 若新记录的 `epochTime` > 已存在记录的 `epochTime`，则更新该记录（scoreId、accuracy、epochTime）
- 若 `epochTime` ≤ 已存在记录，则忽略

**增量更新逻辑**: 按时间倒序遍历成绩，当遇到一条 `(playerId, hash, mode, difficulty, epochTime)`组合已存在于本地时，停止拉取（后续成绩都已拉取过）

## Playlist 表

歌单元数据。

| Column | Type | 说明 |
|--------|------|------|
| playlistId | INTEGER | **主键**，BeatSaver 歌单 ID |
| name | TEXT | 歌单名 |
| description | TEXT | 描述 |
| playlistImage | TEXT | 封面 URL |
| ownerId | INTEGER | 作者 BeatSaver ID |
| ownerName | TEXT | 作者名 |
| totalMaps | INTEGER | 谱面数量 |
| createdAt | TEXT (ISO8601) | 创建时间 |
| songsChangedAt | TEXT (ISO8601) | 谱面最后更新时间 |
| downloadURL | TEXT | bplist 下载链接 |

### 数据填充

**来源**: BeatSaver `/playlists/id/{id}/{page}`

**时机**: 用户添加歌单时

**字段映射** (BeatSaver PlaylistPage.playlist → Playlist 表):

| API 字段 | 表字段 |
|----------|--------|
| `playlistId` | `playlistId` |
| `name` | `name` |
| `description` | `description` |
| `playlistImage` | `playlistImage` |
| `owner.id` | `ownerId` |
| `owner.name` | `ownerName` |
| `stats.totalMaps` | `totalMaps` |
| `createdAt` | `createdAt` |
| `songsChangedAt` | `songsChangedAt` |
| `downloadURL` | `downloadURL` |

**注意**: 歌单 API 返回的 `maps` 数组包含完整谱面信息（MapDetail），可同步填充 Map 和 MapDiff 表。

## PlaylistMap 表

歌单与谱面的关联关系。

| Column | Type | 说明 |
|--------|------|------|
| playlistId | INTEGER | 外键，关联 Playlist 表 |
| mapKey | VARCHAR(16) | 外键，关联 Map 表 |
| mapOrder | FLOAT | 谱面在歌单中的顺序 |

**复合主键**: `(playlistId, mapKey)`

### 数据填充

**来源**: BeatSaver PlaylistPage.maps 数组

**时机**: 添加歌单时同步填充

**字段映射** (BeatSaver PlaylistPage.maps[] → PlaylistMap 表):

| API 字段 | 表字段 |
|----------|--------|
| 父级 `playlist.playlistId` | `playlistId` |
| `map.id` | `mapKey` |
| `order` | `mapOrder` |

**填充策略**: 遍历 maps 数组，逐条插入关联关系。同时可利用 `map` 字段（MapDetail）填充 Map 和 MapDiff 表。