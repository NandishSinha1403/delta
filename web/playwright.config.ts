import { defineConfig } from '@playwright/test';

// The smoke tests, run at six viewports (04-ROADMAP.md, P4 exit): two phones, a tablet in both
// orientations and two desktops. `metadata.layout` is the app layout each one gets.
// It runs against the production build (`pnpm build` first), served by `vite preview`.
export default defineConfig({
  testDir: './e2e',
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? [['github'], ['html', { open: 'never' }]] : 'list',
  use: { baseURL: 'http://localhost:4173', browserName: 'chromium', trace: 'retain-on-failure' },
  projects: [
    { name: 'phone-small', metadata: { layout: 'phone' }, use: { viewport: { width: 360, height: 640 }, isMobile: true, hasTouch: true } },
    { name: 'phone', metadata: { layout: 'phone' }, use: { viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true } },
    { name: 'tablet', metadata: { layout: 'tablet' }, use: { viewport: { width: 820, height: 1180 }, isMobile: true, hasTouch: true } },
    { name: 'tablet-landscape', metadata: { layout: 'tablet' }, use: { viewport: { width: 1180, height: 820 }, isMobile: true, hasTouch: true } },
    { name: 'desktop', metadata: { layout: 'desktop' }, use: { viewport: { width: 1366, height: 768 } } },
    { name: 'desktop-wide', metadata: { layout: 'desktop' }, use: { viewport: { width: 1920, height: 1080 } } },
  ],
  webServer: {
    command: 'pnpm preview --port 4173 --strictPort',
    url: 'http://localhost:4173',
    reuseExistingServer: !process.env.CI,
  },
});
