// Local elapsed-age derivation (WEB-004, WEB-005, spec 09 DST cases):
// elapsed minutes are actual elapsed time across daylight-saving
// transitions, and impossible local times must be corrected.

import { expect, test } from "@playwright/test";

import { preAcknowledge } from "./helpers";

async function enterTimes(
  page: import("@playwright/test").Page,
  birth: [string, string],
  assessment: [string, string],
) {
  await page.goto("/assessment");
  await page.getByLabel("Gestational age at birth").fill("38");
  await page.getByLabel("Timezone for all dates and times").selectOption("Europe/London");
  await page.getByLabel("Birth date and time date").fill(birth[0]);
  await page.getByLabel("Birth date and time time (24 hour)").fill(birth[1]);
  await page.getByLabel("Assessment date and time date").fill(assessment[0]);
  await page.getByLabel("Assessment date and time time (24 hour)").fill(assessment[1]);
}

test.beforeEach(async ({ page }) => {
  await preAcknowledge(page);
});

test("spring-forward: 24 wall-clock hours is 23 elapsed hours", async ({ page }) => {
  // Europe/London clocks go forward at 01:00 GMT on 2026-03-29.
  await enterTimes(page, ["2026-03-28", "12:00"], ["2026-03-29", "12:00"]);
  await expect(page.getByTestId("derived-age-inline")).toContainText("23 h 0 min (1,380 minutes)");
});

test("autumn-back: 24 wall-clock hours is 25 elapsed hours", async ({ page }) => {
  // Europe/London clocks go back at 02:00 BST on 2026-10-25.
  await enterTimes(page, ["2026-10-24", "12:00"], ["2026-10-25", "12:00"]);
  await expect(page.getByTestId("derived-age-inline")).toContainText("1 d 1 h 0 min (1,500 minutes)");
});

test("a nonexistent spring-forward local time must be corrected", async ({ page }) => {
  // 01:30 on 2026-03-29 does not exist in Europe/London.
  await enterTimes(page, ["2026-03-29", "01:30"], ["2026-03-30", "12:00"]);
  await expect(
    page.getByText("does not exist in the selected timezone", { exact: false }),
  ).toBeVisible();
});

test("an assessment before birth must be corrected", async ({ page }) => {
  await enterTimes(page, ["2026-07-30", "12:00"], ["2026-07-29", "12:00"]);
  await expect(page.getByText("before the birth time", { exact: false })).toBeVisible();
});

test("gestation is labelled as completed weeks, not corrected", async ({ page }) => {
  await page.goto("/assessment");
  await expect(page.getByText("Do not round up for extra days", { exact: false })).toBeVisible();
  await expect(page.getByText("do not use corrected gestation", { exact: false })).toBeVisible();
});
