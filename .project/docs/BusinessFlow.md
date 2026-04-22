# 业务流程

## 打谱查询

1. **单谱面查询**：输入 key 或 hash → 获取谱面信息 → 查询指定玩家成绩 → 按 `(mode, difficulty)` 分组展示
2. **歌单查询**：选择歌单 → 分页获取谱面列表 → 批量查询成绩 → 逐页展示
3. **跨版本合并**：获取谱面所有版本 hash → 查询玩家在各 hash 上的成绩 → 同难度取最高 accuracy → 非当前版本标记
4. **难度排序**：按模式编号排序（Legacy放最后），每模式下按 Easy→Normal→Hard→Expert→ExpertPlus 排序

## 玩家成绩管理

1. 添加或更新玩家时，先查 DB 判断是否已存在
2. 不存在：全量拉取该玩家所有成绩
3. 已存在：刷新玩家信息 + 增量拉取新成绩（以本地最新 `epochTime` 为截止，遇到不新于本地的成绩即停止）
4. 每个玩家对每个 `(hash, mode, difficulty)` 只保留最高分

## 谱面信息获取

- 通过 BeatSaver `/maps/hash/{hash}` 或 `/maps/id/{key}` 获取
- 通过 BeatLeader `/leaderboard/{id}` 可获取各版本（包括旧版本）的 hash

## leaderboardId 解析

BL 的 leaderboardId 格式为 `{key}{版本段}{难度}{模式}`：

- **版本段**: 空=首次上传，`x`=第二次，`xx`=第三次...
- **难度**: 1=Easy, 3=Normal, 5=Hard, 7=Expert, 9=ExpertPlus
- **模式**: 0=Legacy, 1=Standard, 2=OneSaber, 3=NoArrows, 4=90Degree, 5=360Degree, 6=Lightshow, 7=Lawless

解析规则：
1. 若存在版本段（含有一个或多个 `x`）：以这些 `x` 为锚点，向前取 key，向后依次取难度值（1位）和模式值（剩余位）
2. 若版本段为空：从末尾取1位作为模式值，再取1位作为难度值，剩余部分为 key

注意：版本段为空且模式值超过1位（非常规模式）时，会导致 key 解析不准确，但 hash 仍正确。
