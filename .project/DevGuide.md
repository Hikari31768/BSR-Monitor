# 开发调试指南

## Tauri 应用启动

```powershell
npm run tauri dev
```

## CDP 远程调试

Windows 平台的 Tauri 2 使用 WebView2（基于 Chromium），支持通过 Chrome DevTools Protocol (CDP) 连接前端页面，实现自动化调试和 UI 交互。

### 启用 CDP

启动前设置环境变量，开启 9222 端口：

```powershell
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9222"
npm run tauri dev
```

验证 CDP 是否可用：

```powershell
curl http://localhost:9222/json/version
```

返回 JSON 表示已启用。

### 连接 agent-browser

```bash
# 连接
agent-browser connect 9222

# 获取页面快照（包含元素引用，如 @e1, @e2）
agent-browser snapshot

# 常用操作
agent-browser get title        # 页面标题
agent-browser get url          # 页面 URL
agent-browser click @e5        # 点击元素
agent-browser fill @e2 "text"  # 填充输入框
agent-browser screenshot out.png    # 截图
agent-browser eval "document.title" # 执行 JS
agent-browser wait @e1         # 等待元素可见
agent-browser close            # 断开连接
```

> 注意：在 git bash 等 Unix 风格 shell 中，`agent-browser` 命令可能不在 PATH 中，需使用 `npx agent-browser`。

### 平台限制

| 平台 | WebView | CDP 支持 |
|------|---------|----------|
| Windows | WebView2 (Chromium) | 支持 |
| macOS | WebKit | 不支持 CDP |
| Linux | WebKitGTK | 不支持 CDP |

### 安全

CDP 端口暴露后，任何本地进程都可以连接并控制应用。仅在开发环境启用，生产构建时务必移除该参数。

### 多窗口

如果应用有多个窗口，CDP 会列出所有调试目标：

```bash
curl http://localhost:9222/json/list
```

## 打包

```powershell
npm run tauri build
```

## Subagent 使用策略

### 类型与能力

| 类型 | 文件编辑 | Bash 命令 | 嵌套 Subagent | 用途 |
|------|---------|-----------|--------------|------|
| `general-purpose` | ✅ | ✅ | ❌ | 通用任务、代码编写 |
| `Explore` | ⚠️ 有限 | ⚠️ 仅限搜索 | ❌ | 代码搜索、文件查找 |
| `Plan` | ❌ | ❌ | ❌ | 架构设计、方案规划 |

类型由主 agent 通过 `subagent_type` 参数设定。选错类型会导致 subagent 无法完成对应工作。

### 上下文隔离

每个 subagent 拥有独立的上下文窗口，**不继承主会话的对话历史**。必须通过 prompt 显式传入上下文，包括：
- 任务目标和背景
- 已知的代码变更或状态
- 具体的文件路径和行号

## 任务编排模式

### 原则

涉及代码修改的多批次 Task 应按**「一会话一批次」**执行，而非在同一会话中并行多个 subagent。

| 原则 | 原因 |
|------|------|
| 每批次开独立会话 | subagent 看不到前一批次的对话和代码变更 |
| 重叠文件必须串行 | 同一文件被两个 subagent 同时修改会产生 git 冲突 |
| 批次间先合并验证 | 每批完成后应 `cargo build` / `npm run tauri dev` 确认无回归 |
| 改动区域分离可并行 | 仅 Rust 端 + 仅前端等无文件重叠的 Task 可在同批次并行 |

### 编排结构

采用**「总-分」两级**模式：

```
总任务文档（如 09-00）
├── 问题分级（P0/P1/P2/P3）
├── 依赖关系图（标识哪些批次有先后依赖）
├── 批次计划表（每个批次包含哪些 Task、串行还是并行）
├── 新会话开始清单（标准化步骤）
└── 批次交接记录（每批完成后追加，供后续批次阅读）
    ├── 批次 A 完成记录
    ├── 批次 B 完成记录
    └── ...

分任务文档（如 09-01、09-02）
├── 背景与 Review 来源
├── 具体任务清单（checklist 格式）
├── 涉及文件列表
└── 验收标准
```

### 批次间信息传递

每个批次完成后，在总任务文档的**批次交接记录**中追加：

1. **变更文件列表** — 列出被修改的所有文件路径
2. **签名/接口变更** — 函数签名、类型定义、命令返回值的变更（最影响后续批次）
3. **需注意的破坏性变更** — 如"某字段从 `String` 改为 `Option<String>`"
4. **遗留问题** — 本批次未解决但可能影响后续的问题

新会话开始时的标准流程：

1. 读取总任务文档，查看**批次交接记录**了解前序变更
2. 读取当前批次对应的分任务文档
3. `git status` + `git diff` 确认上一批次已合并到代码库
4. 开始执行

### 适用场景

| 场景 | 推荐做法 |
|------|---------|
| 只读分析（Review、搜索） | 同会话内多个 subagent 并行 |
| 独立文件的修改 | 同会话内并行 |
| 依赖前序代码变更 | 分会话串行，通过交接记录传递信息 |
| 同一文件多处关联修改 | 分会话串行 |
| 需要参考对话上下文的决策 | 主 agent 自己做，不要委托 subagent |
