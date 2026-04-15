# 谱面-Map

这里的谱面是指社区谱面，官方谱面内置在游戏中，不流通，不过 BeatLeader 上也会上传玩家游玩官谱之后的成绩。

谱师在 BeatSaver 上传谱面后，这张谱面会被分配到一个 key ，如 4c89a 。

此后，这个 key 就属于该谱师了，他可以对这个 key 重新上传谱面文件，因此一个 key 下可能会有多个谱面版本，不过使用 BeatSaver API 通过 key 获取谱面信息时只会返回当前被谱师设为公开的那个版本。

通常，一个 key 下的多个版本是一张谱面的多次迭代，但有时，部分谱师也会把某几个 key 作为测试专用，其下的多个版本可能没有关联，不过这类情况很少。

在一张谱面中，一组 Difficulty(难度)-Mode(模式) 作为最小游玩单元，这个最小游玩单元在日常交流中通常也被称为“难度”。

常规模式如下：

- Standard: 标准模式
- OneSaber: 单手模式
- NoArrows: 无箭头模式，所有 Note 都是圆点
- 90Degree: Note 轨道不固定，会在玩家面前90度角的范围内分布
- 360Degree: Note 轨道不固定，四面八方都有分布
- Lightshow: 没有 Note ，纯观赏
- Lawless: 谱面难度更加疯狂、谱面设计更加异想天开
- Legacy: 可能是用来给谱师存放旧版本难度

难度列表如下：

- Easy
- Normal
- Hard
- Expert
- ExpertPlus
