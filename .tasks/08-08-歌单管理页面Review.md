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

## 审查发现的问题

### 1. 文档对齐

- [x] 卡片网格展示歌单，显示封面和谱面数量 — 已实现（Playlists.tsx:174, PlaylistCard.tsx:11-98）
- [x] 通过 BeatSaver 歌单 ID 添加/更新，逐页进度追踪 — 已实现（Playlists.tsx:36-127），但缺少手动触发更新的 UI 入口
- [x] 查看歌单内谱面列表，支持删除 — 已实现（Playlists.tsx:146-154, 129-144）

### 2. React 最佳实践

- [ ] `playlists` 状态使用交叉类型 `Playlist & { loading?: boolean; progress?: number; status_message?: string }`（Playlists.tsx:12）来扩展 UI 状态。这种方式与 Players.tsx 中相同，但会丢失原始 `Playlist` 类型的完整性（临时条目的 `name` 被设为 `'加载中...'` 等伪造数据）。建议考虑将 UI 状态与数据模型分离，使用 Map<number, LoadingState> 管理进度，而非直接污染数据实体。
- [ ] `handleAddPlaylist` 中 `listen('progress-update', ...)` 的事件监听器通过闭包捕获 `playlistId`（Playlists.tsx:81-104）。如果用户在歌单添加过程中快速连续点击添加，前一个 `unlisten` 在 finally 中被正确清理，但 `setPlaylists` 中的 `playlistId` 闭包仍可能匹配到错误的条目（当多个歌单同时添加时）。当前有 `isProcessing` 检查防护，但防护仅针对已有歌单，新增条目不受限。
- [ ] `handleAddPlaylist` 成功后调用 `loadPlaylists()`（Playlists.tsx:111），这是一个独立的 `async` 函数但没有 `await`。这意味着成功回调不会等待列表重新加载完成，`LoadingSpinner` 可能短暂闪现，给用户造成页面重新加载的错觉。建议改为 `await loadPlaylists()`。

### 3. 状态管理

- [ ] `showMapsModal`（number | null）和 `playlistMaps`（Playlists.tsx:16-17）是两个独立状态。当切换歌单查看谱面时，如果新 API 调用较慢，`playlistMaps` 仍保留旧歌单数据，导致用户看到错误的谱面列表。**建议在 `handleViewMaps` 开头添加 `setPlaylistMaps([])` 或在关闭弹窗时清空**。
- [ ] `handleDeletePlaylist` 中通过 `playlist.loading` 检查防止删除处理中的歌单（Playlists.tsx:132）。但刷新页面后所有 loading 状态丢失，如果后端仍有未完成的添加任务，用户可能在不知情的情况下删除歌单。建议在 `loadPlaylists` 后对后端任务状态做一次校验，或后端自行拒绝删除处理中的歌单。

### 4. 组件设计

- [ ] `PlaylistCard`（PlaylistCard.tsx）与 `PlayerCard`（PlayerCard.tsx）结构高度相似：都是卡片容器 + 左侧封面/图标 + 中间信息区 + 右侧操作按钮。建议抽取通用 `InfoCard` 组件，通过 slots 或 props 配置内容区域，减少重复的 Tailwind class 和布局逻辑。
- [ ] ⚠️ CRITICAL: `MapCard` 中难度颜色映射（MapCard.tsx:48-56）是第三次重复出现。前两次分别在 `ScoreRow.tsx:14-22` 和 `MapCard.tsx:48-56` 自身。三处使用完全相同的三元链和 Tailwind 颜色值。建议提取为 `getDifficultyColor(difficulty: string)` 工具函数，统一管理。
- [ ] ⚠️ CRITICAL: `MapCard` 接收可选的 `diffs` prop（MapCard.tsx:6），但在 `Playlists.tsx:251` 中使用时仅传了 `map={pm.map}`，未传 `diffs`。这导致 MapCard 中的难度列表区域（MapCard.tsx:43-69）永远不会渲染。如果歌单谱面弹窗本意是展示谱面基础信息，应明确设计决策并考虑移除 diffs prop 或传入对应数据。

### 5. 边界情况

- [ ] 添加歌单时输入非数字 ID（如 `"abc"`），`parseInt(addId)` 返回 `NaN`（Playlists.tsx:40）。`NaN` 作为 `playlistId` 传给后端，后端 `add_playlist` 命令可能无法正确处理。建议在前端添加输入校验，拦截非数字输入。
- [ ] 歌单 ID 输入框（Playlists.tsx:194-200）没有格式校验提示。用户输入 `"123abc"` 时 `parseInt` 会截断为 `123`，行为不透明。建议改为 `type="number"` 或添加正则校验。
- [ ] ⚠️ CRITICAL: 歌单卡片没有「更新」按钮（PlaylistCard.tsx:73-97 只有"查看"和"删除"按钮）。但后端 `add_playlist` 命令支持对已存在的歌单做增量更新。用户没有任何 UI 入口来触发此操作。建议添加"刷新/更新"按钮。
- [ ] 谱面列表弹窗中 `pm.map && <MapCard ...>`（Playlists.tsx:251）：如果 `pm.map` 为 `undefined`（数据库中该谱面信息被删除或 API 返回不完整），该条目静默跳过不显示。用户无法得知有缺失谱面。建议添加缺失占位提示，如显示"谱面信息缺失 (key: {pm.map_key})"。

### 6. 用户体验

- [ ] 谱面列表弹窗中 `MapCard` 尺寸较大（封面 `w-28 h-28`，整体 padding-5、text-xl），当歌单包含数百张谱面时滚动体验较差（MapCard.tsx:11-18）。建议在歌单谱面弹窗中使用紧凑版卡片布局（缩小封面、减少间距、简化信息）。
- [ ] 歌单删除确认弹窗（Playlists.tsx:224）仅显示"确定要删除该歌单吗？"，没有提示删除的影响范围（如：谱面关联数据是否一并删除？谱面本身是否保留？）。建议在弹窗中补充说明，如"删除歌单不会移除已下载的谱面文件"。
- [ ] `PlaylistCard` 中 `owner_name` 仅在非 loading 时显示（PlaylistCard.tsx:43），但 `total_maps` 使用了 fallback `0`（PlaylistCard.tsx:52）。对于刚加载完成的歌单，如果 `total_maps` 为 `undefined`，会显示"0 谱面"而非更友好的提示。建议区分"加载中"和"已加载但无谱面"的状态。
- [ ] `handleAddPlaylist` 中 `isProcessing` 检查（Playlists.tsx:43-50）通过 `alert` 提示用户，但 alert 是浏览器原生弹窗，与项目整体的 Modal 组件风格不一致。建议改为自定义提示或使用 Toast/通知组件。

