import { test, expect } from './fixtures';

test.describe('Home Page', () => {
  test('should display user profile when logged in', async ({ authenticatedPage: page }) => {
    await page.goto('/home', { waitUntil: 'networkidle' });

    // Wait for /api/me to complete after navigation
    await page.waitForLoadState('networkidle');

    // Expect to see username (from stub DB: RegularUser)
    await expect(page.getByText('RegularUser')).toBeVisible();
    // Expect Discord Connected badge
    await expect(page.getByText('Discord Connected')).toBeVisible();
  });

  test('should show linked wallets', async ({ authenticatedPage: page }) => {
    await page.goto('/home');

    // Expect to see wallet count
    await expect(page.getByText('Linked Wallets (1)')).toBeVisible();
    // Expect to see wallet address (from stub DB: 0xregularwallet987654321)
    // formatAddress uses slice(0,6)...slice(-4) = 0xregu...4321
    await expect(page.getByText('0xregu...4321')).toBeVisible();
  });

  test('should display last login banner', async ({ authenticatedPage: page }) => {
    await page.goto('/home');

    // Expect to see last login banner (timestamp from stub DB will be recent)
    await expect(page.getByText('Last logged in on')).toBeVisible();
  });

  test('should redirect to login when not authenticated', async ({ page }) => {
    // Don't use the authenticated fixture for this test
    await page.goto('/home');

    // Expect access denied or redirect prompt
    await expect(page.getByRole('heading', { name: 'Access Denied' })).toBeVisible();
  });

  test('should show Link Another Wallet section when wallets exist', async ({ authenticatedPage: page }) => {
    await page.goto('/home');

    // Expect Link Another Wallet section
    await expect(page.getByText('Link Another Wallet')).toBeVisible();
  });
});
