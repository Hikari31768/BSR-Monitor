# 08-01 - API 模块 Review

## Review 范围

`src-tauri/src/api/` 目录下所有文件：
- [mod.rs](src-tauri/src/api/mod.rs)
- [client.rs](src-tauri/src/api/client.rs)
- [error.rs](src-tauri/src/api/error.rs)
- [types.rs](src-tauri/src/api/types.rs)
- [beatleader.rs](src-tauri/src/api/beatleader.rs)
- [beatsaver.rs](src-tauri/src/api/beatsaver.rs)
- [tests.rs](src-tauri/src/api/tests.rs)

## Review 维度

### 1. 文档对齐

- 对照 [.project/docs/API.md](.project/docs/API.md) 检查 API 封装是否一致
- 检查数据结构是否与文档中的字段映射一致
- 验证重试机制（最大3次、超时1s、429限流5s）是否按文档实现

### 2. 代码质量（Rust Best Practices）

- 错误枚举 `ApiError` 是否完整覆盖边界情况
- `ApiClient` 的 `request_with_retry` 泛型闭包设计是否合理
- `BeatLeaderApi` 和 `BeatSaverApi` 中重复的 `handle_response` 方法是否可以提取
- `new()` 和 `with_client()` 构造模式是否一致
- `Default` trait 实现是否有必要

### 3. 异步模式（Rust Async Patterns）

- `request_with_retry` 中的 `Pin<Box<dyn Future>>` 使用是否最优
- 重试逻辑中 `attempts` 计数是否准确（初始为0，每次失败+1，与 `max_retries` 比较的时机）
- 超时和429重试之间的 `attempts` 共享是否会导致过早放弃

### 4. 边界情况

- `ApiClient::new()` 中使用 `.expect()` 处理 HTTP 客户端创建失败 — 在 WebView2 环境中是否安全
- `handle_response` 中 `response.text().await.unwrap_or_default()` 在 body 已被消费时的行为
- API 响应结构中 Optional 字段是否都正确标记了 `#[serde(default)]`
- `CompactScore` 中 `base_score` 和 `modified_score` 字段有 `#[serde(default)]` 但从未使用

### 5. 类型安全（TypeScript Advanced Types）

- Rust 结构体与前端 TypeScript 类型的对应关系是否一致
- `serde` 序列化后的字段名是否与前端期望的匹配（snake_case vs camelCase）

### 6. 测试

- [tests.rs](src-tauri/src/api/tests.rs) 中的测试是集成测试（真实网络请求），是否需要 mock 层
- 测试用例是否覆盖了错误路径（404、超时、限流）

---

## 发现的问题

### 1. 文档对齐

- [ ] `CompactScore` 结构体中 `base_score` 和 `modified_score` 字段在 `.project/docs/API.md` 文档中未定义，代码中存在但从未被业务层使用，属于冗余字段
- [ ] `PlayerResponse.country` 字段类型为 `String` + `#[serde(default)]`，文档中未标注是否为可选字段；若 API 可能不返回该字段，`default` 会填入空字符串，建议改为 `Option<String>`
- [ ] 文档中的难度值映射（1→Easy, 3→Normal, 5→Hard, 7→Expert, 9→ExpertPlus）仅在文档中说明，代码中 `CompactLeaderboard.difficulty` 为裸 `i32`，未使用类型安全的枚举来约束

### 2. 重试机制

- [ ] ⚠️ CRITICAL: `request_with_retry` 中超时和 429 限流共用同一个 `attempts` 计数器，若一次请求超时（attempts=1）后下一次遇到 429（attempts=2），第三次再 429 就会直接返回 `RateLimited`，实际仅重试了 2 次而非文档承诺的 3 次
- [ ] 429 重试时硬编码 `ApiError::RateLimited(60)`，未读取 HTTP `Retry-After` 响应头的值，实际延迟可能与 60 秒不符
- [ ] `request_with_retry` 的重试循环中，初始 `attempts = 0`，当 `attempts >= max_retries` 时才退出，实际重试次数为 `max_retries` 次（正确），但首次请求失败后才开始计数，逻辑可读性差，建议改为 `for attempt in 0..=max_retries` 模式

### 3. 代码质量（Rust Best Practices）

- [ ] ⚠️ CRITICAL: `BeatLeaderApi` 和 `BeatSaverApi` 中的 `handle_response` 方法完全重复（位于 beatleader.rs:92-102 和 beatsaver.rs:152-162），应提取到共享模块（如 `client.rs` 或新建 `utils.rs`）
- [ ] `ApiClient::new()` 使用 `.expect("Failed to create HTTP client")` 处理 HTTP 客户端创建失败 -- 在正常环境中 reqwest::Client::builder().build() 几乎不会失败，但若有自定义 CA 证书或系统网络栈异常会导致 panic。建议改为 `Result` 返回或 `unwrap_or_else(|_| Client::new())` 降级
- [ ] `handle_response` 中 `response.text().await.unwrap_or_default()` 在 body 已被消费时返回空字符串，但 `response.json::<T>().await` 也会消费 body -- 由于 `.text()` 和 `.json()` 是互斥分支，不存在双重消费问题，但 `unwrap_or_default()` 会吞掉 body 读取失败的错误，建议用 `map_err` 记录
- [ ] `CompactScore` 中 `base_score` 和 `modified_score` 有 `#[serde(default)]` 但从未被业务层使用，若 BeatLeader API 已不再返回这些字段，应考虑移除
- [ ] 每个 API 方法（`get_map_by_key`、`get_map_by_hash`、`get_player` 等）的请求构造逻辑高度重复，可提取为通用的 `execute_request` 辅助方法
- [ ] `format!("...").as_str()` 模式在 `bl_url`/`beatsaver_url` 调用中存在不必要的临时 String 分配，可直接接受 `impl AsRef<str>` 或使用 `format!` 内联

### 4. 异步模式

- [ ] `request_with_retry` 使用 `Pin<Box<dyn Future + Send>>` 泛型闭包，每次请求都需要堆分配和动态分发。对于本项目（请求量有限）可接受，但若追求性能可考虑使用 `async-trait` 或宏来避免闭包分配
- [ ] 闭包内 `let client = self.client.client().clone()` 克隆 `reqwest::Client`（内部为 Arc 引用计数），开销极小，无问题

### 5. 类型安全

- [ ] ⚠️ CRITICAL: BeatLeader 响应结构体（`PlayerResponse`、`CompactScore`、`CompactLeaderboard` 等）大多数必填字段缺少 `#[serde(default)]`，若 API 返回字段缺失会导致整个反序列化失败。相比之下，BeatSaver 结构体更防御性地使用了 `#[serde(default)]`
- [ ] ⚠️ CRITICAL: Rust 端 `LeaderboardResponse` 结构与前端 TypeScript 的 `LeaderboardResponse` 接口完全不匹配。Rust 端返回 `{id, song: {hash, name, author, mapper}, difficulty: {value, mode, mode_name, difficulty_name}}`，前端期望 `{leaderboard: {id, song: {name, author, coverImage}, difficulty: {name, value}, mode, modeName}, scores: [...]}`。当前通过 commands 层的数据库转换间接桥接，但如果直接暴露给前端会产生问题
- [ ] `Metadata` 结构体（types.rs）所有字段均为必填，无 `#[serde(default)]`，若 BeatLeader API 分页响应缺少 `itemsPerPage`、`page` 或 `total` 中任何一个，反序列化会失败
- [ ] `PlaylistType` 使用 `#[serde(rename_all = "PascalCase")]` 序列化，但 BeatSaver API 实际返回的是小写字符串（如 `"private"`, `"public"`），应改为 `#[serde(rename_all = "lowercase")]` 或通过实际 API 响应验证
- [ ] 前端 TypeScript 类型中的 `PlayerInfo.avatar_url` 对应 Rust `PlayerResponse.avatar`，字段名不一致，需通过 commands 层的转换桥接（当前 `commands/player.rs` 已处理，但增加了维护负担）

### 6. 边界情况

- [ ] ⚠️ CRITICAL: `ApiClient::new()` 中 `.expect()` 会导致进程 panic，Tauri WebView2 环境下如果系统网络栈初始化延迟或失败，应用启动即崩溃。建议将 API 客户端创建延迟到首次使用或返回 Result
- [ ] `get_player_scores` 使用 POST 方法 + query parameters 传递 `page`、`count`、`sortBy`，虽然 BeatLeader API 的 `/player/{id}/scores/compact` 确实使用 POST，但通常 RESTful 设计中分页参数应在 body 中。需确认 BeatLeader API 文档是否确实要求 query params
- [ ] URL 路径参数（`player_id`、`leaderboard_id`、`hash`、`key`）未做 URL 编码，若包含特殊字符（如 `/`、`?`）会导致请求错误。当前场景下 Steam ID 和 hash 不太可能出现特殊字符，但缺乏防御性处理
- [ ] `CompactLeaderboard.difficulty` 为 `i32` 类型，无范围校验，若 API 返回未知难度值（如 2、4、6），业务层 `difficulty_from_value` 函数会返回空字符串，静默失败

### 7. 测试

- [ ] ⚠️ CRITICAL: `tests.rs` 中所有测试均为真实网络请求的集成测试，无 mock 层，在无网络环境下或 CI 中无法运行
- [ ] 测试未覆盖错误路径：无 404、超时、限流、JSON 解析失败等场景的测试用例
- [ ] `tests.rs` 缺少 `#[cfg(test)]` 属性保护（虽然 `mod.rs` 中已用 `#[cfg(test)] mod tests` 包裹，但文件顶部无属性，单独编译时会被包含）
- [ ] `test_beatsaver_get_playlist` 使用 `playlist_id = 123` 作为示例，注释说"可能不存在"，这种非确定性测试在 CI 中不稳定
- [ ] 测试仅覆盖正常路径，未测试 `request_with_retry` 的重试逻辑是否真正生效
