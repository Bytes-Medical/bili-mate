// Chart semantics (spec 06 chart rules, WEB-015): dash pattern and marker
// fill carry the meaning, and the tabular twin shows identical server
// points.

import { expect, test } from "@playwright/test";

import { fillWholeAssessment, preAcknowledge, submitAssessment } from "./helpers";

test.beforeEach(async ({ page }) => {
  await preAcknowledge(page);
  await fillWholeAssessment(page);
  await submitAssessment(page);
  await expect(page.getByTestId("threshold-chart")).toBeVisible();
});

test("lines are distinguished by dash pattern and markers by fill", async ({ page }) => {
  const photo = page.locator('[data-line="phototherapy"]');
  const exchange = page.locator('[data-line="exchange"]');
  await expect(photo).toHaveCount(1);
  await expect(exchange).toHaveAttribute("stroke-dasharray", /\d/);
  await expect(photo).not.toHaveAttribute("stroke-dasharray", /\d/);
  // The serum measurement renders as a filled marker with a text
  // alternative naming the server relationship.
  const marker = page.locator('[data-marker="serum"]');
  await expect(marker).toHaveAttribute("aria-label", /Serum result 180 µmol\/L/);
  await expect(marker).toHaveAttribute("aria-label", /below the phototherapy line/);
  await expect(marker.locator("circle")).toHaveAttribute("fill", "#000");
});

test("the table twin shows the identical server points", async ({ page }) => {
  const table = page.getByTestId("threshold-table");
  // Hourly resolution: 337 points from 0 through 20,160 minutes.
  await expect(table.locator("tbody tr")).toHaveCount(337);
  const first = table.locator("tbody tr").first();
  await expect(first).toContainText("100.0");
  const last = table.locator("tbody tr").last();
  await expect(last).toContainText("20160");
  await expect(last).toContainText("350.0");
  await expect(last).toContainText("450.0");
  // The 48-hour row matches the threshold shown in the result table.
  const row48h = table.locator("tbody tr", { hasText: "2880" }).first();
  await expect(row48h).toContainText("250.0");
});
