import { defineConfig } from '@playwright/test';

// One smoke test, run at the three layout sizes from docs/plan/05-PLAYGROUND-UX.md §2.
// It runs against the production build (`pnpm build` first), served by `vite preview`.
export default defineConfig({
  testDir: './e2e',
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? [['github'], ['html', { open: 'never' }]] : 'list',
  use: { baseURL: 'http://localhost:4173', browserName: 'chromium', trace: 'retain-on-failure' },
  projects: [
    { name: 'phone', use: { viewport: { width: 375, height: 667 }, isMobile: true, hasTouch: true } },
    { name: 'tablet', use: { viewport: { width: 820, height: 1180 }, isMobile: true, hasTouch: true } },
    { name: 'desktop', use: { viewport: { width: 1366, height: 768 } } },
  ],
  webServer: {
    command: 'pnpm preview --port 4173 --strictPort',
    url: 'http://localhost:4173',
    reuseExistingServer: !process.env.CI,
  },
});
