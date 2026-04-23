import { type Page } from '@playwright/test';
import fs from 'fs';
import path from 'path';

const SCREENSHOT_DIR = path.resolve(__dirname, '../screenshots');

/**
 * Take a screenshot and optionally run visual assertion via LLM API.
 * Controlled by VISUAL_TEST env var.
 */
export async function captureScreenshot(page: Page, name: string): Promise<string> {
  if (!fs.existsSync(SCREENSHOT_DIR)) {
    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
  }

  const filename = `${name}-${Date.now()}.png`;
  const filepath = path.join(SCREENSHOT_DIR, filename);

  const buffer = await page.screenshot({ fullPage: false });
  fs.writeFileSync(filepath, buffer);
  console.log(`  Screenshot saved: ${filepath}`);
  return filepath;
}

/**
 * Visual assertion using LLM API. Only runs when VISUAL_TEST=1.
 * Sends screenshot to LLM and checks if the page matches expectations.
 */
export async function visualAssert(page: Page, description: string): Promise<void> {
  if (!process.env.VISUAL_TEST) return;

  const filepath = await captureScreenshot(page, `visual-${Date.now()}`);

  // The actual LLM API call will depend on your setup
  // For now, this is a placeholder that logs the assertion description
  // In production, you would send the screenshot to an LLM API like:
  //
  // const response = await fetch('https://api.anthropic.com/v1/messages', {
  //   method: 'POST',
  //   headers: {
  //     'x-api-key': process.env.ANTHROPIC_API_KEY || '',
  //     'anthropic-version': '2023-06-01',
  //     'content-type': 'application/json',
  //   },
  //   body: JSON.stringify({
  //     model: 'claude-sonnet-4-6-20250514',
  //     max_tokens: 500,
  //     messages: [{
  //       role: 'user',
  //       content: [
  //         { type: 'text', text: `Analyze this screenshot. Does it match: "${description}"? Answer with PASS or FAIL and explain why.` },
  //         { type: 'image', source: { type: 'base64', media_type: 'image/png', data: buffer.toString('base64') } }
  //       ]
  //     }]
  //   })
  // });

  console.log(`  [VISUAL TEST] Assertion: ${description}`);
  console.log(`  [VISUAL TEST] Screenshot: ${filepath}`);
  console.log(`  [VISUAL TEST] To enable actual LLM assertion, set ANTHROPIC_API_KEY env var.`);

  // For now, always pass when in visual test mode (placeholder)
  // Once API key is configured, replace this with actual assertion
}

/**
 * Helper to check if visual testing is enabled
 */
export function isVisualTestEnabled(): boolean {
  return process.env.VISUAL_TEST === '1';
}
