import { test, expect } from '@playwright/test';

test.describe('Roster Page', () => {
  // Mock User Data
  const mockUser = {
    id: "admin-id",
    discordId: "123",
    username: "AdminUser",
    discriminator: "0000",
    avatar: null,
    tribe: "Fire",
    isAdmin: true,
    wallets: []
  };

  const mockRoster = [
    { discord_id: "1", username: "Alice", avatar: null, wallets: ["0x1234567890abcdef1234567890abcdef56780000"] },
    { discord_id: "2", username: "Bob", avatar: null, wallets: [] }
  ];

  test.beforeEach(async ({ page }) => {
    // Mock /api/me to return admin user
    await page.route('http://localhost:5038/api/me', async route => {
        // Only return admin if Authorization header is present (simulated)
        const headers = route.request().headers();
        if (headers['authorization']) {
             await route.fulfill({ json: mockUser });
        } else {
             await route.fulfill({ status: 401 });
        }
    });

    // Mock /api/roster
    await page.route('http://localhost:5038/api/roster*', async route => {
        await route.fulfill({ json: mockRoster });
    });

    // Mock Login (simulate setting token loalstorage)
    await page.goto('/');
    await page.evaluate(() => {
        localStorage.setItem('sui_jwt', 'fake-token');
    });
  });

  test('should display roster table for admin', async ({ page }) => {
    await page.goto('/roster');

    // Expect to see title/nav
    await expect(page.getByText('Tribe Roster')).toBeVisible();

    // Expect items
    await expect(page.getByText('Alice')).toBeVisible();
    await expect(page.getByText('Bob')).toBeVisible();
    await expect(page.getByText('0x1234...0000')).toBeVisible();
  });

  test('should filter list (mocked by frontend check or backend call)', async ({ page }) => {
      // Since filtering is server-side in our implementation, this test verifies the query param is sent.
      // We can intercept the request and check params.

      let searchParam = '';
      await page.route('http://localhost:5038/api/roster*', async route => {
          const url = new URL(route.request().url());
          searchParam = url.searchParams.get('search') || '';
          await route.fulfill({ json: [mockRoster[0]] }); // Return only Alice
      });

      await page.goto('/roster');
      const searchBox = page.getByPlaceholder('Search by username');
      await searchBox.fill('Alice');

      // Wait for debounce/fetch
      await page.waitForResponse(resp => resp.url().includes('search=Alice'));

      expect(searchParam).toBe('Alice');
      await expect(page.getByText('Alice')).toBeVisible();
      // Bob should be filtered out if we return filtered list
      await expect(page.getByText('Bob')).not.toBeVisible();
  });
});
