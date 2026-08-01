import { defineConfig, devices } from "@playwright/test";

// Browser end-to-end tests run against the BUILT static export (the real
// deployment artifact) backed by the real Rust API (TEST-023). Build first:
//   NEXT_PUBLIC_API_BASE_URL=http://localhost:18099 npm run build

export default defineConfig({
  testDir: "./tests",
  timeout: 60_000,
  fullyParallel: true,
  retries: process.env.CI ? 1 : 0,
  reporter: [["list"]],
  use: {
    baseURL: "http://localhost:3100",
  },
  projects: [
    { name: "chromium", use: { ...devices["Desktop Chrome"] } },
    { name: "firefox", use: { ...devices["Desktop Firefox"] } },
    { name: "webkit", use: { ...devices["Desktop Safari"] } },
  ],
  webServer: [
    {
      command: "npx serve out -l 3100",
      url: "http://localhost:3100",
      reuseExistingServer: true,
      timeout: 30_000,
    },
    {
      command: "cargo run -p bili-mate-api",
      cwd: "..",
      url: "http://localhost:18099/health/live",
      reuseExistingServer: true,
      timeout: 180_000,
      env: {
        BILI_MATE_BIND: "127.0.0.1:18099",
        BILI_MATE_ALLOWED_ORIGINS: "http://localhost:3100",
      },
    },
  ],
});
