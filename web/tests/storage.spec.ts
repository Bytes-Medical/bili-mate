// Storage discipline (WEB-018, WEB-019, spec 09): clinical inputs and
// results exist only in in-memory state; persistent storage never holds
// clinical data, and a reload loses the assessment by design.

import { expect, test } from "@playwright/test";

import { ACK_KEY, fillWholeAssessment, preAcknowledge, submitAssessment } from "./helpers";

test.beforeEach(async ({ page }) => {
  await preAcknowledge(page);
});

test("no clinical data reaches localStorage, sessionStorage or cookies", async ({ page, context }) => {
  await fillWholeAssessment(page);
  await submitAssessment(page);
  await expect(page.getByTestId("primary-action")).toBeVisible();

  const storage = await page.evaluate((ackKey) => {
    const session: Record<string, string | null> = {};
    for (let i = 0; i < sessionStorage.length; i += 1) {
      const key = sessionStorage.key(i)!;
      session[key] = sessionStorage.getItem(key);
    }
    return {
      localStorageLength: localStorage.length,
      sessionKeys: Object.keys(session),
      ackValue: session[ackKey] ?? null,
    };
  }, ACK_KEY);

  expect(storage.localStorageLength).toBe(0);
  expect(storage.sessionKeys).toEqual([ACK_KEY]);
  expect(storage.ackValue).toBe("acknowledged");

  const cookies = await context.cookies();
  expect(cookies).toEqual([]);
});

test("reloading loses the clinical form by design", async ({ page }) => {
  await page.goto("/assessment");
  await page.getByLabel("Gestational age at birth").fill("38");
  await page.reload();
  await expect(page.getByLabel("Gestational age at birth")).toHaveValue("");
});
