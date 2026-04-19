---
name: task-doc-verification
description: 任务文档必须包含验收章节
type: feedback
---

任务文档必须包含验收章节。

**Why:** 任务完成需要有明确的验收标准和方法，确保实现符合预期。

**How to apply:** 在 `.tasks/xx-xx-*.md` 任务文档末尾，完成记录之后添加 `# 验收` 章节，包含：
- 验收方法（测试命令、手动验证步骤等）
- 验收结果（测试输出、验证截图等）
- 验收结论（通过/不通过）