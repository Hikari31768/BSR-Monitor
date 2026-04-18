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

## PlayerInfo 表

玩家基本信息。

| Column | Type | 说明 |
|--------|------|------|
| id | VARCHAR(40) | **主键**，BL 玩家 ID（Steam ID 格式） |
| name | TEXT | 玩家名 |
| avatarURL | TEXT | 头像 URL |
| liveURL | TEXT | 直播间地址 |
| alias | TEXT | 备注名 |

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

## PlaylistMap 表

歌单与谱面的关联关系。

| Column | Type | 说明 |
|--------|------|------|
| playlistId | INTEGER | 外键，关联 Playlist 表 |
| mapKey | VARCHAR(16) | 外键，关联 Map 表 |
| order | FLOAT | 谱面在歌单中的顺序 |

**复合主键**: `(playlistId, mapKey)`