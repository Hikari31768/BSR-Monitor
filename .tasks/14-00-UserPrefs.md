# 任务 14-00：用户偏好持久化

## 需求

持久化记忆用户的页面操作偏好：

1. **打谱查询页面视图模式**：单谱面模式还是歌单模式
2. **歌单模式下上次选择的歌单**：恢复上次选择
3. **成绩表格排序方式**：谱面排序（asc/desc）、难度排序（asc/desc）
4. **标星玩家**：从 localStorage 迁移到 JSON 文件
5. **音量大小**：全局音量控制
6. **打谱查询页面已选玩家**：记忆表格中显示的玩家列表

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
    pub selected_player_ids: Vec<String>,  // 打谱查询页已选玩家
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
    selectedPlayerIds: string[];  // 打谱查询页已选玩家
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
| src/pages/ScoreTable.tsx | 使用新 prefs 系统管理视图模式、歌单、排序、玩家列表 |
| src/contexts/AudioContext.tsx | 使用新 prefs 系统管理音量 |

## 修复：避免视图模式切换抽动

**问题**：当记忆的是歌单模式时，进入打谱查询页会先显示"单谱面"Tab点亮，然后迅速切换到"歌单"。

**原因**：`useState('single')` 初始值硬编码为 'single'，然后异步加载偏好后才切换到正确的模式。

**修复** ([src/pages/ScoreTable.tsx](src/pages/ScoreTable.tsx))：
- `viewMode` 初始值改为 `null`，表示尚未加载
- 偏好加载完成后一次性设置正确的 `viewMode`
- 加载期间显示骨架屏（animate-pulse），避免任何 Tab 处于激活状态

## 修复：切换歌单时不重置排序

**问题**：切换歌单、切换页面、重启程序后，谱面排序（mapSort）被重置为 'asc'，持久化偏好未能生效。

**原因**：切换歌单的 effect 中调用了 `setMapSort('asc')`，导致每次切换歌单都强制重置排序。

**修复** ([src/pages/ScoreTable.tsx:306-310](src/pages/ScoreTable.tsx#L306-L310))：
- 移除 `setMapSort('asc')` 调用
- 切换歌单时只重置页码和数据，保留用户的排序偏好

## 新增：打谱查询页面玩家列表持久化

**需求**：记忆打谱查询页面表格中已选的玩家列表。

**实现** ([src/pages/ScoreTable.tsx](src/pages/ScoreTable.tsx))：
- 添加 `selectedPlayerIds` 到 UserPrefs 类型
- 玩家选择变化时自动保存 ID 列表
- 加载页面时按保存的顺序恢复玩家选择
- 优先恢复保存的玩家列表，无保存时回退到标星玩家或第一个玩家

**文件变更**：
- `src/types/index.ts`: 添加 `selectedPlayerIds: string[]` 字段
- `src-tauri/src/commands/common.rs`: 添加 `selected_player_ids: Vec<String>` 字段
- `src/utils/prefs.ts`: 更新转换逻辑
- `src/pages/ScoreTable.tsx`: 
  - 添加 `savedPlayerIdsRef` 存储加载的玩家ID
  - 修改 `loadPlayers` 优先恢复保存的玩家列表
  - 保存 effect 中添加 `selectedPlayers` 依赖，自动持久化

## 状态

已完成
