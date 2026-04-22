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

## Review 发现

### 1. 类型定义（`src/types/index.ts`）

- [ ] ⚠️ CRITICAL: `PlayerInfo` 混入了 `loading?`、`progress?`、`status_message?` 三个 UI 状态字段，与纯数据模型职责冲突。`PlayerWithScoreCount` 继承 `PlayerInfo` 后也会携带这些 UI 状态，导致类型语义混乱。应拆分为 `PlayerInfo`（纯数据）和 `PlayerInfoUIState`（UI 状态），或在前端组件层用 `Record<string, PlayerUIState>` 单独管理。
- [ ] ⚠️ CRITICAL: `Playlist` 同样混入了 `loading?`、`progress?`、`status_message?` UI 状态字段，问题同上。
- [ ] ⚠️ CRITICAL: `LeaderboardResponse` 类型未被任何前端代码引用（`grep` 全局搜索仅在 `types/index.ts` 定义处命中），且其结构与 Rust 端 `LeaderboardResponse` / API 响应格式完全不同。Rust 端使用 `SongResponse`、`DifficultyResponse` 等独立结构，而 TS 端嵌套了 `leaderboard: { id, song, difficulty, mode, modeName }`。该类型极可能是残留的过期定义，应删除。
- [ ] ⚠️ CRITICAL: `ScoreRow`（TS 端）与 Rust 端 `ScoreRowData` 名称不一致，虽然字段完全对应，但跨端调试时容易造成混淆。建议统一为 `ScoreRowData` 或在注释中明确映射关系。
- [ ] `difficultyColors` 和 `modeNames` 定义在 `types/index.ts` 中。这些是展示层常量而非类型定义，建议移至 `src/constants/` 或 `src/styles/constants.ts` 等独立文件。
- [ ] `difficultyColors` 的值使用 Tailwind 工具类字符串（如 `'text-green-400 bg-green-500/20'`），属于"样式耦合"——将 Tailwind 类名硬编码在 JS 对象中。如需切换主题或调整配色，必须修改 JS 代码而非 CSS。建议改为纯 CSS 变量或 CSS 类名映射。
- [ ] `modeNames` 中的 `'ExpertPlus'` 与 `ScoreRow.tsx` 中 `difficulty` 到 Tailwind 颜色的映射逻辑重复——两处都维护了 difficulty 名称到颜色的对应关系，存在不一致风险。

### 2. 工具函数（`src/utils/format.ts`）

- [ ] `formatEpochTime` 硬编码 `'zh-CN'` 语言。Tauri 桌面应用应跟随系统语言，建议使用 `navigator.language` 或通过 Tauri `@tauri-apps/plugin-os` 获取系统 locale。
- [ ] `formatAccuracy` 固定 2 位小数。对于接近 100% 的成绩（如 99.999%），2 位精度可能不够；而对于较低成绩，2 位又显得冗余。建议提供 `decimals?: number` 参数。
- [ ] ⚠️ CRITICAL: `formatRating` 中 `if (!rating)` 将 `0` 视为 falsy 返回"未评分"。`0` 可能是一个有效的评分值（表示评分为 0%），而非"未评分"。应改为 `if (rating === undefined || rating === null)` 或 `if (rating == null)`。
- [ ] 三个工具函数都很简洁，封装本身是合理的。但 `formatEpochTime` 使用 `toLocaleDateString` 返回包含时分秒的字符串，函数名中 `Date` 一词可能误导调用者以为只返回日期部分，建议更名为 `formatDateTime` 或类似名称。

### 3. 音频上下文（`src/contexts/AudioContext.tsx` + `src/components/CoverWithAudio.tsx`）

- [ ] `AudioContext` 仅管理 `volume` 一个状态，逻辑过于简单。但作为全局音量控制的扩展点，保留 Context 是合理的。
- [ ] ⚠️ CRITICAL: `AudioProvider` 中 `volume` 初始值为 `1`，未做任何持久化。用户关闭应用后重新打开，音量恢复为最大值，体验不佳。应使用 localStorage 或 Tauri 的 `@tauri-apps/plugin-store` 持久化。
- [ ] ⚠️ CRITICAL: `CoverWithAudio` 中每次 `map.preview_url` 变化都会创建新的 `Audio` 实例（`new Audio(map.preview_url)`）。在歌单模式下快速滚动时，若每个 `MapCard` 都渲染 `CoverWithAudio`，会创建大量 Audio 对象，可能造成内存泄漏和音频资源竞争。建议复用单个 Audio 实例或使用对象池。
- [ ] `CoverWithAudio` cleanup 中 `audio.src = ''` 确实会触发一次额外的网络请求（浏览器尝试加载空字符串 URL）。应改为 `audio.srcObject = null` 或者直接不调用 `audio.src = ''`，仅 `audio.pause()` 和 `audio.remove()` 即可。
- [ ] `CoverWithAudio` 中的 SVG 进度环使用了硬编码 `stroke="#7c3aed"`（紫色），与 `index.css` 中定义的 CSS 变量无任何关联，应使用 CSS 变量。

### 4. 全局样式（`src/index.css`）

- [ ] `@import "tailwindcss"` 是 Tailwind CSS v4 的导入语法。`package.json` 中 `tailwindcss` 版本为 `^4.2.2`，匹配正确。
- [ ] CSS 变量定义了一套颜色系统（`--color-bg-primary` 等），但组件中大量使用 `bg-[#15151c]`、`bg-[#1e1e28]` 等硬编码值（`grep` 搜索发现超过 50 处）。CSS 变量形同虚设，无法实现主题切换。建议将组件中的硬编码颜色替换为对应的 CSS 变量引用。
- [ ] ⚠️ CRITICAL: `body` 设置了 `overflow-y: auto`，但 `Layout.tsx` 中 `div.h-screen.overflow-hidden` 直接覆盖此行为。如果页面内容超出视口，`body` 的滚动条永远不会出现，实际滚动由子组件控制。这导致 `body` 的 `overflow-y: auto` 是无用声明，应移除或调整布局策略。
- [ ] `::selection` 使用 `rgba(255, 107, 107, 0.3)`，而 `--color-accent-primary` 为 `#ff6b6b`。两者颜色值一致（`255, 107, 107` = `#ff6b6b`），但应使用 `color-mix(in srgb, var(--color-accent-primary) 30%)` 来保持一致性和可维护性。
- [ ] `@keyframes fadeIn` 和 `@keyframes slideUp` 可以替换为 Tailwind v4 的 `animate-fade-in` / `animate-slide-in` 工具类或 `@theme` 自定义动画，减少手写 CSS。

### 5. 前端入口（`src/main.tsx`）

- [ ] `main.tsx` 中使用了 `React.StrictMode`，这是正确的。但 StrictMode 在开发模式下会导致 `useEffect` 执行两次，某些副作用（如 `CoverWithAudio` 中创建 Audio 实例）会触发双倍行为。目前代码中 cleanup 逻辑正确，不会有严重问题，但需留意。
- [ ] ⚠️ CRITICAL: 项目中没有全局 Error Boundary。`main.tsx` 中直接渲染 `<App />`，任何未捕获的 React 错误会导致整个应用白屏。建议在 App 层级添加 Error Boundary 组件，至少提供"重新加载"按钮。

### 6. Vite 配置（`src/vite-env.d.ts` + `vite.config.ts`）

- [ ] `vite-env.d.ts` 仅包含 `/// <reference types="vite/client" />`，未显式声明 `@tauri-apps/api` 模块类型。由于 `@tauri-apps/api` 自带 `.d.ts`，TypeScript 可以自动识别，但显式声明可提高可读性。
- [ ] ⚠️ CRITICAL: `vite.config.ts` 中仅配置了 `react()` 插件，**未引入 Tauri 插件**（如 `@tauri-apps/plugin-vite` 或 `tauriPlugin`）。在 Tauri v2 中，虽然前端可以独立构建，但缺少 Tauri 插件意味着无法使用 Tauri 提供的 Vite 注入功能（如自动注入 `__TAURI__` 全局变量、dev server 配置等）。如果项目能正常运行，可能是 Tauri CLI 在构建时单独处理了这一步，但配置不完整。
- [ ] `vite.config.ts` 中 `server.port: 1420` 是 Tauri 默认端口，`strictPort: true` 配置正确。
- [ ] `clearScreen: false` 在开发时保留终端输出，是合理的 Tauri 配置习惯。
