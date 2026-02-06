import { test, expect } from '@playwright/test';

test.describe('Roster Page', () => {
  // Mock User Data
  const mockUser = {
    id: "1001",
    discordId: "123",
    username: "AdminUser",
    discriminator: "0000",
    avatar: null,
    tribes: ["Fire"],
    adminTribes: ["Fire"],
    isAdmin: true,
    wallets: []
  };

  const mockRoster = [
    {
        discordId: "1",
        username: "Alice",
        avatar: null,
        wallets: [
            { id: "w1", address: "0x1234567890abcdef1234567890abcdef56780000", tribes: [] }
        ]
    },
    { discordId: "2", username: "Bob", avatar: null, wallets: [] }
  ];

  test.beforeEach(async ({ page }) => {
    // Set JWT token BEFORE any navigation so AuthProvider initializes with it
    await page.goto('/');
    await page.evaluate(() => {
        localStorage.setItem('sui_jwt', 'fake-token');
    });

    // Set up route mocks BEFORE reload to prevent unmocked API calls
    // Mock /api/me to return admin user
    await page.route('**/api/me', async route => {
        // Only return admin if Authorization header is present (simulated)
        const headers = route.request().headers();
        if (headers['authorization']) {
             await route.fulfill({ json: mockUser });
        } else {
             await route.fulfill({ status: 401 });
        }
    });

    // Mock /api/roster
    await page.route('**/api/roster*', async route => {
        await route.fulfill({ json: mockRoster });
    });

    // Reload to reinitialize AuthProvider with the token
    await page.reload();
  });

  test('should display roster table for admin', async ({ page }) => {
    await page.goto('/roster');
    await page.waitForSelector('table');

    // Expect to see title/nav
    await expect(page.getByRole('heading', { name: 'Roster' })).toBeVisible();

    // Expect items
    await expect(page.getByText('Alice')).toBeVisible();
    await expect(page.getByText('Bob')).toBeVisible();
    await expect(page.getByText('0x1234...0000')).toBeVisible();
  });

  test('should filter list (mocked by frontend check or backend call)', async ({ page }) => {
      // Since filtering is server-side in our implementation, this test verifies the query param is sent.
      // We can intercept the request and check params.

      let searchParam = '';
      await page.route('**/api/roster*', async route => {
          const url = new URL(route.request().url());
          searchParam = url.searchParams.get('search') || '';
          // Return valid structure here too
          await route.fulfill({ json: [mockRoster[0]] });
      });

      await page.goto('/roster');

      // Wait for table to load to ensure page is stable
      await page.waitForSelector('table');

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
