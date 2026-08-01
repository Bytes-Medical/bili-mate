// Full assessment workflow against the real API (TEST-023): the same facts
// as the specification's normal-below-threshold example, plus the emergency
// presentation path (WEB-011, WEB-012).

import { expect, test } from "@playwright/test";

import { fillWholeAssessment, preAcknowledge, submitAssessment } from "./helpers";

test.beforeEach(async ({ page }) => {
  await preAcknowledge(page);
});

test("well term baby below threshold reaches NO_ROUTINE_REPEAT", async ({ page }) => {
  await fillWholeAssessment(page);
  // Derived age is shown before submission (WEB-005).
  await expect(page.getByTestId("derived-age")).toContainText("2,880 minutes");
  await submitAssessment(page);

  await expect(page.getByRole("heading", { name: "Evaluation result" })).toBeVisible();
  await expect(page.getByTestId("primary-action")).toContainText(
    "Do not routinely repeat the bilirubin measurement",
  );
  await expect(page.getByTestId("result-mode")).toContainText("demonstration");
  // Server display values, not browser recalculation (WEB-014).
  const table = page.getByTestId("threshold-assessments");
  await expect(table).toContainText("250.0");
  await expect(table).toContainText("-70.0");
  await expect(table).toContainText("450.0");
  // Rule pack and source date visible without a secondary screen (WEB-016).
  await expect(page.getByTestId("rule-pack-line")).toContainText("nice-cg98-2023-10-31.1");
  await expect(page.getByTestId("rule-pack-line")).toContainText("2023-10-31");
  // Assay warning (PRD-023) and NICE attribution (WEB-017).
  await expect(page.getByTestId("assay-warning")).toContainText("pathology");
  await expect(
    page.getByText("NICE UK Open Content Licence", { exact: false }).first(),
  ).toBeVisible();
  // No emergency banner for a routine primary action.
  await expect(page.getByTestId("priority-banner")).toHaveCount(0);
});

test("encephalopathy presents an emergency banner with a live region", async ({ page }) => {
  await fillWholeAssessment(page, { encephalopathy: "Present" });
  await submitAssessment(page);

  const banner = page.getByTestId("priority-banner");
  await expect(banner).toBeVisible();
  await expect(banner).toContainText("emergency");
  await expect(banner).toHaveAttribute("role", "alert");
  await expect(page.getByTestId("primary-action")).toContainText("acute bilirubin encephalopathy");
});

test("unknown danger sign blocks reassurance and is reported", async ({ page }) => {
  await fillWholeAssessment(page, { encephalopathy: "Unknown" });
  await submitAssessment(page);
  await expect(page.getByTestId("primary-action")).toContainText("danger-sign assessment");
  await expect(
    page.getByText("/clinical_features/acute_bilirubin_encephalopathy", { exact: false }),
  ).toBeVisible();
});

test("focus moves to the result heading after a successful evaluation", async ({ page }) => {
  await fillWholeAssessment(page);
  await submitAssessment(page);
  await expect(page.getByRole("heading", { name: "Evaluation result" })).toBeFocused();
});

test("clear requires confirmation and erases the assessment", async ({ page }) => {
  await fillWholeAssessment(page);
  await submitAssessment(page);
  await page.getByRole("button", { name: "Clear assessment" }).click();
  await page.getByTestId("confirm-clear").click();
  await expect(page.getByRole("heading", { name: "Neonatal jaundice assessment" })).toBeVisible();
  await expect(page.getByLabel("Gestational age at birth")).toHaveValue("");
});

test("measurements can be added and removed", async ({ page }) => {
  await page.goto("/assessment");
  await page.getByLabel("Gestational age at birth").fill("38");
  await page.getByTestId("continue").click();
  await page.getByTestId("continue").click();
  await page.getByTestId("continue").click();
  await page.getByRole("button", { name: "Add a bilirubin result" }).click();
  await page.getByRole("button", { name: "Add a bilirubin result" }).click();
  await expect(page.getByRole("group", { name: /Result \d/ })).toHaveCount(2);
  await page.getByRole("button", { name: "Remove result 1" }).click();
  await expect(page.getByRole("group", { name: /Result \d/ })).toHaveCount(1);
});
