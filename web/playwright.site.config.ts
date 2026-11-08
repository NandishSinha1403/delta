import { defineConfig } from '@playwright/test';

// The whole site (front page, docs, and the Pad at /pad/), built by scripts/build-site.sh and
// served by `astro preview` from site/.
export default defineConfig({
  testDir: './e2e-site',
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? [['github']] : 'list',
  use: { baseURL: 'http://localhost:4321', browserName: 'chromium' },
  projects: [
    { name: 'desktop', use: { viewport: { width: 1280, height: 900 } } },
    { name: 'phone', use: { viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true } },
  ],
  webServer: {
    command: 'pnpm --dir ../site preview --port 4321',
    url: 'http://localhost:4321',
    reuseExistingServer: !process.env.CI,
  },
});
