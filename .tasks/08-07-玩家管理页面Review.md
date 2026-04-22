# 08-07 - 玩家管理页面 Review

## Review 范围

页面文件：
- [src/pages/Players.tsx](src/pages/Players.tsx)

相关组件：
- [src/components/PlayerCard.tsx](src/components/PlayerCard.tsx)

## Review 维度

### 1. 文档对齐

- 对照 [.project/UserFeatures.md](.project/UserFeatures.md) 玩家管理功能检查：
  - 卡片网格展示玩家，显示分数数量 — 是否实现
  - 添加玩家（ID + 可选直播地址/备注名），实时进度追踪 — 是否实现
  - 增量更新分数，支持删除 — 是否实现

### 2. React 最佳实践

- `players` 状态类型为 `(PlayerWithScoreCount & { loading?: boolean; progress?: number; status_message?: string })[]` — 交叉类型扩展，是否应定义为独立接口
- `handleAddPlayer` 中先检查 `players.findIndex(p => p.id === addForm.id)` 来判断是否已存在 — 但 `addForm.id` 在用户手动输入时可能与数据库中已有的不同玩家 ID 相同，此时 `existingIdx` 不为 -1，但 `players` 中对应条目的 `loading` 状态可能不是由当前操作引起的
- `loadPlayers` 中使用 `Promise.all` 为每个玩家调用 `get_player_score_count` — 当玩家数量较多时会产生 N+1 次 invoke 调用，是否应后端提供批量接口

### 3. 状态管理

- 进度事件监听器在 `handleAddPlayer` 内部创建，通过闭包捕获 `addForm.id` — 如果用户快速连续添加两个不同玩家，`addForm.id` 会被第二个覆盖，第一个的进度更新可能会应用到错误的玩家上
- `handleDeletePlayer` 中检查 `player.loading` 来阻止删除 — 但如果页面重新加载，`loading` 状态会丢失，正在后台进行的任务无法被追踪
- `handleUpdateScores` 使用 `alert()` 显示更新结果 — 是否应使用 toast 或其他非阻塞提示

### 4. 组件设计

- `PlayerCard` 接收 `player` 和 `scoreCount` 作为独立 props — 但 `player` 类型中可能已经包含 `score_count`（如果扩展了 `PlayerWithScoreCount`），是否应统一
- `PlayerCard` 中 difficulty 颜色映射逻辑与 `DiffBadge` 和 `ScoreRow` 中的重复 — 是否应使用共享配置
- 进度条使用 `player.progress || 0` — 当 progress 为 `undefined` 时显示 0%，是否合理

### 5. 边界情况

- 添加玩家时如果输入的 ID 格式不合法（非数字），前端未做校验就直接发送到后端
- `handleAddPlayer` 中 `addForm.id` 为空时被 `if (!addForm.id) return` 拦截 — 但按钮没有 disabled 状态，用户可能多次点击
- 删除确认弹窗中使用 `showDeleteModal !== null` 来判断是否打开 — 但 `showDeleteModal` 类型是 `string | null`，当玩家 ID 恰好为空字符串时会怎样（实际上 Steam ID 不会是空字符串，但类型系统不保证）

### 6. 用户体验

- 添加玩家弹窗中字段没有表单验证提示
- 玩家卡片上的「更新」按钮点击后只有 loading 状态（`updatingId`），但没有进度条 — 与添加/更新歌单的体验不一致
- 玩家列表中处理中的玩家使用黄色主题 — 是否与整体设计语言一致
