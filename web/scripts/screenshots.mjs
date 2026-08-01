// Capture review screenshots of the key screens (engineering aid; not part
// of the test suite). Requires the static export served on :3100 and the
// API on :18099.

import { chromium } from "@playwright/test";

const outDir = process.argv[2] ?? "screenshots";
const ACK_KEY = "bili-mate.professional-acknowledgement";

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
await page.addInitScript(([key]) => sessionStorage.setItem(key, "acknowledged"), [ACK_KEY]);

async function fillAssessment(options = {}) {
  await page.goto("http://localhost:3100/assessment");
  await page.getByLabel("Gestational age at birth").fill("38");
  await page.getByLabel("Timezone for all dates and times").selectOption("Europe/London");
  await page.getByLabel("Birth date and time date").fill("2026-07-28");
  await page.getByLabel("Birth date and time time (24 hour)").fill("12:00");
  await page.getByLabel("Assessment date and time date").fill("2026-07-30");
  await page.getByLabel("Assessment date and time time (24 hour)").fill("12:00");
  await page.screenshot({ path: `${outDir}/step1-birth-details.png`, fullPage: true });
  await page.getByTestId("continue").click();
  const tri = async (field, value) =>
    page.locator(`#field-${field}`).getByLabel(value, { exact: true }).check();
  await tri("suspected_or_obvious_jaundice", "Present");
  await tri("visible_jaundice", "Present");
  await tri("clinically_well", "Present");
  await tri("acute_bilirubin_encephalopathy", options.abe ?? "Absent");
  await tri("pale_chalky_stools", "Absent");
  await tri("dark_urine_stains_nappy", "Absent");
  await tri("rhesus_haemolytic_disease", "Absent");
  await tri("abo_haemolytic_disease", "Absent");
  await tri("infection_suspected", "Absent");
  await tri("urinary_tract_infection_suspected", "Absent");
  await tri("routine_metabolic_screen_completed", "Present");
  await page.screenshot({ path: `${outDir}/step2-recognition.png`, fullPage: false });
  await page.getByTestId("continue").click();
  await tri("previous_sibling_required_phototherapy", "Absent");
  await tri("exclusive_breastfeeding_intended", "Absent");
  await page.getByTestId("continue").click();
  await page.getByRole("button", { name: "Add a bilirubin result" }).click();
  await page.getByLabel("Collection date and time date").fill("2026-07-30");
  await page.getByLabel("Collection date and time time (24 hour)").fill("12:00");
  await page.getByLabel("Total bilirubin").fill(options.bilirubin ?? "180");
  await page.getByLabel("Serum (laboratory blood test)").check();
  await page.getByTestId("continue").click();
  await page.getByTestId("continue").click();
  await page.getByTestId("check-thresholds").click();
  await page.getByTestId("primary-action").waitFor();
}

await page.goto("http://localhost:3100/");
await page.screenshot({ path: `${outDir}/home.png`, fullPage: true });

await fillAssessment();
await page.screenshot({ path: `${outDir}/result-routine.png`, fullPage: true });

await fillAssessment({ abe: "Present" });
await page.screenshot({ path: `${outDir}/result-emergency.png`, fullPage: false });

await browser.close();
console.log(`screenshots written to ${outDir}/`);
