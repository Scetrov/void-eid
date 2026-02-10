import { test, expect } from '@playwright/test';

test.describe('Login Page', () => {
  test('should display login button', async ({ page }) => {
    await page.goto('/login');

    // Expect login with Discord button
    await expect(page.getByRole('button', { name: 'Login with Discord' })).toBeVisible();

    // Expect Logo
    await expect(page.locator('img[alt="VoID eID"]')).toBeVisible();
  });

  test('should redirect to Discord OAuth when clicking login', async ({ page }) => {
    // We can't fully test OAuth flow, but we can verify the redirect is attempted
    await page.goto('/login');

    // Intercept navigation to Discord
    const [request] = await Promise.all([
      page.waitForRequest(request => request.url().includes('/api/auth/discord/login')),
      page.getByRole('button', { name: 'Login with Discord' }).click()
    ]);

    expect(request.url()).toContain('/api/auth/discord/login');
  });
});

test.describe('Auth Callback', () => {
  test('should store token from query parameter', async ({ page }) => {
    // Mock the /api/me endpoint
    await page.route('**/api/me', async route => {
      await route.fulfill({
        json: {
          id: "test-user-id",
          discordId: "123",
          username: "TestUser",
          discriminator: "0000",
          avatar: null,
          tribes: ["Fire"],
          adminTribes: [],
          isAdmin: false,
          lastLoginAt: null,
          wallets: []
        }
      });
    });

    // Navigate to callback with token
    await page.goto('/auth/callback?token=test-jwt-token');

    // Wait for redirect to /home (now using client-side routing)
    await page.waitForURL('**/home', { timeout: 5000 });

    // Check localStorage was set
    const token = await page.evaluate(() => localStorage.getItem('sui_jwt'));
    expect(token).toBe('test-jwt-token');
  });

  test('should redirect to login when no token provided', async ({ page }) => {
    await page.goto('/auth/callback');

    // Should redirect to login page
    await page.waitForURL('**/login', { timeout: 5000 });
  });
});
