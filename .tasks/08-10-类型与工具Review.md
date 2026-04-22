# 08-10 - 类型与工具 Review

## Review 范围

- [src/types/index.ts](src/types/index.ts) — TypeScript 类型定义
- [src/utils/format.ts](src/utils/format.ts) — 格式化工具函数
- [src/contexts/AudioContext.tsx](src/contexts/AudioContext.tsx) — 音频上下文
- [src/index.css](src/index.css) — 全局样式
- [src/main.tsx](src/main.tsx) — 前端入口
- [src/vite-env.d.ts](src/vite-env.d.ts) — Vite 类型声明

## Review 维度

### 1. 类型定义

- `types/index.ts` 中 `PlayerInfo` 包含 `loading?`、`progress?`、`status_message?` 字段 — 这些是 UI 状态而非数据模型，是否应拆分
- `PlayerWithScoreCount` 扩展 `PlayerInfo` — 如果 `PlayerInfo` 已包含 UI 状态字段，扩展后类型更复杂
- `LeaderboardResponse` 类型定义与 Rust 端 `LeaderboardResponse` 结构不匹配：
  - Rust 端：`{ id, song: SongResponse, difficulty: DifficultyResponse }`
  - TS 端：`{ leaderboard: { id, song: {...}, difficulty: {...}, mode, modeName }, scores: [...] }`
  - 这两个结构完全不同 — 前端此类型是否从未被使用或已过期
- `ScoreRow` 与 Rust 端 `ScoreRowData` — 名称不一致
- `MapScoresResponse` 中 Rust 端已移除 `no_score_groups` 改为 `no_score_diffs` — TS 类型已同步，但命令层文档未更新
- `difficultyColors` 和 `modeNames` 定义在 `types/index.ts` — 是否应放到独立的常量文件中

### 2. 工具函数

- `format.ts` 中三个函数都很简洁 — 是否有必要封装
- `formatEpochTime` 使用 `toLocaleDateString('zh-CN')` — 硬编码中文，是否应跟随系统语言
- `formatAccuracy` 固定 2 位小数 — 是否应可配置
- `formatRating` 中对 `undefined`/`null`/`0` 返回 "未评分" — `0` 是否是有效评分

### 3. 音频上下文

- `AudioContext` 只管理 `volume` — 是否过于简单，值得一个 Context
- `AudioProvider` 初始音量为 1 — 是否有必要持久化用户选择（localStorage）
- `CoverWithAudio` 中每次 `map.preview_url` 变化都创建新的 `Audio` 实例 — 在歌单模式下快速滚动时会创建大量 Audio 对象
- `CoverWithAudio` 的 cleanup 中 `audio.src = ''` — 是否会触发额外的网络请求

### 4. 全局样式

- `index.css` 中使用 `@import "tailwindcss"` — Tailwind v4 的导入语法，是否与项目版本匹配
- CSS 变量定义了一套颜色系统，但组件中大量使用 `bg-[#15151c]` 等硬编码值 — 是否应使用 `var(--color-bg-secondary)` 替代
- `body` 设置了 `overflow-y: auto` — 但 `Layout.tsx` 中 `h-screen` + `overflow-hidden` 覆盖了此行为
- `::selection` 样式使用 `rgba(255, 107, 107, 0.3)` — 是否与 `--color-accent-primary` 一致
- `@keyframes fadeIn` 和 `@keyframes slideUp` — 是否可考虑使用 Tailwind 的 `animate-` 工具类

### 5. 前端入口

- `main.tsx` — 标准的 React 18 + ReactDOM 创建根节点
- 是否有 React StrictMode — 可能导致 `useEffect` 执行两次
- 是否有全局错误边界（Error Boundary）

### 6. Vite 配置

- `vite-env.d.ts` — 是否声明了 `@tauri-apps/api` 模块类型
- `vite.config.ts` — 是否正确配置了 Tauri 插件
- 构建产物是否被 Tauri 正确引用
