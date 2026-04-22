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

- 对照 [.project/API.md](.project/API.md) 检查 API 封装是否一致
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
