"use client";

// Persistent review summary (spec 06 workflow): entered local times AND the
// derived elapsed ages are both visible before submission (WEB-005).

import { deriveAges, FEATURE_FIELDS, RISK_FIELDS, type FormState } from "@/lib/form";
import { elapsedMinutes, formatElapsed, toInstant } from "@/lib/time";

export default function ReviewSummary({ state }: { state: FormState }) {
  const derived = deriveAges(state);
  const unknownCount =
    FEATURE_FIELDS.filter(([key]) => state.features[key] === "unknown").length +
    RISK_FIELDS.filter(([key]) => state.risks[key] === "unknown").length;
  const unanswered =
    FEATURE_FIELDS.filter(([key]) => state.features[key] === null).length +
    RISK_FIELDS.filter(([key]) => state.risks[key] === null).length;

  return (
    <aside className="summary-rail no-print" aria-label="Assessment summary">
      <p className="eyebrow" style={{ marginTop: 0 }}>
        Assessment summary
      </p>
      <dl>
        <dt>Gestation at birth</dt>
        <dd className="mono">
          {state.gestationWeeks ? `${state.gestationWeeks} completed weeks` : "—"}
        </dd>
        <dt>Timezone</dt>
        <dd className="mono">{state.zone}</dd>
        <dt>Birth (local)</dt>
        <dd className="mono">
          {state.birth.date && state.birth.time ? `${state.birth.date} ${state.birth.time}` : "—"}
        </dd>
        <dt>Assessment (local)</dt>
        <dd className="mono">
          {state.assessment.date && state.assessment.time
            ? `${state.assessment.date} ${state.assessment.time}`
            : "—"}
        </dd>
        <dt>Derived age at assessment</dt>
        <dd className="mono" data-testid="derived-age">
          {derived.assessmentAgeMinutes !== null ? formatElapsed(derived.assessmentAgeMinutes) : "—"}
        </dd>
        <dt>Bilirubin results</dt>
        <dd>
          {state.measurements.length === 0 && "None"}
          {state.measurements.map((m, index) => {
            const collected = toInstant(m.collected, state.zone);
            const age =
              derived.birthInstant && collected.instant
                ? elapsedMinutes(derived.birthInstant, collected.instant)
                : null;
            return (
              <span key={m.key} className="mono" style={{ display: "block" }}>
                {index + 1}. {m.value || "—"} µmol/L{" "}
                {m.method === "serum" ? "serum" : m.method === "transcutaneous" ? "TcB" : "—"}
                {age !== null && age >= 0 ? ` at ${formatElapsed(age)}` : ""}
              </span>
            );
          })}
        </dd>
        <dt>Treatment state</dt>
        <dd className="mono">{state.treatmentMode ?? "—"}</dd>
        <dt>Answers</dt>
        <dd>
          {unanswered > 0
            ? `${unanswered} question${unanswered === 1 ? "" : "s"} unanswered`
            : "All questions answered"}
          {unknownCount > 0 && `, ${unknownCount} recorded as unknown`}
        </dd>
      </dl>
      <p className="small muted rule-above">
        Local dates and times stay in this browser. Only elapsed ages in minutes are sent for
        evaluation.
      </p>
    </aside>
  );
}
