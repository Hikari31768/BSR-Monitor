# 进度事件系统

为耗时操作（添加玩家/歌单）提供实时进度反馈，避免前端"卡死"状态。

## 模块结构

```
src-tauri/src/
└── progress.rs     # 进度事件定义与发射
```

## 数据结构

`ProgressPayload` 包含以下字段：

| 字段 | 类型 | 说明 |
|------|------|------|
| `task_type` | String | `"add_player"` 或 `"add_playlist"` |
| `task_id` | String | UUID，区分不同任务实例 |
| `stage` | String | 当前阶段标识 |
| `progress` | f64 | 0.0 - 100.0 百分比 |
| `total` | Option<u64> | 总数（如总页数） |
| `current` | Option<u64> | 当前完成数 |
| `message` | Option<String> | 状态消息 |
| `player_name` | Option<String> | 玩家昵称（add_player 时用） |
| `player_avatar_url` | Option<String> | 玩家头像 URL（add_player 时用） |

### Builder 方法

```rust
ProgressPayload::new(task_type, task_id)
    .with_progress(stage, progress)  // 设置阶段和进度
    .with_count(stage, current, total)  // 设置计数并自动计算进度
    .with_message(message)            // 状态消息
    .with_player_info(name, avatar)   // 玩家信息（前端先行显示）
```

## 事件发射

```rust
emit_progress_event(&app, &payload)?;
```

底层使用 Tauri 的 `app.emit("progress-update", payload)`。

## 进度阶段

### add_player

| 进度 | 阶段 | 说明 |
|------|------|------|
| 10% | `fetching_player_info` | 正在获取玩家信息 |
| 20% | `saving_player_info` | 已获取玩家（含昵称、头像） |
| 30%-100% | `fetching_scores` | 拉取成绩，按页递增 |
| 100% | `completed` | 完成 |

### add_playlist

| 进度 | 阶段 | 说明 |
|------|------|------|
| 5% | `fetching_playlist` | 正在获取歌单信息 |
| 10% | `saving_playlist_info` | 已获取歌单 |
| 10%-100% | `processing_page` / `processing_maps` | 处理谱面页面 |
| 100% | `completed` | 完成 |

## 前端订阅

```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen('progress-update', (event) => {
  const { task_type, task_id, progress, message, player_name, player_avatar_url } = event.payload;
  if (task_type === 'add_player') {
    // 更新对应用户的进度显示
    // player_name / player_avatar_url 可用于先行显示
  }
});

// invoke 完成后清理
unlisten();
```
