import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// config.ts reads `window.ENV` at module scope, so we need to:
// 1. Set up `globalThis.window` (since vitest runs in Node, not a browser)
// 2. Use vi.resetModules() + dynamic import to re-evaluate config.ts each time

// Minimal window shim for Node environment
function ensureWindow(): Window & { ENV?: Record<string, string | undefined> } {
    if (typeof globalThis.window === 'undefined') {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        (globalThis as any).window = globalThis;
    }
    return globalThis.window as Window & { ENV?: Record<string, string | undefined> };
}

describe('config', () => {
    const originalViteApiUrl = import.meta.env.VITE_API_URL;
    const originalViteBlockExplorerUrl = import.meta.env.VITE_BLOCK_EXPLORER_URL;
    const originalViteMumbleServerUrl = import.meta.env.VITE_MUMBLE_SERVER_URL;
    const originalViteSuiNetwork = import.meta.env.VITE_SUI_NETWORK;

    beforeEach(() => {
        vi.resetModules();
        const win = ensureWindow();
        delete win.ENV;
        // Clear build-time env vars
        delete import.meta.env.VITE_API_URL;
        delete import.meta.env.VITE_BLOCK_EXPLORER_URL;
        delete import.meta.env.VITE_MUMBLE_SERVER_URL;
        delete import.meta.env.VITE_SUI_NETWORK;
    });

    afterEach(() => {
        const win = ensureWindow();
        delete win.ENV;
        // Restore original values
        if (originalViteApiUrl !== undefined) import.meta.env.VITE_API_URL = originalViteApiUrl;
        if (originalViteBlockExplorerUrl !== undefined) import.meta.env.VITE_BLOCK_EXPLORER_URL = originalViteBlockExplorerUrl;
        if (originalViteMumbleServerUrl !== undefined) import.meta.env.VITE_MUMBLE_SERVER_URL = originalViteMumbleServerUrl;
        if (originalViteSuiNetwork !== undefined) import.meta.env.VITE_SUI_NETWORK = originalViteSuiNetwork;
    });

    describe('API_URL resolution priority', () => {
        it('should use window.ENV.VITE_API_URL when set', async () => {
            const win = ensureWindow();
            win.ENV = { VITE_API_URL: 'https://runtime-api.example.com' };
            const { API_URL } = await import('./config');
            expect(API_URL).toBe('https://runtime-api.example.com');
        });

        it('should use import.meta.env.VITE_API_URL when window.ENV is absent', async () => {
            import.meta.env.VITE_API_URL = 'https://buildtime-api.example.com';
            const { API_URL } = await import('./config');
            expect(API_URL).toBe('https://buildtime-api.example.com');
        });

        it('should fall back to default when neither source provides a value', async () => {
            const { API_URL } = await import('./config');
            expect(API_URL).toBe('http://localhost:5038');
        });

        it('should prefer window.ENV over import.meta.env', async () => {
            const win = ensureWindow();
            win.ENV = { VITE_API_URL: 'https://runtime-wins.example.com' };
            import.meta.env.VITE_API_URL = 'https://buildtime-loses.example.com';
            const { API_URL } = await import('./config');
            expect(API_URL).toBe('https://runtime-wins.example.com');
        });

        it('should fall through empty window.ENV to import.meta.env', async () => {
            const win = ensureWindow();
            win.ENV = {};
            import.meta.env.VITE_API_URL = 'https://buildtime-fallback.example.com';
            const { API_URL } = await import('./config');
            expect(API_URL).toBe('https://buildtime-fallback.example.com');
        });
    });

    describe('BLOCK_EXPLORER_URL resolution', () => {
        it('should use window.ENV value when set', async () => {
            const win = ensureWindow();
            win.ENV = { VITE_BLOCK_EXPLORER_URL: 'https://custom-explorer.example.com' };
            const { BLOCK_EXPLORER_URL } = await import('./config');
            expect(BLOCK_EXPLORER_URL).toBe('https://custom-explorer.example.com');
        });

        it('should fall back to default', async () => {
            const { BLOCK_EXPLORER_URL } = await import('./config');
            expect(BLOCK_EXPLORER_URL).toBe('https://suiscan.xyz/mainnet');
        });
    });

    describe('MUMBLE_SERVER_URL resolution', () => {
        it('should use window.ENV value when set', async () => {
            const win = ensureWindow();
            win.ENV = { VITE_MUMBLE_SERVER_URL: 'custom-mumble.example.com' };
            const { MUMBLE_SERVER_URL } = await import('./config');
            expect(MUMBLE_SERVER_URL).toBe('custom-mumble.example.com');
        });

        it('should fall back to default', async () => {
            const { MUMBLE_SERVER_URL } = await import('./config');
            expect(MUMBLE_SERVER_URL).toBe('mumble.void.scetrov.live');
        });
    });

    describe('SUI_NETWORK resolution', () => {
        it('should use window.ENV value when set', async () => {
            const win = ensureWindow();
            win.ENV = { VITE_SUI_NETWORK: 'mainnet' };
            const { SUI_NETWORK } = await import('./config');
            expect(SUI_NETWORK).toBe('mainnet');
        });

        it('should fall back to default', async () => {
            const { SUI_NETWORK } = await import('./config');
            expect(SUI_NETWORK).toBe('testnet');
        });
    });
});
