// Failure and stale-data behaviour (spec 06 failure table, TEST-026): the
// client fails closed, keeps entries, and never silently retries a clinical
// POST. Server failures are simulated by intercepting the evaluation route.

import { expect, test } from "@playwright/test";

import { fillWholeAssessment, preAcknowledge, submitAssessment } from "./helpers";

test.beforeEach(async ({ page }) => {
  await preAcknowledge(page);
});

function problemBody(status: number, code: string, extra: Record<string, unknown> = {}) {
  return {
    status,
    contentType: "application/problem+json",
    body: JSON.stringify({
      type: `https://bili-mate.uk/problems/${code.toLowerCase()}`,
      title: code,
      status,
      detail: "Test problem",
      instance: "urn:bili-mate:request:test",
      code,
      ...extra,
    }),
  };
}

test("409 stale rule pack requires review and explicit resubmission", async ({ page }) => {
  let postCount = 0;
  await page.route("**/v1/evaluations", async (route) => {
    postCount += 1;
    if (postCount === 1) {
      await route.fulfill(
        problemBody(409, "RULE_PACK_NOT_ACTIVE", { active_rule_pack_id: "nice-cg98-2027-01-01.1" }),
      );
    } else {
      await route.continue();
    }
  });
  await fillWholeAssessment(page);
  await submitAssessment(page);

  await expect(page.getByRole("heading", { name: "Guidance has changed" })).toBeVisible();
  await expect(page.getByText("nice-cg98-2027-01-01.1")).toBeVisible();
  // No clinical result was shown.
  await expect(page.getByTestId("primary-action")).toHaveCount(0);
  // Only the explicit user action returns to review; nothing auto-retried.
  expect(postCount).toBe(1);
  await page.getByTestId("resubmit").click();
  await expect(page.getByTestId("check-thresholds")).toBeVisible();
});

test("422 maps field pointers into the error summary without clearing entries", async ({ page }) => {
  await page.route("**/v1/evaluations", (route) =>
    route.fulfill(
      problemBody(422, "VALIDATION_FAILED", {
        errors: [
          {
            pointer: "/measurements/0/age_minutes",
            code: "MEASUREMENT_AFTER_ASSESSMENT",
            message: "Measurement age must be no later than the assessment age.",
          },
        ],
      }),
    ),
  );
  await fillWholeAssessment(page);
  await submitAssessment(page);

  const summary = page.getByTestId("error-summary");
  await expect(summary).toBeVisible();
  await expect(summary).toBeFocused();
  await expect(summary).toContainText("/measurements/0/age_minutes");
  // Entries are preserved (spec 06: do not clear entries).
  await page.getByRole("button", { name: "Back" }).click();
  await page.getByRole("button", { name: "Back" }).click();
  await expect(page.getByLabel("Total bilirubin")).toHaveValue("180");
});

test("429 shows the retry time and produces no result", async ({ page }) => {
  await page.route("**/v1/evaluations", (route) =>
    route.fulfill({
      ...problemBody(429, "RATE_LIMITED"),
      headers: {
        "content-type": "application/problem+json",
        "retry-after": "30",
        // Cross-origin responses must expose Retry-After for the browser to
        // read it, exactly as the real API does.
        "access-control-expose-headers": "retry-after, x-request-id",
      },
    }),
  );
  await fillWholeAssessment(page);
  await submitAssessment(page);
  await expect(page.getByTestId("failure-panel")).toContainText("30");
  await expect(page.getByTestId("primary-action")).toHaveCount(0);
});

test("service failure fails closed with the local-protocol direction", async ({ page }) => {
  await page.route("**/v1/evaluations", (route) => route.fulfill(problemBody(503, "ENGINE_UNAVAILABLE")));
  await fillWholeAssessment(page);
  await submitAssessment(page);
  const panel = page.getByTestId("failure-panel");
  await expect(panel).toContainText("No clinical result was produced");
  await expect(panel).toContainText("locally approved procedure");
  await expect(page.getByTestId("primary-action")).toHaveCount(0);
});

test("network failure never shows a previous result as current", async ({ page }) => {
  // First evaluation succeeds.
  await fillWholeAssessment(page);
  await submitAssessment(page);
  await expect(page.getByTestId("primary-action")).toBeVisible();

  // Return to the form, change nothing, and fail the network on resubmit.
  await page.getByRole("button", { name: "Clear assessment" }).click();
  await page.getByTestId("confirm-clear").click();
  await page.route("**/v1/evaluations", (route) => route.abort());
  await fillWholeAssessment(page);
  await submitAssessment(page);
  await expect(page.getByTestId("failure-panel")).toContainText("could not be reached");
  await expect(page.getByTestId("primary-action")).toHaveCount(0);
});
