# 13-00 歌单模块大改 — 统一 .bplist 架构

## Context

当前歌单只有一种来源：BeatSaver 线上歌单（通过输入歌单 ID 添加）。本次改动的目标是引入三种歌单来源的统一架构：

1. **BeatSaver 线上歌单**：输入歌单 ID，通过 `GET /playlists/id/{id}` → `downloadURL` 下载 .bplist
2. **谱师歌单**：输入谱师 ID，通过 `GET /users/id/{id}` → `playlistUrl` 下载 .bplist
3. **本地歌单**：导入 .bplist 文件，复制到 Playlists 目录

三种来源统一通过 .bplist 文件解析，再调用 BeatLeader `GET /leaderboards/hash/{hash}` 获取谱面详细信息（key、星级、封面等）。

## 关键设计决策

- **组合主键**：`source` (beatsaver/mapper/local) + `identifier`（各自不同的标识符）
  - BeatSaver：identifier = 歌单 ID（字符串）
  - Mapper：identifier = 谱师用户 ID
  - Local：identifier = 原始文件的**绝对路径**
- **文件管理**：三种来源的 .bplist 都存储在 `Playlists/` 目录下（与数据库同级）
  - BeatSaver：`Playlists/beatsaver_{id}.bplist`
  - Mapper：`Playlists/mapper_{user_id}.bplist`
  - Local：`Playlists/local_{hash_of_original_path}.bplist`（副本）
- **本地歌单更新**：比对原始路径文件与副本，有变化则更新副本并重新解析
- **已有数据**：清空重来，不做迁移
- **谱面信息**：完全依赖 BeatLeader `GET /leaderboards/hash/{hash}` 获取
  - key 从 `response.song.id` 取（去掉末尾的 x）
  - 封面从 `response.song.coverImage` 取
  - 星级从 `response.leaderboards[].difficulty.stars` 取
- **PlaylistMap 表**：完全移除，歌单中的谱面列表由 .bplist 文件直接定义

## 修改文件清单

### 后端 (Rust)

| 文件 | 改动 |
|------|------|
| `src-tauri/src/database/schema.rs` | 重建 Playlist 表（source + identifier），删除 PlaylistMap 表 |
| `src-tauri/src/database/operations.rs` | PlaylistSource 枚举、更新 Playlist 结构体、新增 BplistFile 解析结构体、重写所有 playlist DB 操作、删除 PlaylistMap 相关操作 |
| `src-tauri/src/database/mod.rs` | 初始化时创建 Playlists 目录 |
| `src-tauri/src/api/beatsaver.rs` | 新增 `get_user_by_id` 接口（获取谱师 playlistUrl） |
| `src-tauri/src/api/beatleader.rs` | 扩展 SongResponse 结构体（id、coverImage 字段），扩展 LeaderboardsByHashResponse |
| `src-tauri/src/commands/playlist.rs` | 重写 add_playlist（source + identifier）、新增 refresh_playlist、统一 bplist 处理流程、删除 PlaylistMap 相关代码 |
| `src-tauri/src/commands/score.rs` | get_playlist_scores 改用 source + identifier，从 .bplist 读取谱面列表 |
| `src-tauri/src/lib.rs` | 更新命令注册 |

### 前端 (TypeScript/React)

| 文件 | 改动 |
|------|------|
| `src/types/index.ts` | 更新 Playlist 接口，新增 PlaylistSource 类型，删除 PlaylistMap 接口 |
| `src/pages/Playlists.tsx` | 添加歌单弹窗改为三种来源选择器 |
| `src/components/PlaylistCard.tsx` | 显示来源标签 |
| `src/components/PlaylistSelect.tsx` | 适配组合主键 |
| `src/pages/ScoreTable.tsx` | 歌单模式下传递 source + identifier |

## 实施步骤

### Step 1: 数据库 Schema 重构

在 `schema.rs` 中：
- DROP 旧 Playlist 和 PlaylistMap 表
- 创建新的 `Playlist` 表：
  ```sql
  CREATE TABLE IF NOT EXISTS Playlist (
      source TEXT NOT NULL CHECK(source IN ('beatsaver', 'mapper', 'local')),
      identifier TEXT NOT NULL,
      name TEXT,
      description TEXT,
      playlistImage TEXT,
      ownerId TEXT,
      ownerName TEXT,
      totalMaps INTEGER,
      createdAt TEXT,
      songsChangedAt TEXT,
      downloadURL TEXT,
      localPath TEXT,
      lastBplistFetchedAt BIGINT,
      PRIMARY KEY (source, identifier)
  )
  ```

### Step 2: Rust 结构体更新

在 `operations.rs` 中：
- 新增 `PlaylistSource` 枚举（beatsaver/mapper/local），实现 rusqlite 的 `ToSql`/`FromSql`
- 更新 `Playlist` 结构体（source + identifier + localPath + lastBplistFetchedAt）
- 新增 .bplist 解析结构体：
  ```rust
  pub struct BplistSong {
      pub key: Option<String>,
      pub song_name: Option<String>,
      pub level_author_name: Option<String>,
      pub hash: String,
      pub levelid: Option<String>,
  }
  pub struct BplistCustomData {
      #[serde(rename = "syncURL")]
      pub sync_url: Option<String>,
  }
  pub struct BplistFile {
      #[serde(rename = "playlistTitle")]
      pub playlist_title: String,
      #[serde(rename = "playlistAuthor")]
      pub playlist_author: String,
      #[serde(rename = "customData")]
      pub custom_data: Option<BplistCustomData>,
      pub songs: Vec<BplistSong>,
      #[serde(default)]
      pub image: Option<String>,
  }
  ```
- 重写所有 playlist DB 操作（insert、get、get_all、delete、get_hashes）
- 删除所有 PlaylistMap 相关操作

### Step 3: BeatLeader API 扩展

在 `beatleader.rs` 中：
- 扩展 `SongResponse` 结构体：
  ```rust
  pub struct SongResponse {
      pub id: String,          // 谱面 key（需去掉末尾 x）
      pub hash: String,
      pub name: String,
      pub author: String,
      pub mapper: String,
      #[serde(rename = "coverImage")]
      pub cover_image: Option<String>,
  }
  ```

### Step 4: BeatSaver API 新增谱师接口

在 `beatsaver.rs` 中：
- 新增 `UserProfileResponse` 结构体：
  ```rust
  pub struct UserProfileResponse {
      pub id: String,
      pub name: String,
      pub avatar: Option<String>,
      #[serde(rename = "playlistUrl")]
      pub playlist_url: Option<String>,
  }
  ```
- 新增 `get_user_by_id(user_id: &str)` 方法

### Step 5: 统一 .bplist 处理流程（核心）

在 `playlist.rs` 中：

**重写 `add_playlist` 命令**：接收 `source` + `identifier`
```rust
#[tauri::command]
pub async fn add_playlist(
    app: AppHandle,
    task_state: State<'_, TaskState>,
    source: String,
    identifier: String,
    task_id: Option<String>,
) -> Result<AddPlaylistResult, String>
```

**三种来源获取 .bplist 内容**：
- **beatsaver**：
  1. `GET /playlists/id/{id}` 获取歌单元信息
  2. 从 `downloadURL` 下载 .bplist 文件
  3. 保存到 `Playlists/beatsaver_{id}.bplist`
  4. 解析 JSON
- **mapper**：
  1. `GET /users/id/{id}` 获取谱师信息
  2. 从 `playlistUrl` 下载 .bplist 文件
  3. 保存到 `Playlists/mapper_{id}.bplist`
  4. 解析 JSON
- **local**：
  1. 读取原始路径文件
  2. 复制到 `Playlists/local_{hash_of_path}.bplist`
  3. 解析 JSON

**实现 `process_bplist()` 统一处理**：
1. 解析 .bplist JSON → `BplistFile`
2. 从 songs 中提取所有 hash
3. 保存歌单元信息到 Playlist 表
4. 后台任务：对每个 hash 调用 BeatLeader API：
   - `GET /leaderboards/hash/{hash}` → 获取 key（去掉末尾 x）、封面、星级
   - 更新 Map 和 MapDiff 表
   - 发射进度事件

**新增 `refresh_playlist` 命令**：
- 读取数据库中的歌单信息
- local 类型：比对原始文件与副本的内容 hash，有变化则更新
- beatsaver/mapper 类型：重新下载 .bplist 并比对内容 hash
- 如有变化，重新解析并更新 Map/MapDiff

### Step 6: 更新其他命令

- `get_playlist_list`：返回更新后的 Playlist 结构
- `get_playlist`：接收 source + identifier
- `delete_playlist`：接收 source + identifier，删除时清理 Playlists 目录下的副本文件
- `get_playlist_hashes`（新命令，替代 `get_playlist_maps`）：返回歌单中所有谱面的 hash 列表（从 .bplist 读取）
- `refresh_playlist_ratings`：接收 source + identifier

### Step 7: 更新 score.rs

`get_playlist_scores`：
- 接收 source + identifier 替代 playlistId
- 从 .bplist 文件读取谱面 hash 列表
- 分页查询每个 hash 的成绩

### Step 8: 前端适配

1. `src/types/index.ts`：
   - 新增 `PlaylistSource = 'beatsaver' | 'mapper' | 'local'`
   - 更新 Playlist 接口（source, identifier, localPath, lastBplistFetchedAt）
   - 删除 PlaylistMap 接口

2. `src/pages/Playlists.tsx`：
   - 添加弹窗改为三种来源选择（tab/按钮切换）
   - BeatSaver：输入歌单 ID
   - Mapper：输入谱师 ID
   - Local：文件选择器（.bplist）

3. `src/components/PlaylistCard.tsx`：
   - 显示来源标签（不同颜色区分）

4. `src/components/PlaylistSelect.tsx`：
   - 选项值改为 `${source}:${identifier}` 格式

5. `src/pages/ScoreTable.tsx`：
   - 歌单模式的状态管理改为 source + identifier

## 验证方式

1. 编译通过：`cargo check` + `npm run tauri dev`
2. 测试 BeatSaver 线上歌单添加、更新
3. 测试谱师歌单添加、更新
4. 测试本地歌单导入、更新（修改原始文件后更新）
5. 测试打谱查询页面歌单模式下的三种来源选择
6. 测试删除歌单时 Playlists 目录下文件的清理
7. 测试进度显示是否正常
