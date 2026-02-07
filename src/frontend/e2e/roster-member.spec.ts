import { test, expect } from './fixtures';

test.describe('Roster Member Detail Page', () => {
  test('should display member details for admin', async ({ adminPage: page }) => {
    // Navigate to RegularUser's detail page (discord_id: regular-discord-id from stub DB)
    await page.goto('/roster/regular-discord-id');

    // Expect member username heading
    await expect(page.getByRole('heading', { name: 'RegularUser' })).toBeVisible();
    // Expect Member Details header
    await expect(page.getByText('Member Details')).toBeVisible();
  });

  test('should show linked wallets', async ({ adminPage: page }) => {
    await page.goto('/roster/regular-discord-id');

    // Expect Linked Wallets section
    await expect(page.getByText('Linked Wallets')).toBeVisible();
    // Expect wallet address to be visible
    await expect(page.getByText('0xregularwallet987654321')).toBeVisible();
  });

  test('should display audit history', async ({ adminPage: page }) => {
    await page.goto('/roster/regular-discord-id');

    // Expect Audit History section
    await expect(page.getByText('Audit History')).toBeVisible();
  });

  test('should show back to roster link', async ({ adminPage: page }) => {
    await page.goto('/roster/regular-discord-id');
    // Expect back link
    await expect(page.getByText('Back to Roster')).toBeVisible();
  });
});

// Separate describe block for non-admin access test
test.describe('Roster Member Detail Page - Access Control', () => {
  test('should deny access for non-admin', async ({ authenticatedPage: page }) => {
    // Navigate to roster detail page as non-admin (RegularUser)
    await page.goto('/roster/admin-discord-id');

    // Expect access denied message
    await expect(page.getByText('Access Denied')).toBeVisible();
    await expect(page.getByText('Only users with the \'Admin\' role')).toBeVisible();
  });
});
