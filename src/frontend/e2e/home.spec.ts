import { test, expect } from '@playwright/test';

test.describe('Home Page', () => {
  // Mock User Data
  const mockUser = {
    id: "user-id",
    discordId: "123456",
    username: "TestUser",
    discriminator: "0000",
    avatar: null,
    tribe: "Fire",
    isAdmin: false,
    lastLoginAt: "2026-01-21T10:30:00Z",
    wallets: [
      { id: "wallet-1", address: "0x1234567890abcdef1234567890abcdef12345678", verifiedAt: "2026-01-20T12:00:00Z" }
    ]
  };

  test.beforeEach(async ({ page }) => {
    // Mock /api/me to return user - MUST be before any navigation
    await page.route('**/api/me', async route => {
      const headers = route.request().headers();
      if (headers['authorization']) {
        await route.fulfill({ json: mockUser });
      } else {
        await route.fulfill({ status: 401 });
      }
    });

    await page.goto('/');
    
    // Set JWT token after page load
    await page.evaluate(() => {
      localStorage.setItem('sui_jwt', 'fake-token');
    });
  });

  test('should display user profile when logged in', async ({ page }) => {
    await page.goto('/home');

    // Expect to see username
    await expect(page.getByText('TestUser')).toBeVisible();
    // Expect Discord Connected badge
    await expect(page.getByText('Discord Connected')).toBeVisible();
  });

  test('should show linked wallets', async ({ page }) => {
    await page.goto('/home');

    // Expect to see wallet count
    await expect(page.getByText('Linked Wallets (1)')).toBeVisible();
    // Expect to see wallet address (truncated)
    await expect(page.getByText('0x1234...5678')).toBeVisible();
  });

  test('should display last login banner', async ({ page }) => {
    await page.goto('/home');

    // Expect to see last login banner
    await expect(page.getByText('Last logged in on')).toBeVisible();
    await expect(page.getByText('2026.01.21 at 10:30 UTC')).toBeVisible();
  });

  test('should redirect to login when not authenticated', async ({ page }) => {
    // Clear token
    await page.evaluate(() => {
      localStorage.removeItem('sui_jwt');
    });

    await page.goto('/home');

    // Expect access denied or redirect prompt
    await expect(page.getByRole('heading', { name: 'Access Denied' })).toBeVisible();
  });

  test('should show Link Another Wallet section when wallets exist', async ({ page }) => {
    await page.goto('/home');

    // Expect Link Another Wallet section
    await expect(page.getByText('Link Another Wallet')).toBeVisible();
  });
});
