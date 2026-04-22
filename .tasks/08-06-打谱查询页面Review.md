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

- 对照 [.project/UserFeatures.md](.project/UserFeatures.md) 打谱查询功能逐项检查：
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
