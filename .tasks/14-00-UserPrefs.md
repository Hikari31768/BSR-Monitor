# 任务 14-00：用户偏好持久化

## 需求

持久化记忆用户的页面操作偏好：

1. **打谱查询页面视图模式**：单谱面模式还是歌单模式
2. **歌单模式下上次选择的歌单**：恢复上次选择
3. **成绩表格排序方式**：谱面排序（asc/desc）、难度排序（asc/desc）
4. **标星玩家**：从 localStorage 迁移到 JSON 文件
5. **音量大小**：全局音量控制

## 存储方案

- **路径**：`app_data_dir() + "/user_prefs.json"`
- **方式**：前端通过 Tauri invoke 调用 Rust command 读写
- **格式**：JSON，snake_case（Rust 侧）/ camelCase（前端侧）

## 数据结构

### Rust (src-tauri/src/commands/common.rs)
```rust
pub struct UserPrefs {
    pub score_table: ScoreTablePrefs,
    pub volume: f32,
    pub starred_player_id: Option<String>,
}

pub struct ScoreTablePrefs {
    pub view_mode: String,
    pub selected_playlist_key: Option<String>,
    pub map_sort: String,
    pub diff_sort: String,
}
```

### TypeScript (src/types/index.ts)
```typescript
interface UserPrefs {
  scoreTable: {
    viewMode: 'single' | 'playlist';
    selectedPlaylistKey: string | null;
    mapSort: 'asc' | 'desc';
    diffSort: 'asc' | 'desc';
  };
  volume: number;
  starredPlayerId: string | null;
}
```

## 实现步骤

1. [x] 创建类型定义和默认值 ([src/types/index.ts](src/types/index.ts))
2. [x] 创建 Rust command ([src-tauri/src/commands/common.rs](src-tauri/src/commands/common.rs))
3. [x] 注册 Rust command ([src-tauri/src/lib.rs](src-tauri/src/lib.rs))
4. [x] 创建偏好管理模块 ([src/utils/prefs.ts](src/utils/prefs.ts))
5. [x] 迁移 Players 页面的 starredPlayerId ([src/pages/Players.tsx](src/pages/Players.tsx))
6. [x] 迁移 ScoreTable 页面的偏好状态 ([src/pages/ScoreTable.tsx](src/pages/ScoreTable.tsx))
7. [x] 迁移 AudioContext 音量控制 ([src/contexts/AudioContext.tsx](src/contexts/AudioContext.tsx))

## 文件变更

| 文件 | 变更 |
|------|------|
| src/types/index.ts | 添加 UserPrefs 类型和默认值 |
| src-tauri/src/commands/common.rs | 添加 UserPrefs 结构体和 load/save 命令 |
| src-tauri/src/lib.rs | 注册 load_user_prefs 和 save_user_prefs 命令 |
| src/utils/prefs.ts | 新建：前端偏好管理模块，处理 camel/snake 转换 |
| src/pages/Players.tsx | 使用新 prefs 系统管理 starredPlayerId |
| src/pages/ScoreTable.tsx | 使用新 prefs 系统管理视图模式、歌单、排序 |
| src/contexts/AudioContext.tsx | 使用新 prefs 系统管理音量 |

## 状态

已完成
