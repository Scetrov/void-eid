import { Page } from '@playwright/test';

/**
 * Injects a mock wallet into the page that implements the Wallet Standard.
 * @param page The Playwright Page object.
 * @param network The network to simulate (e.g., 'sui:mainnet', 'sui:testnet').
 */
export async function injectMockWallet(page: Page) {
    await page.addInitScript(() => {
        const MOCK_WALLET_NAME = 'My Mock Wallet';

        // Mock Account
        const mockAccount = {
            address: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
            publicKey: new Uint8Array(32).fill(1),
            chains: ['sui:mainnet', 'sui:testnet', 'sui:devnet', 'mainnet', 'testnet', 'devnet', 'localnet'],
            features: ['sui:signTransactionBlock', 'sui:signPersonalMessage', 'sui:signTransaction'],
            label: 'Mock Account 1'
        };

        // Standard Connect Feature
        const connectFeature = {
            version: '1.0.0',
            connect: async () => ({ accounts: [mockAccount] })
        };

        // Standard Disconnect Feature
        const disconnectFeature = {
            version: '1.0.0',
            disconnect: async () => {}
        };

        // Standard Events Feature
        const eventsFeature = {
            version: '1.0.0',
            on: () => {
                // Determine logic for events if needed. For now, no-op.
                return () => {};
            }
        };

        // The Wallet Object
        const mockWallet = {
            name: MOCK_WALLET_NAME,
            icon: 'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNDgiIGhlaWdodD0iNDgiIHZpZXdCb3g9IjAgMCA0OCA0OCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHBhdGggZD0iTTAgMjRDMCAxMC43NDUyIDEwLjc0NTIgMCAyNCAwQzM3LjI1NDggMCA0OCAxMC43NDUyIDQ4IDI0QzQ4IDM3LjI1NDggMzcuMjU0OCA0OCAyNCA0OEMxMC43NDUyIDQ4IDAgMzcuMjU0OCAwIDI0WiIgZmlsbD0iIzBDMEExRiIvPgo8cGF0aCBkPSJNMTMuMTM1OCAzMi4xMDg1QzE0LjE3MDEgMzUuOTY4MyAxOC4wMzMxIDM5LjQ2MjQgMjYuMDI1NSAzNy4zMjA4QzMzLjY1MTUgMzUuMjc3NCAzOC40MzA5IDI5LjAwNCAzNy4xOTE2IDI0LjM3ODlDMzYuNzYzNiAyMi43ODE3IDM1LjQ3NDYgMjEuNzAwNiAzMy40ODcyIDIxLjg3NjVMMTUuNzE2NSAyMy4zNTcyQzE0LjU5NzMgMjMuNDQzIDE0LjA4NDIgMjMuMjU5NiAxMy43ODgxIDIyLjU1NDNDMTMuNTAxIDIxLjg4MjMgMTMuNjY0NiAyMS4xNjA5IDE1LjAxNjMgMjAuNDc3N0wyOC41NDAxIDEzLjUzNzRDMjkuNTc2NyAxMy4wMSAzMC4yNjcxIDEyLjc4OTMgMzAuODk4IDEzLjAxMjZDMzEuMjkzNCAxMy4xNTYzIDMxLjU1MzggMTMuNzI4NCAzMS4zMTQ3IDE0LjQzNDRMMzAuNDM3OCAxNy4wMjMyQzI5LjM2MTcgMjAuMjAwMiAzMS42NjUzIDIwLjkzODIgMzIuOTY0MSAyMC41OTAyQzM0LjkyODkgMjAuMDYzNyAzNS4zOTExIDE4LjE5MjMgMzQuNzU4MSAxNS44Mjk5QzMzLjE1MzMgOS44NDA1NCAyNi43OTkgOC45MDQxMSAyMS4wMzc4IDEwLjQ0NzhDMTUuMTc2NyAxMi4wMTgzIDEwLjA5NiAxNi43Njc2IDExLjY0NzQgMjIuNTU3M0MxMi4wMTI5IDIzLjkyMTYgMTMuMjY4NyAyNS4wMTE2IDE0LjcyMzIgMjQuOTc4NUwxNi45NDM4IDI0Ljk3MzFDMTcuNDAwNCAyNC45NjI1IDE3LjIzNiAyNSAxOC4xMTcgMjQuOTI3MUMxOC45OTggMjQuODU0MSAyMS4zNTA5IDI0LjU2NDYgMjEuMzUwOSAyNC41NjQ2TDMyLjg5NjIgMjMuMjU4TDMzLjE5MzcgMjMuMjE0OEMzMy44Njg5IDIzLjA5OTcgMzQuMzc5MiAyMy4yNzUgMzQuODEwNiAyNC4wMTgzQzM1LjQ1NjMgMjUuMTMwNCAzNC40NzEyIDI1Ljk2OTEgMzMuMjkyIDI2Ljk3MzFDMzMuMjYwNSAyNyAzMy4yMjg4IDI3LjAyNyAzMy4xOTcgMjcuMDU0MUwyMy4wNDgyIDM1LjgwMDVDMjEuMzA4NyAzNy4zMDA4IDIwLjA4NjcgMzYuNzM2NyAxOS42NTg4IDM1LjEzOTVMMTguMTQzMSAyOS40ODI5QzE3Ljc2ODcgMjguMDg1NCAxNi40MDQxIDI2Ljk4ODkgMTQuODA1NiAyNy40MTcyQzEyLjgwNzUgMjcuOTUyNiAxMi42NDU1IDMwLjI3ODQgMTMuMTM1OCAzMi4xMDg1WiIgZmlsbD0iI0ZCRkFGRiIvPgo8L3N2Zz4K',
            version: '1.0.0',
            accounts: [mockAccount],
            chains: ['sui:mainnet', 'sui:testnet', 'sui:devnet', 'mainnet', 'testnet', 'devnet', 'localnet'],
            features: {
                'standard:connect': connectFeature,
                'standard:disconnect': disconnectFeature,
                'standard:events': eventsFeature,
                'sui:signTransactionBlock': { version: '1.0.0', signTransactionBlock: async () => ({ signature: 'mock', transactionBlockBytes: 'mock' }) },
                'sui:signAndExecuteTransactionBlock': { version: '1.0.0', signAndExecuteTransactionBlock: async () => ({ digest: 'mock' }) },
                'sui:signTransaction': { version: '2.0.0', signTransaction: async () => ({ signature: 'mock', bytes: 'mock' }) },
                'sui:signMessage': { version: '1.0.0', signMessage: async () => ({ signature: 'mock', messageBytes: 'mock' }) }
            }
        };

        // Expose a way to manually trigger register if needed, but dispatching now should work for listeners attaching soon.
        // Also expose as window.suiWallet for legacy detection
        (window as unknown as { suiWallet: unknown }).suiWallet = mockWallet;

        // Helper to register via navigator.wallets if available (Standard Standard)
        const registerViaNavigator = () => {
             const navigator = window.navigator as unknown as { wallets?: { push(item: unknown): void } | unknown[] };
             if (!navigator.wallets) {
                 navigator.wallets = [];
             }
             if (Array.isArray(navigator.wallets)) {
                 navigator.wallets.push({ register: (wallets: { register(w: unknown): void }) => wallets.register(mockWallet) });
             } else if (navigator.wallets && typeof (navigator.wallets as { push: unknown }).push === 'function') {
                 (navigator.wallets as { push(item: unknown): void }).push({ register: (wallets: { register(w: unknown): void }) => wallets.register(mockWallet) });
             }
        };

        function register() {
             console.log('[MockWallet] Registering wallet:', MOCK_WALLET_NAME);
             // 1. Dispatch event (The Standard)
             window.dispatchEvent(new CustomEvent('wallet-standard:register-wallet', {
                bubbles: true,
                cancelable: false,
                detail: () => mockWallet
            }));

            // 2. Push to navigator (The "Host" pattern)
            registerViaNavigator();
        }

        // Initial Registration
        register();

        // Periodic re-registration to ensure app picks it up
        const interval = setInterval(register, 100);

        // Clear interval after 5 seconds
        setTimeout(() => clearInterval(interval), 5000);

    });
}
