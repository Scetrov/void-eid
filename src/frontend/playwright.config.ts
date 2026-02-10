import { defineConfig, devices } from '@playwright/test';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// stub_api should be pre-built by test-e2e.sh script
const stubApiPath = join(__dirname, '../backend/target/debug/stub_api');
const stubApiCommand = process.env.STUB_API_CMD || stubApiPath;
const stubApiCwd = undefined;

// For E2E, use ports that don't conflict with development (5173 / 5038)
const frontendPort = Number(process.env.FRONTEND_PORT) || (process.env.CI ? 4173 : 5174);
const stubApiPort = Number(process.env.STUB_API_PORT) || 5039;

// For CI, use preview (production build already exists). For local, use dev server.
const frontendCommand = process.env.CI
  ? `bun run preview --port ${frontendPort}`
  : `bun run dev --port ${frontendPort}`;

export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: process.env.CI ? 'blob' : 'html',
  use: {
    baseURL: process.env.BASE_URL || `http://localhost:${frontendPort}`,
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        launchOptions: {
          args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu'],
        },
      },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
  ],
  webServer: [
    {
      command: frontendCommand,
      url: `http://localhost:${frontendPort}`,
      reuseExistingServer: false,
      timeout: 30_000,
      stdout: 'pipe',
      stderr: 'pipe',
      env: {
        ...process.env,
        VITE_API_URL: `http://localhost:${stubApiPort}`,
      },
    },
    {
      command: stubApiCommand,
      url: `http://localhost:${stubApiPort}/api/auth/discord/login`,
      cwd: stubApiCwd,
      reuseExistingServer: false,
      timeout: 120_000,
      stdout: 'pipe',
      stderr: 'pipe',
      env: {
        ...process.env,
        PORT: stubApiPort.toString(),
        DATABASE_URL: process.env.DATABASE_URL || 'sqlite::memory:',
        JWT_SECRET: process.env.JWT_SECRET || 'dev-jwt-secret',
        FRONTEND_URL: process.env.FRONTEND_URL || `http://localhost:${frontendPort}`,
      },
    },
  ],
});
