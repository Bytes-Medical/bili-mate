"use client";

// The single transient clinical assessment (spec 06): a step-based form
// with a persistent review summary. All clinical state lives in this
// component's memory and is lost on clear, navigation or reload — by
// design (WEB-018). No clinical input ever appears in a URL.

import { useCallback, useEffect, useRef, useState } from "react";

import AckGate from "@/components/AckGate";
import FailureState, { type FailureKind } from "@/components/FailureState";
import MeasurementsEditor from "@/components/MeasurementsEditor";
import ResultView from "@/components/ResultView";
import ReviewSummary from "@/components/ReviewSummary";
import TimestampField from "@/components/TimestampField";
import TreatmentStateEditor from "@/components/TreatmentStateEditor";
import TriStateField from "@/components/TriStateField";
import { hasAcknowledged } from "@/lib/ack";
import {
  apiClient,
  evaluate,
  type EvaluationRequest,
  type EvaluationResponse,
  type RulePackMetadata,
  type ThresholdCurve,
} from "@/lib/api/client";
import {
  buildRequest,
  deriveAges,
  FEATURE_FIELDS,
  initialFormState,
  pointerToField,
  RISK_FIELDS,
  type FieldIssue,
  type FormState,
} from "@/lib/form";
import { defaultZone, formatElapsed, zoneOptions } from "@/lib/time";

const STEPS = [
  "Birth details",
  "Recognition and clinical state",
  "Risk factors",
  "Bilirubin results",
  "Treatment state",
  "Review and check",
] as const;

type Phase = "form" | "submitting" | "result" | "failure";

export default function AssessmentPage() {
  const [acknowledged, setAcknowledged] = useState<boolean | null>(null);
  const [form, setForm] = useState<FormState | null>(null);
  const [step, setStep] = useState(0);
  const [pack, setPack] = useState<RulePackMetadata | null>(null);
  const [packFailed, setPackFailed] = useState(false);
  const [issues, setIssues] = useState<FieldIssue[]>([]);
  const [phase, setPhase] = useState<Phase>("form");
  const [failure, setFailure] = useState<FailureKind | null>(null);
  const [result, setResult] = useState<{
    response: EvaluationResponse;
    request: EvaluationRequest;
  } | null>(null);
  const [curve, setCurve] = useState<ThresholdCurve | null>(null);
  const errorSummaryRef = useRef<HTMLDivElement>(null);

  const fetchMetadata = useCallback(async () => {
    setPackFailed(false);
    try {
      const { data } = await apiClient.GET("/v1/guidelines/active");
      if (data) {
        setPack(data);
      } else {
        setPackFailed(true);
      }
    } catch {
      setPackFailed(true);
    }
  }, []);

  useEffect(() => {
    setAcknowledged(hasAcknowledged());
    setForm(initialFormState(defaultZone()));
    void fetchMetadata();
  }, [fetchMetadata]);

  const patch = (partial: Partial<FormState>) =>
    setForm((current) => (current ? { ...current, ...partial } : current));

  const clearAll = () => {
    setForm(initialFormState(defaultZone()));
    setStep(0);
    setIssues([]);
    setResult(null);
    setCurve(null);
    setFailure(null);
    setPhase("form");
    window.scrollTo(0, 0);
  };

  const submit = async () => {
    if (!form || !pack) return;
    const built = buildRequest(form, pack.id);
    if (!built.request) {
      setIssues(built.issues);
      requestAnimationFrame(() => errorSummaryRef.current?.focus());
      return;
    }
    setIssues([]);
    setPhase("submitting");
    const outcome = await evaluate(built.request);
    switch (outcome.kind) {
      case "ok": {
        setResult({ response: outcome.response, request: built.request });
        setPhase("result");
        try {
          const { data } = await apiClient.GET("/v1/threshold-curves/{rule_pack_id}", {
            params: {
              path: { rule_pack_id: outcome.response.rule_pack.id },
              query: {
                gestational_age_completed_weeks:
                  built.request.gestational_age_completed_weeks,
              },
            },
          });
          setCurve(data ?? null);
        } catch {
          setCurve(null);
        }
        break;
      }
      case "stale_pack": {
        await fetchMetadata();
        setFailure({ kind: "stale_pack", activeRulePackId: outcome.activeRulePackId });
        setPhase("failure");
        break;
      }
      case "validation": {
        const fieldIssues: FieldIssue[] = (outcome.problem.errors ?? []).map((error) => ({
          field: pointerToField(error.pointer),
          message: `${error.message} (${error.pointer})`,
        }));
        setIssues(
          fieldIssues.length > 0
            ? fieldIssues
            : [{ field: "form", message: "The service rejected this assessment. Check every field and try again." }],
        );
        setPhase("form");
        requestAnimationFrame(() => errorSummaryRef.current?.focus());
        break;
      }
      case "rate_limited":
        setFailure({ kind: "rate_limited", retryAfterSeconds: outcome.retryAfterSeconds });
        setPhase("failure");
        break;
      default:
        setFailure({ kind: outcome.kind });
        setPhase("failure");
        break;
    }
  };

  if (acknowledged === null || form === null) {
    return <p>Loading…</p>;
  }

  if (!acknowledged) {
    return (
      <>
        <p className="eyebrow">Assessment</p>
        <h1>Professional confirmation needed</h1>
        <AckGate onAcknowledged={() => setAcknowledged(true)} />
      </>
    );
  }

  if (phase === "result" && result) {
    return (
      <ResultView
        response={result.response}
        request={result.request}
        curve={curve}
        onPrint={() => window.print()}
        onClear={clearAll}
      />
    );
  }

  if (phase === "failure" && failure) {
    return (
      <FailureState
        failure={failure}
        onRetry={() => {
          setFailure(null);
          setPhase("form");
          setStep(STEPS.length - 1);
        }}
        onClear={clearAll}
      />
    );
  }

  const derived = deriveAges(form);
  const issueFor = (field: string) => issues.find((issue) => issue.field === field)?.message;
  const issueMap = new Map(issues.map((issue) => [issue.field, issue.message]));

  return (
    <>
      <p className="eyebrow">Assessment — single transient episode</p>
      <h1>Neonatal jaundice assessment</h1>
      {pack ? (
        <p className="mono small muted">
          Active rule pack {pack.id} · source updated {pack.source_updated_on}
        </p>
      ) : packFailed ? (
        <div className="failure-panel" role="alert">
          <h2>The clinical service cannot be reached</h2>
          <p>
            Guideline metadata could not be loaded, so no assessment can be evaluated. Use your
            locally approved procedure.
          </p>
          <button type="button" className="btn" onClick={() => void fetchMetadata()}>
            Try again
          </button>
        </div>
      ) : (
        <p className="muted">Loading guideline metadata…</p>
      )}

      <ol className="step-list no-print">
        {STEPS.map((title, index) => (
          <li key={title} aria-current={index === step ? "step" : undefined}>
            <span className="step-number">{index + 1}</span>
            {title}
          </li>
        ))}
      </ol>

      {issues.length > 0 && (
        <div
          className="failure-panel no-print"
          ref={errorSummaryRef}
          tabIndex={-1}
          role="alert"
          data-testid="error-summary"
          style={{ marginBottom: "1.5rem" }}
        >
          <h2 style={{ fontSize: "1.125rem" }}>Before this assessment can be evaluated</h2>
          <ul>
            {issues.map((issue, index) => (
              <li key={`${issue.field}-${index}`}>{issue.message}</li>
            ))}
          </ul>
        </div>
      )}

      <div className="assessment-grid">
        <form
          onSubmit={(event) => {
            event.preventDefault();
            if (step < STEPS.length - 1) {
              setStep(step + 1);
              window.scrollTo(0, 0);
            } else {
              void submit();
            }
          }}
        >
          {step === 0 && (
            <fieldset style={{ border: "none", padding: 0, margin: 0 }}>
              <legend className="eyebrow">Step 1 — Birth details (kept in this browser)</legend>
              <div className="field">
                <label htmlFor="gestation">Gestational age at birth</label>
                <p className="field-hint">
                  Completed weeks at birth, 23 through 42. Do not round up for extra days, and do
                  not use corrected gestation.
                </p>
                <span className="input-suffix">
                  <input
                    id="gestation"
                    type="number"
                    inputMode="numeric"
                    min={23}
                    max={42}
                    step={1}
                    value={form.gestationWeeks}
                    onChange={(event) => patch({ gestationWeeks: event.target.value })}
                  />
                  <span className="suffix">completed weeks</span>
                </span>
                {issueFor("gestation") && <p className="field-error">{issueFor("gestation")}</p>}
              </div>
              <div className="field">
                <label htmlFor="zone">Timezone for all dates and times</label>
                <select
                  id="zone"
                  value={form.zone}
                  onChange={(event) => patch({ zone: event.target.value })}
                >
                  {zoneOptions().map((zone) => (
                    <option key={zone} value={zone}>
                      {zone}
                    </option>
                  ))}
                </select>
              </div>
              <TimestampField
                id="birth"
                label="Birth date and time"
                value={form.birth}
                error={issueFor("birth")}
                onChange={(birth) => patch({ birth })}
              />
              <TimestampField
                id="assessment"
                label="Assessment date and time"
                value={form.assessment}
                error={issueFor("assessment")}
                onChange={(assessment) => patch({ assessment })}
              />
              <p className="panel panel-filled mono" data-testid="derived-age-inline">
                Derived age at assessment:{" "}
                {derived.assessmentAgeMinutes !== null
                  ? formatElapsed(derived.assessmentAgeMinutes)
                  : "enter birth and assessment times"}
              </p>
              {derived.issues.map((issue) => (
                <p key={issue.field + issue.message} className="field-error">
                  {issue.message}
                </p>
              ))}
            </fieldset>
          )}

          {step === 1 && (
            <fieldset style={{ border: "none", padding: 0, margin: 0 }}>
              <legend className="eyebrow">Step 2 — Recognition and clinical state</legend>
              <p className="field-hint">
                Answer every question. “Unknown” is recorded as missing information and is treated
                differently from “absent”.
              </p>
              {FEATURE_FIELDS.map(([key, label, hint]) => (
                <TriStateField
                  key={key}
                  id={key}
                  label={label}
                  hint={hint}
                  value={form.features[key]}
                  error={issueFor(key)}
                  onChange={(value) =>
                    patch({ features: { ...form.features, [key]: value } })
                  }
                />
              ))}
            </fieldset>
          )}

          {step === 2 && (
            <fieldset style={{ border: "none", padding: 0, margin: 0 }}>
              <legend className="eyebrow">Step 3 — Risk factors</legend>
              {RISK_FIELDS.map(([key, label, hint]) => (
                <TriStateField
                  key={key}
                  id={key}
                  label={label}
                  hint={hint}
                  value={form.risks[key]}
                  error={issueFor(key)}
                  onChange={(value) => patch({ risks: { ...form.risks, [key]: value } })}
                />
              ))}
            </fieldset>
          )}

          {step === 3 && (
            <fieldset style={{ border: "none", padding: 0, margin: 0 }}>
              <legend className="eyebrow">Step 4 — Bilirubin results</legend>
              <MeasurementsEditor
                measurements={form.measurements}
                errors={issueMap}
                onChange={(measurements) => patch({ measurements })}
              />
              <div className="field rule-above">
                <label htmlFor="conjugated">Conjugated bilirubin (if measured)</label>
                <p className="field-hint">
                  Optional; used for the prolonged-jaundice assessment. Whole number in µmol/L.
                </p>
                <span className="input-suffix">
                  <input
                    id="conjugated"
                    type="number"
                    inputMode="numeric"
                    min={0}
                    max={1000}
                    step={1}
                    value={form.conjugated}
                    onChange={(event) => patch({ conjugated: event.target.value })}
                  />
                  <span className="suffix">µmol/L</span>
                </span>
                {issueFor("conjugated") && <p className="field-error">{issueFor("conjugated")}</p>}
              </div>
            </fieldset>
          )}

          {step === 4 && (
            <fieldset style={{ border: "none", padding: 0, margin: 0 }}>
              <legend className="eyebrow">Step 5 — Treatment state</legend>
              <TreatmentStateEditor state={form} errors={issueMap} onChange={patch} />
            </fieldset>
          )}

          {step === 5 && (
            <div>
              <p className="eyebrow">Step 6 — Review and check thresholds</p>
              <p>
                Check the summary before requesting evaluation. The server calculates thresholds
                and recommendations from the current rule pack; nothing is calculated in this
                browser, and the server keeps no record after it responds.
              </p>
              <div className="notice" role="note">
                The result is advisory. Review it against the baby, local pathology advice and
                local policy before any action.
              </div>
            </div>
          )}

          <p style={{ marginTop: "1.5rem" }}>
            {step > 0 && (
              <button
                type="button"
                className="btn btn-secondary"
                onClick={() => {
                  setStep(step - 1);
                  window.scrollTo(0, 0);
                }}
              >
                Back
              </button>
            )}{" "}
            <button
              type="submit"
              className="btn"
              disabled={phase === "submitting" || (step === STEPS.length - 1 && !pack)}
              data-testid={step === STEPS.length - 1 ? "check-thresholds" : "continue"}
            >
              {step === STEPS.length - 1
                ? phase === "submitting"
                  ? "Evaluating…"
                  : "Check thresholds and recommendations"
                : "Continue"}
            </button>
          </p>
        </form>
        <ReviewSummary state={form} />
      </div>
    </>
  );
}
