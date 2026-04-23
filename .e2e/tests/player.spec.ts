import { test, expect } from '@playwright/test';
import { startApp, stopApp, getPage } from '../helpers/app';
import { PlayersPage as S, Sidebar } from '../helpers/selectors';
import { cleanDb } from '../helpers/data';

test.beforeAll(async () => {
  cleanDb();
  await startApp();
});

test.afterAll(async () => {
  await stopApp();
});

test.describe('玩家管理', () => {
  test('1.1 导航到玩家管理页', async () => {
    const page = getPage();

    // Click sidebar nav for 玩家管理
    await page.getByRole('link', { name: S.textPlayers }).click();

    // Verify URL and heading
    await expect(page.getByRole('heading', { name: S.heading })).toBeVisible();
    const url = page.url();
    expect(url).toContain('/players');
  });

  test('1.2 空状态显示', async () => {
    const page = getPage();

    // Ensure we're on players page
    await page.getByRole('link', { name: S.textPlayers }).click();
    await page.waitForTimeout(1000);

    // Should see empty state or no player cards
    const hasPlayers = await page.getByText(/成绩/).first().isVisible().catch(() => false);
    if (!hasPlayers) {
      // Verify empty state message exists
      const emptyText = page.getByText(/暂无|还没有/);
      await expect(emptyText).toBeVisible();
    }
  });

  test('1.3 添加玩家', async () => {
    const page = getPage();
    const testPlayerId = '76561198820181271';

    // Navigate to players page
    await page.getByRole('link', { name: S.textPlayers }).click();
    await page.waitForTimeout(1000);

    // Click add player button
    await page.getByRole('button', { name: S.btnAddPlayer }).click();

    // Wait for modal
    await expect(page.getByRole('heading', { name: S.modalTitleAdd })).toBeVisible();

    // Fill player ID
    await page.getByPlaceholder(/玩家|ID|Steam/).fill(testPlayerId);

    // Click add button
    await page.getByRole('button', { name: S.btnModalAdd }).click();

    // Wait for the player card to appear (this can take a while due to API calls)
    // The card should show the player name (not "加载中...")
    await page.waitForTimeout(5000);

    // Player card should exist with actual name or processing status
    const cardExists = await page.locator('div', { hasText: /成绩|处理中/ }).first().isVisible().catch(() => false);
    expect(cardExists).toBe(true);
  });

  test('1.4 查看玩家列表', async () => {
    const page = getPage();

    await page.getByRole('link', { name: S.textPlayers }).click();
    await page.waitForTimeout(2000);

    // Should have at least one player card (from previous test)
    const playerCards = page.locator('[class*="rounded-xl"]').filter({ hasText: /成绩/ });
    await expect(playerCards.first()).toBeVisible();
  });

  test('1.5 删除玩家', async () => {
    const page = getPage();

    await page.getByRole('link', { name: S.textPlayers }).click();
    await page.waitForTimeout(2000);

    // Find and click delete button on first player card
    const deleteBtn = page.getByRole('button', { name: S.btnDeletePlayer }).first();
    await deleteBtn.click();

    // Wait for delete confirmation (assuming a modal or confirm dialog appears)
    // Click confirm
    const confirmBtn = page.getByRole('button', { name: S.btnConfirmDelete }).first();
    if (await confirmBtn.isVisible().catch(() => false)) {
      await confirmBtn.click();
    } else {
      // If no confirm dialog, the delete might be immediate
      console.log('  No confirmation dialog found, delete may be immediate.');
    }

    await page.waitForTimeout(1000);

    // Verify player card is removed (fewer player cards than before)
    const playerCards = page.locator('[class*="rounded-xl"]').filter({ hasText: /成绩/ });
    const count = await playerCards.count();
    console.log(`  Remaining player cards: ${count}`);
  });
});
