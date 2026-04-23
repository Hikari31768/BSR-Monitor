# BSR Monitor E2E 自动化回归测试

## 目录结构

```
.e2e/
├── playwright.config.ts       # Playwright 配置
├── README.md                  # 本文档
├── helpers/
│   ├── app.ts                 # 应用启动/停止、CDP 连接
│   ├── data.ts                # 数据库备份/恢复
│   ├── selectors.ts           # 页面元素选择器常量
│   └── visual-test.ts         # 视觉测试（截图 + LLM 断言）
├── tests/
│   ├── player.spec.ts         # 玩家管理 E2E 测试
│   ├── score.spec.ts          # 打谱查询 E2E 测试
│   └── playlist.spec.ts       # 歌单管理 E2E 测试
├── fixtures/                  # 测试数据库备份
└── screenshots/               # 视觉测试截图（自动生成）
```

## 前置条件

1. 确保项目已安装依赖：`npm install`
2. 确保 Tauri 开发环境可用（Rust toolchain、WebView2）

## 运行测试

### 基本测试（无视觉断言）

```bash
npm run test:e2e
```

### 带视觉评判的测试

```bash
npm run test:e2e:visual
```

需要设置 `ANTHROPIC_API_KEY` 环境变量来启用 LLM 视觉断言。

### 可视化调试模式（带浏览器界面）

```bash
npm run test:e2e:headed
```

### Playwright UI 交互模式

```bash
npm run test:e2e:ui
```

## 测试数据

| 类型 | 测试数据 | 说明 |
|------|----------|------|
| 玩家 | `76561198820181271`, `76561198397884994` | 真实玩家 ID |
| 歌单 | `864507`, `859014` | 真实歌单 ID |
| 谱面 | `40dd1` | 单个谱面 key |

## 工作原理

1. 测试启动时自动编译/启动 Tauri 应用（带 CDP 端口 9222）
2. Playwright 通过 CDP 连接到 WebView2
3. 执行测试用例（模拟用户操作）
4. 测试结束后关闭应用

## 注意事项

- **CDP 连接**: 首次启动可能需要等待 CDP 端口就绪
- **测试时间**: 真实 API 调用较慢，超时设置为 120s
- **数据库**: 测试使用独立数据库，避免污染主数据
- **外部 API**: BeatSaver/BeatLeader 不可用时测试可能失败
