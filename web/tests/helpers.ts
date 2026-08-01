import type { Page } from "@playwright/test";

export const ACK_KEY = "bili-mate.professional-acknowledgement";

/** Pre-acknowledge professional use for the session before any page script
 * runs, so tests can start at the assessment. */
export async function preAcknowledge(page: Page): Promise<void> {
  await page.addInitScript(
    ([key]) => {
      sessionStorage.setItem(key, "acknowledged");
    },
    [ACK_KEY],
  );
}

async function setTriState(page: Page, field: string, value: "Present" | "Absent" | "Unknown") {
  await page.locator(`#field-${field}`).getByLabel(value, { exact: true }).check();
}

export interface FlowOptions {
  encephalopathy?: "Present" | "Absent" | "Unknown";
  bilirubinValue?: string;
  skipMeasurement?: boolean;
}

/** Fill the whole assessment mirroring the normal-below-threshold fixture:
 * 38 weeks, birth to assessment exactly 48 hours (no DST in July), one serum
 * result of 180 µmol/L collected at the assessment time. */
export async function fillWholeAssessment(page: Page, options: FlowOptions = {}): Promise<void> {
  await page.goto("/assessment");
  await page.getByLabel("Gestational age at birth").fill("38");
  await page.getByLabel("Timezone for all dates and times").selectOption("Europe/London");
  await page.getByLabel("Birth date and time date").fill("2026-07-28");
  await page.getByLabel("Birth date and time time (24 hour)").fill("12:00");
  await page.getByLabel("Assessment date and time date").fill("2026-07-30");
  await page.getByLabel("Assessment date and time time (24 hour)").fill("12:00");
  await page.getByTestId("continue").click();

  // Step 2 — recognition and clinical state.
  await setTriState(page, "suspected_or_obvious_jaundice", "Present");
  await setTriState(page, "visible_jaundice", "Present");
  await setTriState(page, "clinically_well", "Present");
  await setTriState(page, "acute_bilirubin_encephalopathy", options.encephalopathy ?? "Absent");
  await setTriState(page, "pale_chalky_stools", "Absent");
  await setTriState(page, "dark_urine_stains_nappy", "Absent");
  await setTriState(page, "rhesus_haemolytic_disease", "Absent");
  await setTriState(page, "abo_haemolytic_disease", "Absent");
  await setTriState(page, "infection_suspected", "Absent");
  await setTriState(page, "urinary_tract_infection_suspected", "Absent");
  await setTriState(page, "routine_metabolic_screen_completed", "Present");
  await page.getByTestId("continue").click();

  // Step 3 — risk factors.
  await setTriState(page, "previous_sibling_required_phototherapy", "Absent");
  await setTriState(page, "exclusive_breastfeeding_intended", "Absent");
  await page.getByTestId("continue").click();

  // Step 4 — measurements.
  if (!options.skipMeasurement) {
    await page.getByRole("button", { name: "Add a bilirubin result" }).click();
    await page.getByLabel("Collection date and time date").fill("2026-07-30");
    await page.getByLabel("Collection date and time time (24 hour)").fill("12:00");
    await page.getByLabel("Total bilirubin").fill(options.bilirubinValue ?? "180");
    await page.getByLabel("Serum (laboratory blood test)").check();
  }
  await page.getByTestId("continue").click();

  // Step 5 — treatment state ("none" is preselected).
  await page.getByTestId("continue").click();
}

export async function submitAssessment(page: Page): Promise<void> {
  await page.getByTestId("check-thresholds").click();
}
