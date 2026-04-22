# 08-08 - 歌单管理页面 Review

## Review 范围

页面文件：
- [src/pages/Playlists.tsx](src/pages/Playlists.tsx)

相关组件：
- [src/components/PlaylistCard.tsx](src/components/PlaylistCard.tsx)
- [src/components/MapCard.tsx](src/components/MapCard.tsx)

## Review 维度

### 1. 文档对齐

- 对照 [.project/UserFeatures.md](.project/UserFeatures.md) 歌单管理功能检查：
  - 卡片网格展示歌单，显示封面和谱面数量 — 是否实现
  - 通过 BeatSaver 歌单 ID 添加/更新，逐页进度追踪 — 是否实现
  - 查看歌单内谱面列表，支持删除 — 是否实现

### 2. React 最佳实践

- 与 Players.tsx 类似的模式：`playlists` 状态扩展了 `loading`、`progress`、`status_message` — 交叉类型设计
- `handleAddPlaylist` 中同样的进度事件监听器闭包问题：`playlistId` 通过闭包捕获
- `loadPlaylists` 是简单的一次性 invoke — 是否有必要在操作完成后调用（与 Players.tsx 一致）

### 3. 状态管理

- `showMapsModal` 类型为 `number | null` — 存储当前查看谱面列表的歌单 ID
- `playlistMaps` 状态在切换不同歌单时不会自动清空 — 打开新歌单的谱面列表时，如果 API 调用较慢，可能短暂显示旧数据
- `handleDeletePlaylist` 中同样有 `loading` 检查防止删除处理中的歌单 — 但刷新页面后状态丢失

### 4. 组件设计

- `PlaylistCard` 与 `PlayerCard` 结构高度相似 — 是否可以抽取通用 `InfoCard` 组件
- `MapCard` 中难度颜色映射再次重复 — 第三次出现相同的难度颜色逻辑
- `MapCard` 接收可选的 `diffs` prop — 但在 `Playlists.tsx:251` 中使用时没有传 `diffs`，难度列表区域永远不会显示
- 谱面列表弹窗（`showMapsModal`）中使用 `pm.map && <MapCard ...>` — 如果 `pm.map` 为 `undefined`（数据库中谱面信息被删除），该条目静默不显示，用户不知道缺失

### 5. 边界情况

- 添加歌单时输入非数字 ID（如 `"abc"`），`parseInt(addId)` 会返回 `NaN` — 后端是否会正确处理
- 歌单 ID 输入框没有格式校验提示
- 歌单卡片没有「更新」按钮 — 但后端 `add_playlist` 命令支持增量更新（已存在则刷新），用户如何触发更新？
- `PlaylistCard` 中没有显示歌单的 `owner_name`（有 but 在 `loading` 时隐藏）— 加载完成后是否始终显示

### 6. 用户体验

- 谱面列表弹窗中 `MapCard` 尺寸较大（封面 28x28），在歌单谱面数量较多时滚动体验
- 歌单删除确认弹窗中只说"确定要删除该歌单吗？" — 没有提示关联的谱面关联数据也会删除（但谱面本身不会删除）
- 歌单添加/更新的进度追踪与玩家管理一致 — 体验统一
