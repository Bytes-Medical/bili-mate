// Typed API client generated from the committed contract (PRD-010): the
// schema types come from `npm run generate:api` over spec/openapi.yaml.
// The browser performs no clinical calculation and never compares values to
// thresholds locally (PRD-012, WEB-014).

import createClient from "openapi-fetch";
import type { components, paths } from "./schema";

export type EvaluationRequest = components["schemas"]["EvaluationRequest"];
export type EvaluationResponse = components["schemas"]["EvaluationResponse"];
export type RulePackMetadata = components["schemas"]["RulePackMetadata"];
export type ThresholdCurve = components["schemas"]["ThresholdCurve"];
export type LegalNotices = components["schemas"]["LegalNotices"];
export type Problem = components["schemas"]["Problem"];
export type Recommendation = components["schemas"]["Recommendation"];
export type ThresholdAssessment = components["schemas"]["ThresholdAssessment"];
export type Measurement = components["schemas"]["Measurement"];
export type TriState = components["schemas"]["TriState"];
export type TreatmentMode = components["schemas"]["TreatmentMode"];
export type MeasurementMethod = components["schemas"]["MeasurementMethod"];

export const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_BASE_URL ?? "https://api.bili-mate.uk";

export const apiClient = createClient<paths>({ baseUrl: API_BASE_URL });

/** Outcome of a clinical POST, mapped to the states the UI must present
 * (spec 06 failure table). The client never shows a previous result as
 * current and never retries a clinical POST silently. */
export type EvaluationOutcome =
  | { kind: "ok"; response: EvaluationResponse }
  | { kind: "stale_pack"; activeRulePackId: string | null }
  | { kind: "validation"; problem: Problem }
  | { kind: "rate_limited"; retryAfterSeconds: number | null }
  | { kind: "unavailable" }
  | { kind: "network" };

export async function evaluate(request: EvaluationRequest): Promise<EvaluationOutcome> {
  try {
    const { data, error, response } = await apiClient.POST("/v1/evaluations", {
      body: request,
    });
    if (data) {
      return { kind: "ok", response: data };
    }
    const problem = (error ?? null) as Problem | null;
    switch (response.status) {
      case 409:
        return { kind: "stale_pack", activeRulePackId: problem?.active_rule_pack_id ?? null };
      case 422:
      case 400:
        return problem
          ? { kind: "validation", problem }
          : { kind: "unavailable" };
      case 429: {
        const header = response.headers.get("retry-after");
        return { kind: "rate_limited", retryAfterSeconds: header ? Number(header) : null };
      }
      default:
        return { kind: "unavailable" };
    }
  } catch {
    return { kind: "network" };
  }
}
