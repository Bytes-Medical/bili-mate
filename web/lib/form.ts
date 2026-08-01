// Assessment form model. Clinical inputs live only in in-memory component
// state (WEB-018); this module also builds the wire request, which carries
// elapsed minutes and never a timestamp (WEB-004, PRD-016).

import type { DateTime } from "luxon";

import type { EvaluationRequest, MeasurementMethod, TreatmentMode, TriState } from "./api/client";
import { elapsedMinutes, toInstant, type LocalTimestamp, type TimestampIssue } from "./time";

export const FEATURE_FIELDS = [
  ["suspected_or_obvious_jaundice", "Jaundice suspected or obvious", "Jaundice is suspected or clearly visible at this assessment."],
  ["visible_jaundice", "Visible jaundice", "Yellow colouring of the skin, sclerae or gums is visible now."],
  ["clinically_well", "Clinically well", "You assess the baby as clinically well overall."],
  ["acute_bilirubin_encephalopathy", "Features of acute bilirubin encephalopathy", "Signs such as lethargy, abnormal tone, poor feeding, high-pitched cry or seizures."],
  ["pale_chalky_stools", "Pale chalky stools", "Pale or chalky stools reported or observed."],
  ["dark_urine_stains_nappy", "Dark urine staining the nappy", "Dark urine that stains the nappy, reported or observed."],
  ["rhesus_haemolytic_disease", "Rhesus haemolytic disease", "Rhesus haemolytic disease is established for this baby."],
  ["abo_haemolytic_disease", "ABO haemolytic disease", "ABO haemolytic disease is established for this baby."],
  ["infection_suspected", "Infection suspected", "Infection is clinically suspected."],
  ["urinary_tract_infection_suspected", "Urinary tract infection suspected", "A urinary tract infection is clinically suspected."],
  ["routine_metabolic_screen_completed", "Routine metabolic screening completed", "Newborn blood spot screening, including congenital hypothyroidism, is confirmed as completed."],
] as const;

export type FeatureKey = (typeof FEATURE_FIELDS)[number][0];

export const RISK_FIELDS = [
  ["previous_sibling_required_phototherapy", "Previous sibling needed phototherapy", "A previous sibling had neonatal jaundice requiring phototherapy."],
  ["exclusive_breastfeeding_intended", "Exclusive breastfeeding intended", "The mother intends to breastfeed exclusively."],
] as const;

export type RiskKey = (typeof RISK_FIELDS)[number][0];

export interface FormMeasurement {
  key: string;
  collected: LocalTimestamp;
  value: string;
  method: MeasurementMethod | null;
}

export interface FormState {
  gestationWeeks: string;
  zone: string;
  birth: LocalTimestamp;
  assessment: LocalTimestamp;
  features: Record<FeatureKey, TriState | null>;
  risks: Record<RiskKey, TriState | null>;
  measurements: FormMeasurement[];
  conjugated: string;
  treatmentMode: TreatmentMode | null;
  treatmentStarted: LocalTimestamp;
  treatmentStopped: LocalTimestamp;
  exchangeCompleted: LocalTimestamp;
}

const EMPTY_TS: LocalTimestamp = { date: "", time: "" };

export function initialFormState(zone: string): FormState {
  return {
    gestationWeeks: "",
    zone,
    birth: { ...EMPTY_TS },
    assessment: { ...EMPTY_TS },
    features: Object.fromEntries(FEATURE_FIELDS.map(([key]) => [key, null])) as Record<
      FeatureKey,
      TriState | null
    >,
    risks: Object.fromEntries(RISK_FIELDS.map(([key]) => [key, null])) as Record<
      RiskKey,
      TriState | null
    >,
    measurements: [],
    conjugated: "",
    treatmentMode: "none",
    treatmentStarted: { ...EMPTY_TS },
    treatmentStopped: { ...EMPTY_TS },
    exchangeCompleted: { ...EMPTY_TS },
  };
}

export interface FieldIssue {
  field: string;
  message: string;
}

export interface DerivedAges {
  birthInstant: DateTime | null;
  assessmentAgeMinutes: number | null;
  issues: FieldIssue[];
}

function describeIssue(issue: TimestampIssue): string {
  switch (issue) {
    case "incomplete":
      return "Enter both a date and a time.";
    case "invalid":
      return "This date and time is not valid.";
    case "nonexistent_local_time":
      return "This local time does not exist in the selected timezone because the clocks went forward. Correct the time before continuing.";
  }
}

/** Derive the assessment age. Negative ages and future measurement times
 * must be corrected before submission (spec 06). */
export function deriveAges(state: FormState): DerivedAges {
  const issues: FieldIssue[] = [];
  const birth = toInstant(state.birth, state.zone);
  if (birth.issue) {
    issues.push({ field: "birth", message: describeIssue(birth.issue) });
  }
  const assessment = toInstant(state.assessment, state.zone);
  if (assessment.issue) {
    issues.push({ field: "assessment", message: describeIssue(assessment.issue) });
  }
  let assessmentAgeMinutes: number | null = null;
  if (birth.instant && assessment.instant) {
    const minutes = elapsedMinutes(birth.instant, assessment.instant);
    if (minutes < 0) {
      issues.push({ field: "assessment", message: "The assessment time is before the birth time." });
    } else if (minutes > 40319) {
      issues.push({
        field: "assessment",
        message: "The baby is 28 days or older; Bili Mate supports birth to less than 28 days.",
      });
    } else {
      assessmentAgeMinutes = minutes;
    }
  }
  return { birthInstant: birth.instant, assessmentAgeMinutes, issues };
}

export interface BuiltRequest {
  request: EvaluationRequest | null;
  issues: FieldIssue[];
}

/** Convert the form into the wire request. Only elapsed minutes are sent;
 * no timestamp, name or identifier can be represented (DATA-006). */
export function buildRequest(state: FormState, rulePackId: string): BuiltRequest {
  const issues: FieldIssue[] = [];
  const derived = deriveAges(state);
  issues.push(...derived.issues);

  const gestation = Number(state.gestationWeeks);
  if (!Number.isInteger(gestation) || gestation < 23 || gestation > 42) {
    issues.push({
      field: "gestation",
      message: "Completed gestational weeks must be a whole number from 23 through 42.",
    });
  }

  const features: Partial<Record<FeatureKey, TriState>> = {};
  for (const [key, label] of FEATURE_FIELDS) {
    const value = state.features[key];
    if (value === null) {
      issues.push({ field: key, message: `Select present, absent or unknown for “${label}”.` });
    } else {
      features[key] = value;
    }
  }
  const risks: Partial<Record<RiskKey, TriState>> = {};
  for (const [key, label] of RISK_FIELDS) {
    const value = state.risks[key];
    if (value === null) {
      issues.push({ field: key, message: `Select present, absent or unknown for “${label}”.` });
    } else {
      risks[key] = value;
    }
  }

  const measurements = state.measurements.map((m, index) => {
    const collected = toInstant(m.collected, state.zone);
    let age: number | null = null;
    if (collected.issue) {
      issues.push({ field: `measurement-${m.key}`, message: describeIssue(collected.issue) });
    } else if (derived.birthInstant && collected.instant) {
      age = elapsedMinutes(derived.birthInstant, collected.instant);
      if (age < 0) {
        issues.push({ field: `measurement-${m.key}`, message: "Collection time is before birth." });
        age = null;
      } else if (derived.assessmentAgeMinutes !== null && age > derived.assessmentAgeMinutes) {
        issues.push({
          field: `measurement-${m.key}`,
          message: "Collection time is after the assessment time.",
        });
        age = null;
      }
    }
    const value = Number(m.value);
    if (!Number.isInteger(value) || value < 0 || value > 1000) {
      issues.push({
        field: `measurement-${m.key}`,
        message: "Total bilirubin must be a whole number from 0 through 1,000 µmol/L.",
      });
    }
    if (m.method === null) {
      issues.push({
        field: `measurement-${m.key}`,
        message: "Select serum or transcutaneous for this result.",
      });
    }
    return { source: m, index, age, value };
  });

  const treatmentAge = (
    ts: LocalTimestamp,
    field: string,
    required: boolean,
  ): number | null => {
    if (!ts.date && !ts.time) {
      if (required) {
        issues.push({ field, message: "Enter the date and time for this treatment state." });
      }
      return null;
    }
    const instant = toInstant(ts, state.zone);
    if (instant.issue) {
      issues.push({ field, message: describeIssue(instant.issue) });
      return null;
    }
    if (!derived.birthInstant || !instant.instant) {
      return null;
    }
    const age = elapsedMinutes(derived.birthInstant, instant.instant);
    if (age < 0 || (derived.assessmentAgeMinutes !== null && age > derived.assessmentAgeMinutes)) {
      issues.push({ field, message: "Treatment times must be between birth and the assessment time." });
      return null;
    }
    return age;
  };

  const mode = state.treatmentMode;
  if (mode === null) {
    issues.push({ field: "treatment", message: "Select the current treatment state." });
  }
  const startedRequired = mode === "phototherapy" || mode === "intensified_phototherapy" || mode === "post_phototherapy";
  const started = treatmentAge(state.treatmentStarted, "treatment-started", startedRequired);
  const stopped = treatmentAge(state.treatmentStopped, "treatment-stopped", mode === "post_phototherapy");
  const exchange = treatmentAge(state.exchangeCompleted, "treatment-exchange", mode === "post_exchange");

  let conjugated: number | undefined;
  if (state.conjugated.trim() !== "") {
    const value = Number(state.conjugated);
    if (!Number.isInteger(value) || value < 0 || value > 1000) {
      issues.push({
        field: "conjugated",
        message: "Conjugated bilirubin must be a whole number from 0 through 1,000 µmol/L.",
      });
    } else {
      conjugated = value;
    }
  }

  if (issues.length > 0 || derived.assessmentAgeMinutes === null) {
    return { request: null, issues };
  }

  const request: EvaluationRequest = {
    rule_pack_id: rulePackId,
    gestational_age_completed_weeks: gestation,
    assessment_age_minutes: derived.assessmentAgeMinutes,
    clinical_features: features as EvaluationRequest["clinical_features"],
    risk_factors: risks as EvaluationRequest["risk_factors"],
    measurements: measurements.map((m, i) => ({
      id: `m${i + 1}`,
      age_minutes: m.age ?? 0,
      total_bilirubin_umol_l: m.value,
      method: m.source.method ?? "serum",
    })),
    treatment_state: {
      mode: mode ?? "none",
      ...(startedRequired && started !== null ? { started_age_minutes: started } : {}),
      ...(mode === "post_phototherapy" && stopped !== null ? { stopped_age_minutes: stopped } : {}),
      ...(mode === "post_exchange" && exchange !== null
        ? { exchange_completed_age_minutes: exchange }
        : {}),
    },
    ...(conjugated !== undefined ? { conjugated_bilirubin_umol_l: conjugated } : {}),
  };
  return { request, issues: [] };
}

/** Map an API validation pointer (RFC 6901) to the form field that owns it,
 * for the 422 flow (spec 06: field-level mapping without clearing entries). */
export function pointerToField(pointer: string): string {
  if (pointer.startsWith("/measurements/")) {
    const index = Number(pointer.split("/")[2]);
    return Number.isInteger(index) ? `measurement-index-${index}` : "measurements";
  }
  if (pointer.startsWith("/clinical_features/")) {
    return pointer.split("/")[2] ?? "features";
  }
  if (pointer.startsWith("/risk_factors/")) {
    return pointer.split("/")[2] ?? "risks";
  }
  if (pointer.startsWith("/treatment_state")) {
    return "treatment";
  }
  if (pointer.startsWith("/gestational_age")) {
    return "gestation";
  }
  if (pointer.startsWith("/assessment_age")) {
    return "assessment";
  }
  if (pointer.startsWith("/conjugated")) {
    return "conjugated";
  }
  return "form";
}
