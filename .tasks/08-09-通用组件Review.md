# 08-09 - 通用组件 Review

## Review 范围

- [src/components/Layout.tsx](src/components/Layout.tsx) — 页面布局
- [src/components/Sidebar.tsx](src/components/Sidebar.tsx) — 侧边栏导航
- [src/components/Modal.tsx](src/components/Modal.tsx) — 弹窗
- [src/components/LoadingSpinner.tsx](src/components/LoadingSpinner.tsx) — 加载指示器
- [src/components/EmptyState.tsx](src/components/EmptyState.tsx) — 空状态
- [src/components/DiffBadge.tsx](src/components/DiffBadge.tsx) — 难度标签
- [src/components/ModeIcon.tsx](src/components/ModeIcon.tsx) — 模式图标

## Review 维度

### 1. 文档对齐

- 对照 [.project/docs/UserFeatures.md](.project/docs/UserFeatures.md) 全局功能检查：
  - 侧边栏导航：Logo、三个页面入口、音量滑块、版本号 — 是否完整
  - 暗色主题 + 红色强调色 — 是否一致
  - 通用组件：难度标签、模式图标、弹窗、加载指示器、空状态 — 是否都实现

### 2. React 最佳实践

- `Layout.tsx` 中使用 `Outlet` + `BrowserRouter` — 路由结构是否合理
- `AudioProvider` 包裹整个 Layout — 音频上下文是否应在路由外层
- `Modal.tsx` 通过 `isOpen` 控制渲染（`if (!isOpen) return null`）— 是否应使用 `display: none` 保持 DOM 节点以支持动画
- `Modal` 的 `onClose` 只在关闭按钮上调用 — 点击遮罩层外部是否应关闭
- `Modal` 中缺少键盘事件处理（Escape 键关闭）— 无障碍访问问题
- `Modal` 缺少焦点陷阱（focus trap）— 打开弹窗后 Tab 键是否会跑到背景元素

### 3. 样式一致性

- `Sidebar.tsx` 中版本号硬编码为 `v0.1.0` — 是否应从环境变量或 Tauri 配置中获取
- `Sidebar.tsx` 中导航项硬编码为数组 — 是否应与路由配置联动
- 音量滑块使用 inline `linear-gradient` style — 当 `volume` 变化时每次都重新计算
- 侧边栏中的 emoji 图标（🎯👤📋）与页面标题中的 emoji 重复 — 是否应使用 SVG 图标
- `DiffBadge` 中的难度颜色配置与 `ScoreRow.tsx`、`PlayerCard.tsx`、`PlaylistCard.tsx`、`MapCard.tsx` 中的颜色定义不一致（使用了不同的色值）— 应统一到一处

### 4. 组件设计

- `ModeIcon` 组件依赖 `icons/` 目录下的 SVG 文件 — 这些文件是否存在于 `src-tauri/icons/` 还是 `public/icons/`？路径是否正确
- `ModeIcon` 的 `onError` 处理只隐藏了 img 标签，但 fallback 文本在 `div` 外部 — 失败时可能只显示空白
- `ModeIcon` 内部有 `modeNames` 映射 — 与 [types/index.ts](src/types/index.ts) 中的 `modeNames` 重复
- `LoadingSpinner` 和 `EmptyState` 设计简洁 — 是否满足所有使用场景
- `EmptyState` 的 `icon` prop 默认值为 `📭` — 是否应在各处使用时都显式指定

### 5. 无障碍访问

- `Modal` 缺少 `role="dialog"` 和 `aria-modal="true"`
- `Sidebar` 中音量滑块缺少 `aria-label`
- `LoadingSpinner` 缺少 `aria-live` 或 `role="status"` — 屏幕阅读器无法感知加载状态
- `DiffBadge` 只有视觉展示，缺少 `title` 或 `aria-label` 说明含义
- 导航链接的 emoji 图标对屏幕阅读器来说可能读出无意义的字符

### 6. Tailwind CSS 使用

- 大量使用任意值（如 `bg-[#15151c]`、`border-[#2a2a3a]`）— 是否应在 Tailwind 配置中定义为自定义主题色
- `index.css` 中定义了 CSS 变量但只在 `body` 等少数地方使用 — 是否应更广泛地利用
- `appearance-none` 在音量滑块上的使用 — 浏览器兼容性

---

## 发现的问题

### Layout.tsx

- [x] `AudioProvider` 包裹整个 Layout — 位置合理，所有路由组件均可访问音频上下文
- [x] `Outlet` 在 `main` 区域内 — 路由结构合理
- [ ] 根容器 `div` 使用 `bg-[#0d0d12]` 而非 Tailwind 配置中的 `bg-bs-bg-primary` 或 CSS 变量 `var(--color-bg-primary)`，与自定义主题脱节
- [ ] 页面内容区 `max-w-6xl` 硬编码为固定值，在大屏下可能留白过多，考虑是否应响应式适配

### Sidebar.tsx

- [ ] ⚠️ CRITICAL: 版本号 `v0.1.0` 硬编码在组件内（第 70 行），而 `src-tauri/tauri.conf.json` 中已有 `"version": "0.1.0"`，应从 Tauri 配置或环境变量读取，避免两处不一致
- [ ] 导航项 `navItems` 数组硬编码在组件内（第 4-8 行），未与路由配置联动，新增路由时需手动维护两处
- [ ] 导航项使用 emoji 图标（🎯👤📋），与 `index.css` 中定义的现代设计风格不统一，建议替换为 SVG 图标组件
- [ ] 音量滑块 `background` inline style 中 `linear-gradient` 使用 `#7c3aed` 紫色硬编码，未在 `tailwind.config.js` 的 `bs.accent` 或 `index.css` 变量中定义
- [ ] 音量滑块 `<input type="range">` 缺少 `aria-label="音量控制"` 或 `aria-labelledby`，屏幕阅读器无法识别其用途
- [ ] 音量滑块 `onChange` 中使用 `parseFloat(e.target.value)`，未对 `NaN` 做防御性处理
- [ ] `NavLink` 激活态使用 `bg-[#ff6b6b]/10` 和 `border-[#ff6b6b]/20`，应使用 `bg-bs-accent-primary/10` 等主题色
- [ ] 侧边栏背景 `bg-[#15151c]` 应改为 `bg-bs-bg-secondary`
- [ ] 边框色 `border-[#2a2a3a]` 应改为 `border-bs-border`
- [ ] 文字色 `text-[#6b7280]`、`text-[#a0a0b0]`、`text-[#f0f0f5]` 应分别改为 `text-bs-text-muted`、`text-bs-text-secondary`、`text-bs-text-primary`
- [ ] 导航未激活态 `hover:bg-[#1e1e28]` 应改为 `hover:bg-bs-bg-tertiary`

### Modal.tsx

- [ ] ⚠️ CRITICAL: 缺少 `role="dialog"` 和 `aria-modal="true"` 属性，屏幕阅读器无法正确识别为对话框
- [ ] ⚠️ CRITICAL: 缺少键盘事件处理，按 `Escape` 键无法关闭弹窗
- [ ] ⚠️ CRITICAL: 缺少焦点陷阱（focus trap），打开弹窗后 Tab 键会聚焦到背景元素
- [ ] ⚠️ CRITICAL: 点击遮罩层外部（`bg-black/60` 区域）未触发 `onClose`，用户缺少直观的关闭方式
- [ ] 关闭按钮使用 `✕` 字符而非 SVG 图标，视觉一致性欠佳，且缺少 `aria-label="关闭"`
- [ ] 弹窗容器背景 `bg-[#15151c]` 应改为 `bg-bs-bg-secondary`
- [ ] 弹窗边框 `border-[#2a2a3a]` 应改为 `border-bs-border`
- [ ] 标题文字色 `text-[#f0f0f5]` 应改为 `text-bs-text-primary`
- [ ] 关闭按钮文字色 `text-[#6b7280]` 应改为 `text-bs-text-muted`
- [ ] 使用 `if (!isOpen) return null` 卸载组件而非 `display: none`，不支持进入/退出动画（虽然已有 `animate-fade-in` / `animate-slide-up` 类名，但组件卸载时动画不会播放）

### LoadingSpinner.tsx

- [ ] 缺少 `role="status"` 属性，屏幕阅读器无法感知加载状态
- [ ] 缺少 `aria-live="polite"` 或配套的 `<span className="sr-only">加载中...</span>` 文本
- [ ] 边框色 `border-[#2a2a3a]` 应改为 `border-bs-border`
- [ ] 旋转边框色 `border-[#ff6b6b]` 应改为 `border-bs-accent-primary`

### EmptyState.tsx

- [ ] 默认 icon 为 emoji `📭`，与项目整体使用 SVG 图标的方向不一致
- [ ] 容器背景 `bg-[#1e1e28]` 应改为 `bg-bs-bg-tertiary`
- [ ] 容器边框 `ring-[#2a2a3a]` 应改为 `ring-bs-border`
- [ ] 文字色 `text-[#6b7280]` 应改为 `text-bs-text-muted`

### DiffBadge.tsx

- [ ] ⚠️ CRITICAL: 难度颜色配置与 `tailwind.config.js` 中定义的 `difficulty` 颜色严重不一致。例如：
  - `Easy`: 组件用 `#347e55` / Tailwind 配 `#4ade80` (text-green-400)
  - `Normal`: 组件用 `#2f66a0` / Tailwind 配 `#60a5fa` (text-blue-400)
  - `Hard`: 组件用 `#b15c11` / Tailwind 配 `#fbbf24` (text-yellow-400)
  - `Expert`: 组件用 `#a83922` / Tailwind 配 `#f87171` (text-red-400)
  - `ExpertPlus`: 组件用 `#7049ae` / Tailwind 配 `#c084fc` (text-purple-400)
  组件使用的是内联 style 色值，应统一使用 `tailwind.config.js` 中的 `difficulty.*` 颜色配置
- [ ] 缺少 `title` 或 `aria-label` 属性，屏幕阅读器无法理解难度标签含义（如 "ExpertPlus" 的语义）
- [ ] 使用内联 `style` 设置 `color`、`backgroundColor`、`border`，无法利用 Tailwind 的暗色模式响应和主题切换

### ModeIcon.tsx

- [ ] ⚠️ CRITICAL: `modeNames` 映射（第 17-26 行）与 `src/types/index.ts` 第 145-154 行的 `modeNames` 完全重复，应导入 types 中的定义
- [ ] SVG 图标路径 `icons/standard.svg` 等位于项目根目录 `icons/` 文件夹，在 Vite 构建中应放在 `public/icons/` 目录下才能通过相对路径正确引用；当前路径 `icons/` 在 `src/components/ModeIcon.tsx` 中引用会解析为相对于当前文件的路径，打包后可能 404
- [ ] `onError` 处理仅 `e.currentTarget.style.display = 'none'` 隐藏图片，但 fallback 仅在没有 `iconPath` 时渲染（第 46-48 行的 `else` 分支），图标加载失败时会显示空白而非缩写文本
- [ ] `mode` prop 类型为 `string`，缺少类型约束（应使用 `keyof typeof modeIcons` 或单独的类型定义）
- [ ] 当 `mode` 不在 `modeIcons` 映射中时，显示 `mode.slice(0, 2)` 缩写，但未对空字符串或短字符串做防御

### 全局 Tailwind CSS 问题

- [ ] 整个组件层大量使用 Tailwind 任意值（`bg-[#15151c]`、`border-[#2a2a3a]`、`text-[#6b7280]` 等），而 `tailwind.config.js` 已定义了完整的 `bs.*` 色阶体系但未充分利用。建议全面替换为语义化类名如 `bg-bs-bg-secondary`、`border-bs-border`、`text-bs-text-muted` 等
- [ ] `index.css` 中定义的 CSS 自定义变量（`--color-bg-primary` 等）在组件中几乎未被使用，与 Tailwind 自定义主题存在重复，建议二选一统一使用

