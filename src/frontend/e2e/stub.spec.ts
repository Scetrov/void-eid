import { test, expect } from './fixtures';

test.describe('Stub API Integration', () => {
  test('should login via stub and view roster', async ({ adminPage: page }) => {
    // Already logged in as admin via fixture

    // Navigate to Roster
    await page.goto('/roster');

    // Verify Content from Stub DB
    // AdminUser should be visible
    await expect(page.getByText('AdminUser')).toBeVisible();
    // RegularUser should be visible
    await expect(page.getByText('RegularUser')).toBeVisible();
    // Admin Wallet Address (truncated with slice(0,6)...slice(-4))
    await expect(page.getByText('0xadmi...6789')).toBeVisible();
  });
});
