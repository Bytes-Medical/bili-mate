"use client";

// Result presentation in the normative order from spec 06. Priorities,
// relations and rounding are rendered exactly as the server returned them
// (WEB-011, WEB-014); the result is styled as a formal clinical document
// and prints as the non-identifying receipt (WEB-022).

import { useEffect, useRef, useState } from "react";

import type {
  EvaluationRequest,
  EvaluationResponse,
  Recommendation,
  ThresholdCurve,
} from "@/lib/api/client";
import { formatElapsed } from "@/lib/time";

import ThresholdChart from "./ThresholdChart";
import ThresholdTable from "./ThresholdTable";

function WarningTriangle() {
  return (
    <svg className="warning-icon" viewBox="0 0 24 24" aria-hidden="true">
      <path d="M12 2 23 21 H1 Z" fill="none" stroke="currentColor" strokeWidth={2} strokeLinejoin="round" />
      <line x1="12" y1="9" x2="12" y2="15" stroke="currentColor" strokeWidth={2.5} />
      <circle cx="12" cy="18" r="1.4" fill="currentColor" />
    </svg>
  );
}

function timeframeText(timeframe: Recommendation["timeframe"]): string | null {
  if (!timeframe) return null;
  const unit = timeframe.unit === "hours" ? "hours" : "minutes";
  if (timeframe.exact !== undefined) return `every ${timeframe.exact} ${unit}`;
  if (timeframe.minimum !== undefined && timeframe.maximum !== undefined)
    return `${timeframe.minimum} to ${timeframe.maximum} ${unit}`;
  if (timeframe.maximum !== undefined) return `within ${timeframe.maximum} ${unit}`;
  if (timeframe.minimum !== undefined) return `after ${timeframe.minimum} ${unit}`;
  return null;
}

function relationWord(relation: string): string {
  switch (relation) {
    case "below":
      return "Below";
    case "at":
      return "Exactly at";
    case "above":
      return "Above";
    default:
      return "Not available";
  }
}

function signedValue(value: number | null | undefined): string {
  if (value === null || value === undefined) return "—";
  return `${value > 0 ? "+" : ""}${value.toFixed(1)}`;
}

function RecommendationCard({ recommendation }: { recommendation: Recommendation }) {
  const timeframe = timeframeText(recommendation.timeframe);
  return (
    <article className="recommendation" data-priority={recommendation.priority}>
      <p className="rec-code">
        {recommendation.code} · priority: {recommendation.priority}
        {recommendation.requires_serum_confirmation && " · requires serum confirmation"}
      </p>
      <p className="rec-action">
        {recommendation.action}
        {timeframe && (
          <>
            {" "}
            <strong className="mono">({timeframe})</strong>
          </>
        )}
      </p>
      <p className="small muted" style={{ marginBottom: 0, maxWidth: "none" }}>
        {recommendation.rationale}{" "}
        {recommendation.source_refs.map((ref) => (
          <a key={ref.reference} href={ref.url} style={{ marginRight: "0.5rem" }}>
            NICE CG98 {ref.reference}
          </a>
        ))}
      </p>
    </article>
  );
}

export default function ResultView({
  response,
  request,
  curve,
  onPrint,
  onClear,
}: {
  response: EvaluationResponse;
  request: EvaluationRequest;
  curve: ThresholdCurve | null;
  onPrint: () => void;
  onClear: () => void;
}) {
  const headingRef = useRef<HTMLHeadingElement>(null);
  const [confirmingClear, setConfirmingClear] = useState(false);

  useEffect(() => {
    headingRef.current?.focus();
  }, []);

  const primary = response.primary_action;
  const emergencyLevel = primary.priority === "emergency" || primary.priority === "immediate";
  const supporting = response.recommendations.filter((r) => r.code !== primary.code);
  const assayWarning = response.warnings.find((w) => w.code === "LOCAL_PATHOLOGY_ASSAY_WARNING");
  const otherWarnings = response.warnings.filter((w) => w.code !== "LOCAL_PATHOLOGY_ASSAY_WARNING");
  const tcbIneligible = response.thresholds.some((t) => !t.treatment_decision_eligible);

  return (
    <section aria-labelledby="result-heading">
      <h2 id="result-heading" ref={headingRef} tabIndex={-1}>
        Evaluation result
      </h2>

      <div className="result-document">
        {/* 1. Mode and professional-use status */}
        <div className="result-section">
          <p className="mono small" style={{ margin: 0 }} data-testid="result-mode">
            Mode: {response.mode}
            {response.mode === "demonstration" && " — not for patient care"} · professional use
            only · requires clinician review
          </p>
        </div>

        {/* 2. Emergency or immediate banner (WEB-012) */}
        {emergencyLevel && (
          <div
            className={`priority-banner priority-${primary.priority}`}
            role="alert"
            data-testid="priority-banner"
          >
            <WarningTriangle />
            <div>
              <span className="priority-label">{primary.priority}</span>
              <br />
              {primary.priority === "emergency"
                ? "Emergency — act now and escalate immediately."
                : "Immediate escalation required."}
            </div>
          </div>
        )}

        {/* 3. Primary action */}
        <div className="result-section">
          <p className="eyebrow">Primary action</p>
          <p style={{ fontSize: "1.125rem", fontWeight: 600, maxWidth: "none" }} data-testid="primary-action">
            {primary.action}
          </p>
          <p className="mono small muted" style={{ margin: 0 }}>
            {primary.code} · priority: {primary.priority}
            {timeframeText(primary.timeframe) && ` · ${timeframeText(primary.timeframe)}`}
          </p>
        </div>

        {/* 4. Serum confirmation status (WEB-013) */}
        {(primary.requires_serum_confirmation || tcbIneligible) && (
          <div className="result-section">
            <p className="notice" style={{ margin: 0 }} data-testid="serum-confirmation">
              {tcbIneligible &&
                "A transcutaneous result cannot support a definitive treatment decision. "}
              {primary.requires_serum_confirmation
                ? "Confirm with a serum bilirubin measurement before acting on the treatment line."
                : "Use serum bilirubin for any treatment decision."}
            </p>
          </div>
        )}

        {/* 5. Threshold relationships and signed distances */}
        {response.thresholds.length > 0 && (
          <div className="result-section">
            <p className="eyebrow">Threshold comparison (server-calculated)</p>
            <div
              className="table-scroll"
              tabIndex={0}
              role="region"
              aria-label="Threshold comparison table (scrollable)"
            >
              <table className="data-table" data-testid="threshold-assessments">
                <thead>
                  <tr>
                    <th scope="col">Result</th>
                    <th scope="col">Age</th>
                    <th scope="col">Phototherapy line</th>
                    <th scope="col" className="numeric">
                      Distance
                    </th>
                    <th scope="col">Exchange line</th>
                    <th scope="col" className="numeric">
                      Distance
                    </th>
                    <th scope="col">Decision-eligible</th>
                  </tr>
                </thead>
                <tbody>
                  {response.thresholds.map((t) => (
                    <tr key={t.measurement_id}>
                      <td className="mono">{t.measurement_id}</td>
                      <td>{formatElapsed(t.age_minutes)}</td>
                      <td>
                        {relationWord(t.phototherapy_relation)}
                        {t.phototherapy_threshold_umol_l !== null &&
                          ` (line ${t.phototherapy_threshold_umol_l.toFixed(1)})`}
                      </td>
                      <td className="numeric">{signedValue(t.phototherapy_distance_umol_l)}</td>
                      <td>
                        {relationWord(t.exchange_relation)}
                        {t.exchange_threshold_umol_l !== null &&
                          ` (line ${t.exchange_threshold_umol_l.toFixed(1)})`}
                      </td>
                      <td className="numeric">{signedValue(t.exchange_distance_umol_l)}</td>
                      <td>{t.treatment_decision_eligible ? "Yes (serum)" : "No (transcutaneous)"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}

        {/* 6. Serial trend */}
        {response.trend && (
          <div className="result-section" data-testid="trend">
            <p className="eyebrow">Serial trend</p>
            <p style={{ margin: 0 }}>
              <span className="value">{response.trend.rate_umol_l_per_hour.toFixed(1)} µmol/L per hour</span>{" "}
              ({response.trend.direction}) over {formatElapsed(response.trend.interval_minutes)}
              {" — "}
              {response.trend.reliable_for_rapid_rise
                ? "both results are serum, so this pair can confirm the rapid-rise rule"
                : "this pair includes a transcutaneous result and cannot confirm the rapid-rise rule"}
              .
            </p>
          </div>
        )}

        {/* 7. Supporting recommendations */}
        {supporting.length > 0 && (
          <div className="result-section">
            <p className="eyebrow">Supporting recommendations</p>
            {supporting.map((recommendation) => (
              <RecommendationCard key={recommendation.code} recommendation={recommendation} />
            ))}
          </div>
        )}

        {/* 8. Missing information and limitations */}
        {(response.missing_information.length > 0 ||
          response.suppressed_rules.length > 0 ||
          otherWarnings.length > 0) && (
          <div className="result-section">
            <p className="eyebrow">Missing information and limitations</p>
            {response.missing_information.map((item) => (
              <p key={item.pointer} className="small" style={{ maxWidth: "none" }}>
                <span className="mono">{item.pointer}</span> — {item.impact}
              </p>
            ))}
            {otherWarnings.map((warning) => (
              <p key={warning.code} className="small" style={{ maxWidth: "none" }}>
                <strong>{warning.category}:</strong> {warning.message}
              </p>
            ))}
            {response.suppressed_rules.length > 0 && (
              <p className="small muted" style={{ maxWidth: "none" }}>
                Suppressed by the primary action:{" "}
                <span className="mono">{response.suppressed_rules.join(", ")}</span>
              </p>
            )}
          </div>
        )}

        {/* 9. Chart with textual equivalent */}
        {curve && (
          <div className="result-section">
            <p className="eyebrow">Threshold chart</p>
            <ThresholdChart
              curve={curve}
              measurements={request.measurements}
              assessments={response.thresholds}
            />
            <ThresholdTable curve={curve} />
          </div>
        )}

        {/* 10. Assay warning */}
        {assayWarning && (
          <div className="result-section">
            <p className="notice" style={{ margin: 0 }} data-testid="assay-warning">
              {assayWarning.message}
            </p>
          </div>
        )}

        {/* 11. Rule pack, sources, receipt and legal (WEB-016, WEB-017) */}
        <div className="result-section">
          <p className="eyebrow">Source and receipt</p>
          <p className="mono small" data-testid="rule-pack-line">
            Rule pack {response.rule_pack.id} · source updated {response.rule_pack.source_updated_on} ·
            engine {response.engine_version} · API {response.api_version}
          </p>
          <p className="receipt-foot">
            Evaluation {response.evaluation_id} · receipt digest {response.decision_receipt.digest_sha256}{" "}
            ({response.decision_receipt.canonicalisation}) · not retained by the server
          </p>
          <p className="small" style={{ maxWidth: "none" }}>
            {response.legal.nice_attribution} {response.legal.non_endorsement}{" "}
            {response.legal.professional_use_warning}
          </p>
        </div>
      </div>

      {/* 12. Print and clear (WEB-010, WEB-022) */}
      <p className="no-print" style={{ marginTop: "1rem" }}>
        <button type="button" className="btn btn-secondary" onClick={onPrint}>
          Print receipt
        </button>{" "}
        {confirmingClear ? (
          <>
            <button type="button" className="btn" onClick={onClear} data-testid="confirm-clear">
              Confirm: clear everything
            </button>{" "}
            <button type="button" className="btn btn-secondary" onClick={() => setConfirmingClear(false)}>
              Keep the assessment
            </button>
          </>
        ) : (
          <button type="button" className="btn btn-secondary" onClick={() => setConfirmingClear(true)}>
            Clear assessment
          </button>
        )}
      </p>
    </section>
  );
}
