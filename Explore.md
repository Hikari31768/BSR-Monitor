玩家列表

玩家成绩列表

# BL

## get /player/{id}/scores/compact

按日期倒序拉取玩家所有成绩，每页拉100条

```
CompactScoreResponseResponseWithMetadata
    data
        score
            accuracy                // 0.9273462
        leaderboard
            id                          // "4b302xx91" 可以从中得到 key、版本、难度、模式
            songHash
            modeName        // "Standard"
            difficulty              // 9
```

## get /player/{id}

```
name
avatar
id
```

# Saver

## get /maps/id/{id}

```
MapDetail
    id                          // key
    metadata
        songName
    stats
        score                 // 玩家评分
    versions
        hash
        diffs
            characteristic  // Standard OneSaber NoArrows 90Degree 360Degree Lightshow Lawless Legacy
            difficulty          // Easy Normal Hard Expert ExpertPlus
        coverURL
        previewURL      // 音频预览
```

# DataBase

## Map

| Column | Type | Example |
| - | - | - |
| key | VARCHAR(16) | "4c89a" |
| songName |  TEXT | "Windows 7 Startup" |
| rating | FLOAT | 0.7807 |
| coverURL | TEXT | "https://eu.cdn.beatsaver.com/f01764441f594952b39013bd774feae7533afcc8.jpg" |
| previewURL | TEXT | "https://eu.cdn.beatsaver.com/f01764441f594952b39013bd774feae7533afcc8.mp3" |

## MapDiff

| Column | Type | Example |
| - | - | - |
| key | VARCHAR(16) | "4c89a" |
| hash | CHAR(40) | "f01764441f594952b39013bd774feae7533afcc8" |
| isCurrent | BOOLEAN | true |
| mode | VARCHAR(16) | "Standard" |
| difficulty | VARCHAR(16) | "ExpertPlus" |

## PlayerInfo

| Column | Type | Example |
| - | - | - |
| id | VARCHAR(40) | "76561198159152141" |
| name | TEXT | "YoLeax" |
| avatarURL | TEXT | "https://avatars.akamai.steamstatic.com/4633a745fc6f5a77637c5370748246e0f3dbd7ce_full.jpg" |

## PlayerScore

| Column | Type | Example |
| - | - | - |
| playerId | VARCHAR(40) | "76561198159152141" |
| mapKey | VARCHAR(16) | "4b302" |
| leaderboardId | VARCHAR(64) | "4b302xx91" |
| hash | CHAR(40) | "83bc2dd5c392c2f18594d56c4f71762db2955672" |
| mode | VARCHAR(16) | "Standard" |
| difficulty | VARCHAR(16) | "ExpertPlus" |
| accuracy | FLOAT | 0.9273462 |
| epochTime | BIGINT | 1775478132 |

