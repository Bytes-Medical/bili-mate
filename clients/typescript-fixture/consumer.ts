/**
 * Minimal consumer fixture for the generated TypeScript client (TEST-017,
 * spec 05 native-client path). Compiled in CI against the client generated
 * from the committed spec/openapi.yaml; never executed against production.
 *
 * Demonstrates: metadata fetch, evaluation request construction, the 409
 * stale-rule-pack refresh flow, fail-closed network handling and display of
 * source/version/legal fields.
 */

import * as client from "../generated/typescript/src/index";

const configuration = new client.Configuration({
  basePath: "https://api.bili-mate.uk",
});

const guidelines = new client.GuidelinesApi(configuration);
const evaluations = new client.EvaluationsApi(configuration);

function buildRequest(rulePackId: string): client.EvaluationRequest {
  return {
    rulePackId,
    gestationalAgeCompletedWeeks: 38,
    assessmentAgeMinutes: 2880,
    clinicalFeatures: {
      suspectedOrObviousJaundice: "present",
      visibleJaundice: "present",
      clinicallyWell: "present",
      acuteBilirubinEncephalopathy: "absent",
      paleChalkyStools: "absent",
      darkUrineStainsNappy: "absent",
      rhesusHaemolyticDisease: "absent",
      aboHaemolyticDisease: "absent",
      infectionSuspected: "absent",
      urinaryTractInfectionSuspected: "absent",
      routineMetabolicScreenCompleted: "present",
    },
    riskFactors: {
      previousSiblingRequiredPhototherapy: "absent",
      exclusiveBreastfeedingIntended: "absent",
    },
    measurements: [
      {
        id: "m1",
        ageMinutes: 2880,
        totalBilirubinUmolL: 180,
        method: "serum",
      },
    ],
    treatmentState: { mode: "none" },
  };
}

export async function runOnce(): Promise<void> {
  // 1. Metadata fetch: clients retrieve the active pack before assessing
  // (spec 04) and display source/version fields.
  const metadata = await guidelines.getActiveGuideline();
  console.log(
    `rule pack ${metadata.id} (${metadata.status}), source updated ${metadata.sourceUpdatedOn}`,
  );
  console.log(metadata.legal.niceAttribution);

  try {
    const response = await evaluations.evaluateJaundiceAssessment({
      evaluationRequest: buildRequest(metadata.id),
    });
    // Display of version and receipt fields.
    console.log(
      `evaluation ${response.evaluationId} via engine ${response.engineVersion}, ` +
        `pack ${response.rulePack.id}: ${response.primaryAction.code}`,
    );
    for (const recommendation of response.recommendations) {
      console.log(`[${recommendation.priority}] ${recommendation.action}`);
    }
  } catch (error) {
    if (error instanceof client.ResponseError && error.response.status === 409) {
      // 2. Stale-pack refresh flow: fetch metadata again and require the
      // clinician to review and explicitly resubmit (spec 04, WEB failure
      // table); never silently retried here.
      const refreshed = await guidelines.getActiveGuideline();
      console.log(`guidance changed; active pack is now ${refreshed.id} - review before resubmitting`);
      return;
    }
    // 3. Fail closed (PRD-013): no cached or previous result is shown; the
    // caller is directed to the locally approved procedure.
    console.log("No clinical result was produced. Follow the locally approved procedure.");
    throw error;
  }
}
