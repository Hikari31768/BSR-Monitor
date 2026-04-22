# 08-06 - 打谱查询页面 Review

## Review 范围

页面文件：
- [src/pages/ScoreTable.tsx](src/pages/ScoreTable.tsx)

相关组件：
- [src/components/ScoreTableHeader.tsx](src/components/ScoreTableHeader.tsx)
- [src/components/MapScoreGroup.tsx](src/components/MapScoreGroup.tsx)
- [src/components/ScoreRow.tsx](src/components/ScoreRow.tsx)
- [src/components/NoScoreMapRow.tsx](src/components/NoScoreMapRow.tsx)
- [src/components/PlayerSelectModal.tsx](src/components/PlayerSelectModal.tsx)
- [src/components/PlaylistSelect.tsx](src/components/PlaylistSelect.tsx)
- [src/components/CoverWithAudio.tsx](src/components/CoverWithAudio.tsx)

## Review 维度

### 1. 文档对齐

- 对照 [.project/docs/UserFeatures.md](.project/docs/UserFeatures.md) 打谱查询功能逐项检查：
  - 单谱面模式：输入 key 或 hash 查询 — 是否支持
  - 歌单模式：选择歌单、分页批量查询（每页100）— 是否一致
  - 表格展示 `(谱面, 模式, 难度)` 维度 — 是否匹配
  - 仅显示有得分的难度，无分难度折叠展示 — 是否实现
  - 歌单模式支持正/倒序切换 — 是否实现
  - 自由添加/拖拽/删除玩家列 — 是否实现
  - 点击封面播放音频预览 — 是否实现
  - 悬停 Map Key 复制 `!bsr` 命令 — 是否实现
  - 非当前版本分数标记 🕒 — 是否实现

### 2. React 最佳实践（Vercel React Best Practices）

- `ScoreTable.tsx` 中的状态管理：8 个 `useState` + 多个 `useEffect` — 是否可以合并或使用 `useReducer`
- `searchMap` 和 `searchPlaylist` 使用 `useCallback` 包裹 — 依赖数组是否正确
- `useEffect` 中 `[currentPage, viewMode, selectedPlaylistId, searchPlaylist]` 依赖 — `searchPlaylist` 是 `useCallback`，当其引用变化时是否会触发多余查询
- `loadingPlaylists` 状态被声明但解构中被忽略（`const [, setLoadingPlaylists]`）— 是否应使用
- `ScoreTableHeader` 和 `MapScoreGroup` 中 `gridTemplateColumns` 使用 inline style 动态计算 — 是否可用 CSS 变量替代

### 3. 性能

- `MapScoreGroup` 中 `gridColumns` 每次都通过 `players.map(() => '7rem').join(' ')` 计算 — 是否可以 memo 化
- `sortedPlayers = [...selectedPlayers].sort(...)` 在每次渲染时执行 — 是否可用 `useMemo`
- `displayData` 计算在每次渲染时执行 — 是否可用 `useMemo`
- 歌单模式下每页 100 个谱面，每个谱面都触发一次 `get_map_scores` 逻辑（在 `get_playlist_scores` 中循环查询）— 后端性能是否可接受

### 4. 组件设计

- `ScoreRow.tsx` 和 `NoScoreMapRow.tsx` 似乎未被 `ScoreTable.tsx` 使用 — 是废弃代码还是备用
- `MapScoreGroup` 中 `scores.length === 0` 时无成绩展示 — 与 `NoScoreMapRow` 逻辑重复
- `CopyKeyButton` 定义在 `MapScoreGroup.tsx` 内部 — 是否可以提取为独立组件
- `CoverWithAudio` 中每个封面都创建新的 `Audio` 对象 — 在歌单模式下会创建大量 Audio 实例，是否需要复用或清理
- `PlaylistSelect` 使用自定义下拉实现 — 是否应考虑使用现有库或简化

### 5. 用户体验

- 单谱面查询失败时使用 `alert()` — 是否应使用更友好的错误提示
- 歌单切换时自动触发查询（`useEffect`）— 如果用户快速切换歌单，是否会产生并发请求
- 拖拽排序的视觉反馈（`draggable` 属性）— 是否有拖拽中的视觉指示
- 玩家列删除按钮仅在 hover 时显示 — 移动端/触屏设备是否可用

### 6. TypeScript 类型

- `SelectedPlayer` 扩展 `PlayerInfo` 增加 `order` 字段 — 是否合理
- `ScoreTableHeader` 中 `onReorderPlayers` 的签名使用两个 ID 而非拖拽事件 — 是否足够灵活
- 前后端类型对齐：`PlayerScoreCell` 在 Rust 端有 `is_current_version` 吗？TS 类型定义中没有

## Review 发现的问题

### 1. 状态管理

- [ ] `ScoreTable.tsx` 有 8 个 `useState` + 多个 `useEffect`，查询相关状态（`currentMapData`、`playlistData`、`currentPage`、`mapSort`、`loadingData`）高度耦合，建议使用 `useReducer` 合并为 `queryState`，减少状态同步出错的概率
- [ ] `searchPlaylist` 的 `useCallback` 依赖项包含 `selectedPlayers`，当玩家列表变化时会创建新函数引用，进而触发 `useEffect`（第 148-152 行）重新执行查询，但此时 `selectedPlayers` 的变化并非用户主动触发查询的意图。建议将 `selectedPlayers` 从依赖中移除，使用 `useRef` 获取最新值，或拆分 effect
- [ ] `useEffect`（第 148-152 行）依赖 `[currentPage, viewMode, selectedPlaylistId, searchPlaylist]` 过于复杂。`searchPlaylist` 函数引用变化 + `viewMode` 变化都会导致不必要的查询执行。建议拆分为两个 effect：一个仅监听 `currentPage`，一个监听其他条件
- [ ] `loadingPlaylists` 被声明但 setter 被忽略（`const [, setLoadingPlaylists]`），虽然功能上不影响，但建议移除声明或在 `loadPlaylists` 中真正使用它

### 2. 性能

- [ ] `sortedPlayers = [...selectedPlayers].sort((a, b) => a.order - b.order)`（第 188 行）每次渲染都执行展开+排序，建议用 `useMemo` 包裹
- [ ] `displayData` 计算（第 183-185 行）每次渲染都执行，建议用 `useMemo` 包裹
- [ ] `MapScoreGroup` 中 `gridColumns` 通过 `players.map(() => '7rem').join(' ')`（第 54 行）每次都重新计算字符串拼接，当玩家数较多或谱面数量多时会产生重复计算，建议提取为常量 `const PLAYER_COL = '7rem'` 然后 `` `18rem 6rem ${PLAYER_COL.repeat(players.length)} 7rem` `` 或 `useMemo`
- [ ] `ScoreTableHeader` 中 `gridColumns` 同样使用 `players.map(() => '7rem').join(' ')`（第 41 行），存在相同问题
- [ ] `ScoreTableHeader` 的 `handleDragStart` / `handleDragOver` / `handleDragEnd` 每次渲染创建新函数引用，当 player 数量多时会导致大量子元素重新渲染，建议用 `useCallback` 包裹

### 3. 组件设计

- [ ] `ScoreRow.tsx` 是独立组件文件，但代码中没有任何地方 import 它。`MapScoreGroup.tsx` 中使用了 `ScoreRow` **类型**（来自 `src/types/index.ts` 第 103 行），而非此组件。此文件很可能是早期版本遗留的废弃代码，建议删除
- [ ] `NoScoreMapRow.tsx` 同样是独立组件文件，无任何地方 import。`MapScoreGroup.tsx` 在 `scores.length === 0` 时内联实现了相同的无成绩展示逻辑（第 66-120 行），与此组件功能重复。建议删除 `NoScoreMapRow.tsx`
- [ ] `CopyKeyButton` 定义在 `MapScoreGroup.tsx` 内部（第 7-36 行），虽然是函数而非组件（React 规范要求组件名大写、定义在顶层），但其实现完整且有独立状态。建议提取到独立文件 `src/components/CopyKeyButton.tsx`，提高复用性和可测试性
- [ ] `MapScoreGroup.tsx` 中 `scores.map()` 返回的 Fragment 内有多个 sibling 元素（第 164-207 行），每个都手动拼接 `key` 前缀（`diff-${index}`、`score-${index}-${player.id}`、`empty-${index}`）。使用 index 作为 key 的一部分在排序/过滤场景下可能导致 React 复用错误的 DOM 节点。虽然当前场景谱面不会动态重排序，但建议改为使用更具业务意义的 key
- [ ] `CoverWithAudio` 每个谱面实例都会创建一个 `new Audio(map.preview_url)`（第 21 行）。歌单模式下 100 个谱面 × 每页 = 100 个 Audio 对象同时存在于内存中。虽然 cleanup 函数会移除事件监听并清空 `src`，但 Audio 对象本身不会被销毁直到组件卸载。建议利用 `useAudio` context 管理一个全局 `Audio` 实例，通过切换 `src` 实现复用
- [ ] `PlaylistSelect` 自定义下拉缺少键盘导航支持（无法用方向键选择、Enter 确认、Escape 关闭），也无 ARIA 属性（`role="listbox"`、`aria-expanded` 等），无障碍访问性不佳。如果项目不计划引入第三方库，至少应添加键盘事件处理
- [ ] `PlayerSelectModal` 每次打开都重新调用 `get_player_list`（第 17-21 行），而 `ScoreTable.tsx` 初始化时已经加载过玩家列表。建议通过 props 传入已有的玩家列表，避免重复后端调用

### 4. 用户体验

- [ ] `ScoreTable.tsx` 第 109 行和第 134 行使用原生 `alert()` 展示错误信息。`alert()` 会阻塞主线程，且在 Tauri 桌面应用中样式不统一。建议使用项目已有的 Toast/Notification 组件或创建一个简单的内联错误提示
- [ ] `ScoreTable.tsx` 第 148-152 行的 `useEffect` 在歌单切换后 + `currentPage` 变化时触发查询，但无防抖/取消机制。如果用户快速切换歌单，前一个请求尚未完成时新请求已发出，可能导致：(a) 后端并发压力；(b) 竞态条件 — 先发出的请求后返回，覆盖最新结果。建议使用 `AbortController` 或在 `useEffect` cleanup 中设置取消标志
- [ ] `ScoreTableHeader` 拖拽排序缺少拖拽中的视觉反馈。`draggable` 属性仅改变光标，没有 drop indicator（如高亮目标位置、插入线等）。在玩家列较多时，用户难以判断拖拽目标位置。建议添加 `onDragOver` 时的目标高亮样式或拖拽占位符
- [ ] `ScoreTableHeader` 玩家列删除按钮使用 `opacity-0 group-hover:opacity-100`（第 106 行），仅在鼠标悬停时可见。触屏设备（如有触控屏）无法通过 hover 触发，用户可能找不到删除入口。建议对触屏设备始终显示删除按钮，或提供额外的删除操作方式
- [ ] 单谱面模式下，`searchMap` 被 `useCallback` 包裹并作为 `onKeyDown` 回调（第 232 行）。如果 `selectedPlayers` 为空，`searchMap` 内部直接 return 但不给任何提示。建议在 UI 上禁用输入框或给出"请先选择玩家"的提示

### 5. TypeScript 类型

- [x] `SelectedPlayer` 扩展 `PlayerInfo` 增加 `order` 字段 — 合理的设计，用于拖拽排序的显式排序控制，而非依赖数组位置
- [x] `ScoreTableHeader` 中 `onReorderPlayers` 使用两个 ID 参数而非拖拽事件 — 对于当前的简单 swap 场景足够，封装了拖拽细节，组件不需要关心拖拽实现
- [x] 关于检查点 "`PlayerScoreCell` 在 Rust 端有 `is_current_version` 吗？TS 类型定义中没有" — 经核实，`is_current_version` 在前后端都定义在 `ScoreRow`/`ScoreRowData` 层级（而非 `PlayerScoreCell` 层级），TS 类型 `ScoreRow` 有 `is_current_version: boolean`（`src/types/index.ts` 第 106 行），Rust 端 `ScoreRowData` 也有 `is_current_version: bool`（`src-tauri/src/commands/score.rs` 第 24 行），`MapScoreGroup.tsx` 第 175 行正确使用 `scoreRow.is_current_version`。**此检查点确认无问题**

### 6. 其他

- [ ] `ScoreTable.tsx` 第 319 行条件 `!loadingData && displayData.length === 0 && !loadingData` 中 `!loadingData` 重复了两次，应为笔误
- [ ] `ScoreTable.tsx` 第 198 行使用了 emoji `🎯`，与项目中其他地方使用图标组件/图标字体的风格不一致（如查询按钮用 `⏳`、`🔍`）。如果项目有统一的图标方案，建议保持一致
- [ ] `MapScoreGroup.tsx` 中 `formatAccuracy` 函数（第 62-64 行）在每个组件实例中重新创建，建议提取到 `src/utils/format.ts`（该文件已存在，`ScoreRow.tsx` 就从那里 import 了 `formatAccuracy`）
