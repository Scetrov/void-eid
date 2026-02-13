import { test, expect } from './fixtures';

test.describe('Wallet Linking Network Enforcement', () => {

    test('should disable link button and show warning when wallet is on wrong network', async ({ authenticatedPage: page }) => {
        // Inject a mock wallet account on the WRONG network (mainnet instead of testnet)
        await page.addInitScript(() => {
            (window as unknown as { __MOCK_ACCOUNT__: unknown }).__MOCK_ACCOUNT__ = {
                address: '0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef',
                chains: ['sui:mainnet'],
                features: [],
            };
        });

        await page.goto('/home');

        // The authenticated user (from stub DB) has wallets, so "Link Another Wallet" section renders.
        // With __MOCK_ACCOUNT__ set and wrong network, the "Link this Wallet" button should be hidden.
        const linkBtn = page.getByRole('button', { name: 'Link this Wallet' });
        await expect(linkBtn).not.toBeVisible();

        // Check for wrong network warning
        await expect(page.getByText(/Wrong Network/i)).toBeVisible();
    });

    test('should enable link button when wallet is on correct network', async ({ authenticatedPage: page }) => {
        // Inject a mock wallet account on the CORRECT network (testnet)
        await page.addInitScript(() => {
            (window as unknown as { __MOCK_ACCOUNT__: unknown }).__MOCK_ACCOUNT__ = {
                address: '0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef',
                chains: ['sui:testnet'],
                features: [],
            };
        });

        await page.goto('/home');

        // With correct network, the "Link this Wallet" button should be visible and enabled
        const linkBtn = page.getByRole('button', { name: 'Link this Wallet' });
        await expect(linkBtn).toBeVisible();
        await expect(linkBtn).toBeEnabled();

        // Wrong network warning should NOT be visible
        await expect(page.getByText(/Wrong Network/i)).not.toBeVisible();
    });

});
