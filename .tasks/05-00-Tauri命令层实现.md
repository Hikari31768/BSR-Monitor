# 最终目标

一个本地运行的应用，用户可以持久性地保存一些 BeatLeader 玩家、歌单，也可以临时性地输入谱面的 key ，然后用户就能查看指定玩家对谱面的游玩情况。

# 本次目标

实现 Tauri Commands 层，将业务逻辑层的功能暴露给前端调用。

# 具体任务

1. 创建 `src-tauri/src/commands` 模块结构
2. 实现玩家管理命令：
   - `add_player` - 添加玩家（含全量拉取成绩）
   - `get_player_list` - 获取玩家列表
   - `get_player` - 获取单个玩家详情
   - `delete_player` - 删除玩家
   - `update_player` - 更新玩家信息（liveURL、alias）
3. 实现成绩管理命令：
   - `update_player_scores` - 增量更新成绩
   - `get_player_scores` - 获取玩家成绩列表
   - `get_player_score_count` - 获取玩家成绩数量
4. 实现谱面管理命令：
   - `get_map_by_key` - 通过 key 获取谱面
   - `get_map_by_hash` - 通过 hash 获取谱面
5. 实现歌单管理命令：
   - `add_playlist` - 添加歌单
   - `get_playlist_list` - 获取歌单列表
   - `get_playlist` - 获取单个歌单详情
   - `delete_playlist` - 删除歌单
   - `get_playlist_maps` - 获取歌单中的谱面列表
6. 实现排行榜查询命令：
   - `get_leaderboard` - 获取排行榜详情
7. 在 `lib.rs` 中注册所有命令
8. 处理数据库连接获取（通过 AppHandle）

# 参考文档

- [项目概览](../.project/Project.md)
- [业务逻辑层](./04-00-业务逻辑层实现.md)
- [Tauri 2 Commands 文档](https://v2.tauri.app/reference/javascript/api/#invoke)

# 设计要点

## 模块结构

```
src-tauri/src/
├── commands/
│   ├── mod.rs       # 模块入口
│   ├── player.rs    # 玩家相关命令
│   ├── score.rs     # 成绩相关命令
│   ├── map.rs       # 谱面相关命令
│   ├── playlist.rs  # 歌单相关命令
│   └── leaderboard.rs # 排行榜命令
```

## 命令设计原则

- 所有命令使用 `#[tauri::command]` 宏标记
- 异步命令返回 `Result<T, String>`（错误转为字符串便于前端处理）
- 通过 `AppHandle` 获取数据库连接
- 数据库操作使用 `Mutex` 保护，每次操作获取锁

## 数据库连接获取

```rust
fn get_db(app: &AppHandle) -> Result<MutexGuard<Connection>, String> {
    let state = app.try_state::<Mutex<Connection>>()
        .ok_or("Database not initialized")?;
    state.lock().map_err(|e| e.to_string())
}
```

## 命令示例

```rust
#[tauri::command]
async fn add_player(
    app: AppHandle,
    id: String,
    live_url: Option<String>,
    alias: Option<String>,
) -> Result<PlayerInfo, String> {
    let db = get_db(&app)?;
    let api = BeatLeaderApi::new();
    
    let params = AddPlayerParams { id, live_url, alias };
    business::player::add_player(&db, &api, params)
        .await
        .map_err(|e| e.to_string())
}
```

## 命令注册

在 `lib.rs` 的 `run()` 函数中：

```rust
.invoke_handler(tauri::generate_handler![
    commands::add_player,
    commands::get_player_list,
    // ...
])
```

# 验收

编写简单的调用测试验证命令可用。

**验收命令**：

```bash
cargo build
```

**验收标准**：

- 所有命令编译通过
- 命令可在前端通过 `invoke` 调用
- 错误正确转换为字符串返回