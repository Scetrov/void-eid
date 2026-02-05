import { test, expect } from '@playwright/test';

test.describe('Roster Member Detail Page', () => {
  // Mock User Data (Admin)
  const mockAdminUser = {
    id: "admin-id",
    discordId: "123",
    username: "AdminUser",
    discriminator: "0000",
    avatar: null,
    tribe: "Fire",
    isAdmin: true,
    lastLoginAt: "2026-01-21T10:30:00Z",
    wallets: []
  };

  // Mock Regular User Data
  const mockRegularUser = {
    id: "user-id",
    discordId: "456",
    username: "RegularUser",
    discriminator: "1111",
    avatar: null,
    tribe: "Fire",
    isAdmin: false,
    lastLoginAt: "2026-01-21T10:30:00Z",
    wallets: []
  };

  // Mock Member Detail
  const mockMember = {
    discord_id: "789",
    username: "MemberToView",
    avatar: null,
    wallets: ["0xabcdef1234567890abcdef1234567890abcdef12", "0x1111222233334444555566667777888899990000"],
    audits: [
      {
        id: "audit-1",
        action: "LINK_WALLET",
        actorId: "789",
        targetId: null,
        details: "Linked wallet 0xabcd...",
        createdAt: "2026-01-20T15:30:00Z",
        actorUsername: "MemberToView",
        actorDiscriminator: "2222"
      },
      {
        id: "audit-2",
        action: "LOGIN",
        actorId: "789",
        targetId: null,
        details: "User logged in via Discord",
        createdAt: "2026-01-20T10:00:00Z",
        actorUsername: "MemberToView",
        actorDiscriminator: "2222"
      }
    ]
  };

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await page.evaluate(() => {
      localStorage.setItem('sui_jwt', 'fake-token');
    });
  });

  test('should display member details for admin', async ({ page }) => {
    // Mock /api/me to return admin user
    await page.route('http://localhost:5038/api/me', async route => {
      await route.fulfill({ json: mockAdminUser });
    });

    // Mock member detail endpoint
    await page.route('http://localhost:5038/api/roster/789', async route => {
      await route.fulfill({ json: mockMember });
    });

    await page.goto('/roster/789');

    // Expect member username heading
    await expect(page.getByRole('heading', { name: 'MemberToView' })).toBeVisible();
    // Expect Member Details header
    await expect(page.getByText('Member Details')).toBeVisible();
  });

  test('should show linked wallets', async ({ page }) => {
    await page.route('http://localhost:5038/api/me', async route => {
      await route.fulfill({ json: mockAdminUser });
    });

    await page.route('http://localhost:5038/api/roster/789', async route => {
      await route.fulfill({ json: mockMember });
    });

    await page.goto('/roster/789');

    // Expect Linked Wallets section
    await expect(page.getByText('Linked Wallets')).toBeVisible();
    // Expect wallet addresses to be visible
    await expect(page.getByText('0xabcdef1234567890abcdef1234567890abcdef12')).toBeVisible();
  });

  test('should display audit history', async ({ page }) => {
    await page.route('http://localhost:5038/api/me', async route => {
      await route.fulfill({ json: mockAdminUser });
    });

    await page.route('http://localhost:5038/api/roster/789', async route => {
      await route.fulfill({ json: mockMember });
    });

    await page.goto('/roster/789');

    // Expect Audit History section
    await expect(page.getByText('Audit History')).toBeVisible();
    // Expect audit actions to be visible
    await expect(page.getByText('LINK_WALLET')).toBeVisible();
    await expect(page.getByText('LOGIN')).toBeVisible();
  });

  test('should deny access for non-admin', async ({ page }) => {
    await page.route('http://localhost:5038/api/me', async route => {
      await route.fulfill({ json: mockRegularUser });
    });

    await page.goto('/roster/789');

    // Expect access denied message
    await expect(page.getByText('Access Denied')).toBeVisible();
    await expect(page.getByText('Only users with the \'Admin\' role')).toBeVisible();
  });

  test('should show back to roster link', async ({ page }) => {
    await page.route('http://localhost:5038/api/me', async route => {
      await route.fulfill({ json: mockAdminUser });
    });

    await page.route('http://localhost:5038/api/roster/789', async route => {
      await route.fulfill({ json: mockMember });
    });

    await page.goto('/roster/789');

    // Expect back link
    await expect(page.getByText('Back to Roster')).toBeVisible();
  });
});
