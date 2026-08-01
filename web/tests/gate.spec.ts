// Professional-use gate (WEB-001–WEB-003, spec 09 web list).

import { expect, test } from "@playwright/test";

import { ACK_KEY, preAcknowledge } from "./helpers";

test("assessment is inaccessible until the session acknowledgement is given", async ({ page }) => {
  await page.goto("/assessment");
  await expect(page.getByRole("heading", { name: "Professional confirmation needed" })).toBeVisible();
  await expect(page.getByLabel("Gestational age at birth")).toHaveCount(0);

  const confirm = page.getByRole("button", { name: "Confirm and continue" });
  await expect(confirm).toBeDisabled();
  for (const checkbox of await page.getByRole("checkbox").all()) {
    await checkbox.check();
  }
  await expect(confirm).toBeEnabled();
  await confirm.click();
  await expect(page.getByRole("heading", { name: "Neonatal jaundice assessment" })).toBeVisible();

  // The acknowledgement lives in sessionStorage only and holds no clinical
  // data (WEB-002).
  const stored = await page.evaluate((key) => sessionStorage.getItem(key), ACK_KEY);
  expect(stored).toBe("acknowledged");
  const localStorageLength = await page.evaluate(() => localStorage.length);
  expect(localStorageLength).toBe(0);
});

test("professional-use banner stays visible on every page", async ({ page }) => {
  await preAcknowledge(page);
  for (const path of ["/", "/assessment", "/about"]) {
    await page.goto(path);
    await expect(
      page.getByText("For registered UK healthcare professionals only", { exact: false }),
    ).toBeVisible();
  }
});
