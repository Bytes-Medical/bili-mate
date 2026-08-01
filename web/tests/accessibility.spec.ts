// Automated accessibility checks (TEST-024): zero serious or critical axe
// findings on the key screens, plus the focus behaviour spec 06 requires.
// The manual keyboard/screen-reader review is a separate release-gate
// activity and is tracked in IMPLEMENTATION_PLAN.md.

import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

import { fillWholeAssessment, preAcknowledge, submitAssessment } from "./helpers";

async function expectNoSeriousViolations(page: import("@playwright/test").Page, label: string) {
  const results = await new AxeBuilder({ page }).analyze();
  const serious = results.violations.filter(
    (violation) => violation.impact === "serious" || violation.impact === "critical",
  );
  expect(
    serious,
    `${label}: ${serious.map((v) => `${v.id}: ${v.help}`).join("; ")}`,
  ).toEqual([]);
}

test("home page has no serious accessibility findings", async ({ page }) => {
  await page.goto("/");
  await expectNoSeriousViolations(page, "home");
});

test("acknowledgement gate has no serious accessibility findings", async ({ page }) => {
  await page.goto("/assessment");
  await expectNoSeriousViolations(page, "gate");
});

test("assessment steps have no serious accessibility findings", async ({ page }) => {
  await preAcknowledge(page);
  await page.goto("/assessment");
  await expectNoSeriousViolations(page, "step 1");
  await page.getByTestId("continue").click();
  await expectNoSeriousViolations(page, "step 2");
});

test("result page has no serious accessibility findings", async ({ page }) => {
  await preAcknowledge(page);
  await fillWholeAssessment(page);
  await submitAssessment(page);
  await expect(page.getByTestId("primary-action")).toBeVisible();
  await expectNoSeriousViolations(page, "result");
});

test("invalid submission moves focus to the error summary", async ({ page }) => {
  await preAcknowledge(page);
  await page.goto("/assessment");
  // Walk straight to review with an empty form, then submit.
  for (let i = 0; i < 5; i += 1) {
    await page.getByTestId("continue").click();
  }
  await page.getByTestId("check-thresholds").click();
  const summary = page.getByTestId("error-summary");
  await expect(summary).toBeVisible();
  await expect(summary).toBeFocused();
  await expect(summary).toContainText("Select present, absent or unknown");
});

test("the gate is operable with the keyboard alone", async ({ page }) => {
  // Sequential Tab traversal of links differs by platform convention
  // (Safari uses Option+Tab), so this test proves every control is
  // keyboard-activatable: Space toggles each checkbox and Enter activates
  // the confirm button, with no pointer interaction at all.
  await page.goto("/assessment");
  for (const checkbox of await page.getByRole("checkbox").all()) {
    await checkbox.focus();
    await expect(checkbox).toBeFocused();
    await page.keyboard.press("Space");
    await expect(checkbox).toBeChecked();
  }
  const confirm = page.getByRole("button", { name: "Confirm and continue" });
  await confirm.focus();
  await expect(confirm).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("heading", { name: "Neonatal jaundice assessment" })).toBeVisible();
});
