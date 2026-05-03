# 任务 14-04：玩家 ID 规范化

## 需求

当前添加玩家时直接使用用户输入的 ID（可能是自定义 ID 如 "yyfy"）作为主键。

问题：
- 自定义 ID 可以被修改，导致数据不一致
- 应该使用 BeatLeader 返回的原始数字 ID 作为系统主键

## 解决方案

1. **API 调用**：允许使用自定义 ID（如 yyfy）调用 `/player/{id}`
2. **提取原始 ID**：从响应体 `id` 字段获取原始数字 ID
3. **重复检查**：使用原始数字 ID 检查是否已存在
4. **数据库存储**：以原始数字 ID 为主键存储玩家信息

## 实现步骤

- [x] 修改 `add_player` command：
  - 先调用 API 获取玩家信息
  - 从响应中提取 `player_response.id` 作为原始数字 ID
  - 使用原始 ID 检查是否已存在
  - 调用 `fetch_and_save_player` 时传入原始 ID

- [x] 修改 `fetch_and_save_player` 函数签名：
  - 新增 `player_response: PlayerResponse` 参数
  - 直接使用传入的 player_response 构建 PlayerInfo
  - 移除内部 API 调用逻辑

- [x] 修改 `refresh_player_scores`：
  - 先调用 API 获取玩家信息
  - 提取原始 ID
  - 使用原始 ID 调用 `fetch_and_save_player`

- [x] 修改 `lib.rs` startup 刷新逻辑：
  - 对每个玩家先调用 API 获取信息
  - 提取原始 ID 传递给 `fetch_and_save_player`

## 变更文件

| 文件 | 变更 |
|------|------|
| src-tauri/src/commands/player.rs | 修改 `add_player` 和 `refresh_player_scores`，更新 `fetch_and_save_player` 签名 |
| src-tauri/src/lib.rs | 修改 startup 刷新逻辑，添加 API 调用 |

## 效果

- 用户输入 "yyfy" → API 返回原始 ID "76561199000000000"
- 系统内部使用 "76561199000000000" 作为主键
- 即使自定义 ID 被修改，数据关联仍然正确

## 状态

已完成
