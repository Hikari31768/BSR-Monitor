import { chromium, type Browser, type Page } from '@playwright/test';
import { spawn, type ChildProcess } from 'child_process';
import path from 'path';
import http from 'http';

const CDP_PORT = 9222;
const CDP_URL = `http://localhost:${CDP_PORT}`;
const MAX_RETRIES = 30;
const RETRY_INTERVAL = 2000;

let appProcess: ChildProcess | null = null;
let browser: Browser | null = null;
let page: Page | null = null;

/**
 * Wait for CDP endpoint to be available
 */
async function waitForCDP(): Promise<boolean> {
  for (let i = 0; i < MAX_RETRIES; i++) {
    try {
      const res = await fetch(`${CDP_URL}/json/version`);
      if (res.ok) return true;
    } catch {
      // CDP not ready yet
    }
    console.log(`  Waiting for CDP... (${i + 1}/${MAX_RETRIES})`);
    await new Promise((r) => setTimeout(r, RETRY_INTERVAL));
  }
  return false;
}

/**
 * Start the Tauri application with CDP enabled
 */
export async function startApp(): Promise<void> {
  console.log('Starting Tauri application...');

  // Set environment variable for CDP
  process.env.WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = `--remote-debugging-port=${CDP_PORT}`;

  // Determine if we should run dev or use pre-built binary
  const useDev = process.env.E2E_MODE !== 'build';

  if (useDev) {
    console.log('  Running in dev mode (npm run tauri dev)...');
    appProcess = spawn('npm', ['run', 'tauri', 'dev'], {
      stdio: 'inherit',
      shell: true,
      env: {
        ...process.env,
        WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${CDP_PORT}`,
      },
    });
  } else {
    // For production build mode, user should have the binary ready
    const binaryPath = path.resolve(
      __dirname,
      '../src-tauri/target/release/bsr-monitor.exe'
    );
    console.log(`  Launching: ${binaryPath}`);
    appProcess = spawn(binaryPath, [], {
      stdio: 'inherit',
      env: {
        ...process.env,
        WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${CDP_PORT}`,
      },
    });
  }

  // Wait for CDP to be ready
  const cdpReady = await waitForCDP();
  if (!cdpReady) {
    throw new Error('CDP endpoint not available after timeout');
  }

  console.log('  CDP endpoint available.');

  // Connect to CDP
  browser = await chromium.connectOverCDP(CDP_URL);
  const contexts = browser.contexts();
  const context = contexts[0] || (await browser.newContext());
  const pages = context.pages();
  page = pages[0] || (await context.newPage());

  // Wait for the page to be loaded (check for React app root)
  await page.waitForSelector('#root', { timeout: 30_000 });
  console.log('  Application loaded.');
}

/**
 * Get the current page instance
 */
export function getPage(): Page {
  if (!page) throw new Error('App not started. Call startApp() first.');
  return page;
}

/**
 * Stop the Tauri application and cleanup
 */
export async function stopApp(): Promise<void> {
  console.log('Stopping Tauri application...');

  if (browser) {
    await browser.close().catch(() => {});
    browser = null;
    page = null;
  }

  if (appProcess) {
    // Graceful shutdown
    appProcess.kill('SIGTERM');

    // Force kill after timeout if still running
    setTimeout(() => {
      if (appProcess && !appProcess.killed) {
        appProcess.kill('SIGKILL');
      }
    }, 5000);

    appProcess = null;
  }

  console.log('  Application stopped.');
}

/**
 * Navigate to a route in the app
 */
export async function navigateTo(route: string): Promise<void> {
  const p = getPage();
  // For Tauri apps, navigation is done via sidebar clicks
  // This helper is for convenience - actual navigation depends on UI
  const url = p.url();
  if (!url.endsWith(route)) {
    // Try direct navigation (works in dev mode with vite)
    try {
      await p.goto(`http://localhost:1420/${route.replace(/^\//, '')}`, {
        waitUntil: 'networkidle',
        timeout: 10_000,
      });
    } catch {
      // Fallback: try clicking sidebar
      console.log(`  Direct navigation failed, trying sidebar click for: ${route}`);
    }
  }
}
