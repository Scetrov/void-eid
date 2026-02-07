import { test, expect } from './fixtures';

test.describe('Roster Page', () => {
  test('should display roster table for admin', async ({ adminPage: page }) => {
    await page.goto('/roster');
    await page.waitForSelector('table');

    // Expect to see title/nav
    await expect(page.getByRole('heading', { name: 'Roster' })).toBeVisible();

    // Expect items from stub DB
    await expect(page.getByText('AdminUser')).toBeVisible();
    await expect(page.getByText('RegularUser')).toBeVisible();
    await expect(page.getByText('0xadmi...6789')).toBeVisible();
  });

  test('should filter roster by search', async ({ adminPage: page }) => {
      await page.goto('/roster');

      // Wait for table to load
      await page.waitForSelector('table');

      const searchBox = page.getByPlaceholder('Search by username');
      await searchBox.fill('Admin');

      // Wait for the search to complete
      await page.waitForResponse(resp => resp.url().includes('search=Admin'));

      // RegularUser should disappear
      await expect(page.getByText('RegularUser')).not.toBeVisible();
      // AdminUser should remain visible
      await expect(page.getByText('AdminUser')).toBeVisible();
  });
});
