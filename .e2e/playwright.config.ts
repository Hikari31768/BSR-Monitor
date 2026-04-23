import { defineConfig } from '@playwright/test';
import path from 'path';

export default defineConfig({
  testDir: './tests',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: 1,
  workers: 1,
  reporter: process.env.VISUAL_TEST
    ? [['html', { outputFolder: '../.e2e-results' }], ['list']]
    : 'list',
  timeout: 120_000,
  use: {
    // CDP 连接配置
    connectOverCDP: {
      endpointURL: 'http://localhost:9222',
    },
    // 截图和视频（可选）
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    trace: 'on-first-retry',
  },
  outputDir: path.resolve(__dirname, '.test-results'),
});
