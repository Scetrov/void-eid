import { test as base, expect, Page } from '@playwright/test';

const STUB_API_PORT = process.env.STUB_API_PORT || '5039';
const FRONTEND_PORT = process.env.FRONTEND_PORT || (process.env.CI ? '4173' : '5174');
const API_URL = process.env.API_URL || `http://localhost:${STUB_API_PORT}`;

type AuthFixtures = {
  authenticatedPage: Page;
  adminPage: Page;
};

async function loginAs(page: Page, userId: number) {
  // Make request to stub API, following redirects manually
  const context = page.context();
  const response = await context.request.get(
    `${API_URL}/api/auth/stub-login?user_id=${userId}`,
    {
      maxRedirects: 0,  // Don't auto-follow
    }
  );

  // Check for redirect response (302, 303, 307, etc.)
  if (response.status() >= 300 && response.status() < 400) {
    const location = response.headers()['location'];
    if (!location) {
      throw new Error(`Redirect response (${response.status()}) but no location header`);
    }

    // Extract code from redirect URL
    const url = new URL(location, `http://localhost:${FRONTEND_PORT}`);
    const code = url.searchParams.get('code');
    if (!code) {
      throw new Error(`No code found in redirect URL: ${location}`);
    }

    // Exchange code for token
    const exchangeResponse = await context.request.post(
      `${API_URL}/api/auth/exchange`,
      {
        data: { code }
      }
    );

    if (!exchangeResponse.ok()) {
      throw new Error(`Failed to exchange code: ${exchangeResponse.status()} ${await exchangeResponse.text()}`);
    }

    const { token } = await exchangeResponse.json();
    if (!token) {
      throw new Error('No token in exchange response');
    }

    // Navigate to the frontend and set the token
    await page.goto('/');
    await page.evaluate((t) => localStorage.setItem('sui_jwt', t), token);

    // Set up response listener BEFORE reload (the reload triggers /api/me)
    const meResponsePromise = page.waitForResponse(
      response => response.url().includes('/api/me') && response.status() === 200,
      { timeout: 10000 }
    );

    // Reload to trigger AuthProvider to load user
    await page.reload();
    await meResponsePromise;

    return;
  }

  // If not a redirect, something went wrong
  throw new Error(`Expected redirect from stub API, got ${response.status()}: ${await response.text()}`);
}

/**
 * Fixture that logs in as a regular user (ID 1002 - RegularUser)
 */
/* eslint-disable react-hooks/rules-of-hooks -- Playwright's use() is not a React hook */
export const test = base.extend<AuthFixtures>({
  authenticatedPage: async ({ page }, use) => {
    await loginAs(page, 1002);
    await use(page);
  },

  adminPage: async ({ page }, use) => {
    await loginAs(page, 1001);
    await use(page);
  },
});
/* eslint-enable react-hooks/rules-of-hooks */

export { expect };
