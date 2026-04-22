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

- 对照 [.project/UserFeatures.md](.project/UserFeatures.md) 全局功能检查：
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
