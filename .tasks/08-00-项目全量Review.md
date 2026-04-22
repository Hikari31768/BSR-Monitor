# 08 系列 - 项目全量 Review

对项目各模块、各页面进行一次完整的代码审查，检查实现质量、一致性、潜在问题，并与 .project 文档对齐。

## 技术栈

- **Tauri v2**: 使用 `tauri-v2` skill
- **Rust**: 使用 `rust-best-practices` skill
- **Rust async**: 使用 `rust-async-patterns` skill
- **React + TypeScript**: 使用 `vercel-react-best-practices` skill
- **TypeScript**: 使用 `typescript-advanced-types` skill
- **Tailwind CSS**: 使用 `tailwind-css-patterns` skill

## Review 策略

每个 Review Task 按以下维度检查：

1. **文档对齐**: 代码是否与 .project 文档描述一致
2. **代码质量**: 命名、结构、复用、错误处理
3. **最佳实践**: 对应技术栈 skill 的建议
4. **边界情况**: 空值、异常、并发、性能
5. **安全性**: 注入、路径穿越、数据泄露

## Task 列表

| 编号 | 文件 | 内容 |
|------|------|------|
| 08-01 | 08-01-API模块Review.md | src-tauri/src/api/ 全套文件 |
| 08-02 | 08-02-数据库层Review.md | src-tauri/src/database/ 全套文件 |
| 08-03 | 08-03-命令层Review.md | src-tauri/src/commands/ 全套文件 |
| 08-04 | 08-04-进度事件系统Review.md | src-tauri/src/progress.rs |
| 08-05 | 08-05-Tauri主配置Review.md | src-tauri/src/main.rs, lib.rs, tauri.conf |
| 08-06 | 08-06-打谱查询页面Review.md | ScoreTable.tsx 及相关组件 |
| 08-07 | 08-07-玩家管理页面Review.md | Players.tsx 及相关组件 |
| 08-08 | 08-08-歌单管理页面Review.md | Playlists.tsx 及相关组件 |
| 08-09 | 08-09-通用组件Review.md | Layout, Sidebar, Modal 等共享组件 |
| 08-10 | 08-10-类型与工具Review.md | types/, utils/, 全局样式 |

## 执行策略

### 执行方式

使用 subagent 并行执行 10 个 Review Task。所有 Task 仅做**只读分析**，将发现的问题以 checklist 形式追加到对应 Task 文件底部，**不做代码修改**。全部 Review 完成后汇总问题，按优先级统一修复。

### 两批并行

为避免同时运行过多 agent 造成资源压力，分为两批：

**第一批：后端 Rust（5 个 agent 并行）**
- 08-01 API 模块 → 读取 `.tasks/08-01-API模块Review.md` 中的检查点，审查 `src-tauri/src/api/` 下全部文件
- 08-02 数据库层 → 读取 `.tasks/08-02-数据库层Review.md` 中的检查点，审查 `src-tauri/src/database/` 下全部文件
- 08-03 命令层 → 读取 `.tasks/08-03-命令层Review.md` 中的检查点，审查 `src-tauri/src/commands/` 下全部文件
- 08-04 进度事件系统 → 读取 `.tasks/08-04-进度事件系统Review.md` 中的检查点，审查 `src-tauri/src/progress.rs`
- 08-05 Tauri 主配置 → 读取 `.tasks/08-05-Tauri主配置Review.md` 中的检查点，审查 `src-tauri/src/main.rs`、`lib.rs`、`tauri.conf.json`、`Cargo.toml` 等

**第二批：前端 React（5 个 agent 并行）**
- 08-06 打谱查询页面 → 读取 `.tasks/08-06-打谱查询页面Review.md` 中的检查点，审查对应页面及组件
- 08-07 玩家管理页面 → 读取 `.tasks/08-07-玩家管理页面Review.md` 中的检查点，审查对应页面及组件
- 08-08 歌单管理页面 → 读取 `.tasks/08-08-歌单管理页面Review.md` 中的检查点，审查对应页面及组件
- 08-09 通用组件 → 读取 `.tasks/08-09-通用组件Review.md` 中的检查点，审查对应共享组件
- 08-10 类型与工具 → 读取 `.tasks/08-10-类型与工具Review.md` 中的检查点，审查对应类型、工具、上下文、样式

### Agent 编写要求

每个 agent 的 prompt 应包含：
1. 读取对应 `.tasks/08-XX-*.md` 文件中的 review 检查点
2. 逐一对照代码审查
3. 将发现的问题以 markdown checklist 格式（`- [ ] 问题描述`）追加写入对应 Task 文件底部
4. 如果发现**严重安全问题或明显会导致崩溃的 bug**，单独标注 `⚠️ CRITICAL:` 前缀
5. 不需要修改代码，仅记录问题

---

# 本次任务完成记录

## 操作罗列

1. 读取 .project/ 目录下全部文档（Project.md、UserFeatures.md、BusinessFlow.md、Database.md、API.md、Commands.md、Progress.md、DevGuide.md）
2. 读取 src-tauri/src/ 下全部 Rust 源文件（api、database、commands、progress、lib、main）
3. 读取 src/ 下全部 TypeScript/React 源文件（pages、components、utils、types、contexts、样式）
4. 读取已有的 01-07 系列 Task 文档，了解编写格式
5. 编写 08-00 主索引文档
6. 编写 08-01 至 08-10 共 10 个 Review Task 文档

## 流程归纳

1. 通读项目文档建立整体认知
2. 遍历全部源代码文件建立代码-文档映射关系
3. 按模块/页面划分 Review 范围
4. 为每个 Review Task 编写 5-6 个维度的检查要点

## 总结

创建了 08 系列共 11 个 Task 文档，覆盖项目的 API 模块、数据库层、命令层、进度事件系统、Tauri 主配置、打谱查询页面、玩家管理页面、歌单管理页面、通用组件、类型与工具。每个 Task 都包含文档对齐、代码质量、最佳实践、边界情况等维度的具体检查点。
