import { test, expect } from '@playwright/test';

test.describe('Stub API Integration', () => {
  // This test requires the Stub API to be running on port 5038

  test('should login via stub and view roster', async ({ page }) => {
    // 1. Initiate Stub Login for Admin User
    // The stub API is at http://localhost:5038/api/auth/stub-login?user_id=admin-user-id
    // This redirects to frontend /auth/callback which handles the token

    await page.goto('http://localhost:5038/api/auth/stub-login?user_id=admin-user-id');

    // 2. Wait for redirect to dashboard
    await page.waitForURL('**/home');

    // Verify we are logged in (localStorage has token)
    const token = await page.evaluate(() => localStorage.getItem('sui_jwt'));
    expect(token).toBeTruthy();

    // 3. Navigate to Roster
    await page.goto('/roster');

    // 4. Verify Content from Stub DB
    // AdminUser should be visible
    await expect(page.getByText('AdminUser')).toBeVisible();
    // RegularUser should be visible
    await expect(page.getByText('RegularUser')).toBeVisible();
    // Admin Wallet Address
    await expect(page.getByText('0xadmi...8789')).toBeVisible();
  });
});
