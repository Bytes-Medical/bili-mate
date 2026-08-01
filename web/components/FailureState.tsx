"use client";

// Fail-closed presentation (PRD-013, spec 06 failure table): the interface
// states plainly that no clinical result was produced, never shows a
// previous result as current, and directs the clinician to the locally
// approved procedure. Retrying is always an explicit user action.

import { useEffect, useRef } from "react";

export type FailureKind =
  | { kind: "unavailable" }
  | { kind: "network" }
  | { kind: "rate_limited"; retryAfterSeconds: number | null }
  | { kind: "stale_pack"; activeRulePackId: string | null };

export default function FailureState({
  failure,
  onRetry,
  onClear,
}: {
  failure: FailureKind;
  onRetry: () => void;
  onClear: () => void;
}) {
  const headingRef = useRef<HTMLHeadingElement>(null);
  useEffect(() => {
    headingRef.current?.focus();
  }, []);

  return (
    <section className="failure-panel" role="alert" data-testid="failure-panel">
      <h2 ref={headingRef} tabIndex={-1}>
        <svg width="24" height="24" viewBox="0 0 24 24" aria-hidden="true">
          <path d="M12 2 23 21 H1 Z" fill="none" stroke="#000" strokeWidth={2} strokeLinejoin="round" />
          <line x1="12" y1="9" x2="12" y2="15" stroke="#000" strokeWidth={2.5} />
          <circle cx="12" cy="18" r="1.4" fill="#000" />
        </svg>
        {failure.kind === "stale_pack" ? "Guidance has changed" : "No clinical result was produced"}
      </h2>

      {failure.kind === "stale_pack" && (
        <>
          <p>
            The clinical rule pack changed since this assessment was started
            {failure.activeRulePackId && (
              <>
                {" "}
                — the active pack is now <span className="mono">{failure.activeRulePackId}</span>
              </>
            )}
            . No result was produced. Review your entries against the updated guidance, then submit
            again explicitly.
          </p>
          <p>
            <button type="button" className="btn" onClick={onRetry} data-testid="resubmit">
              Review and resubmit
            </button>
          </p>
        </>
      )}

      {failure.kind === "rate_limited" && (
        <>
          <p>
            The service limited this request
            {failure.retryAfterSeconds !== null && (
              <>
                {" "}
                — try again in about <span className="mono">{failure.retryAfterSeconds}</span> seconds
              </>
            )}
            . No result was produced and nothing you entered has been lost.
          </p>
          <p>
            <button type="button" className="btn" onClick={onRetry}>
              Try again
            </button>
          </p>
        </>
      )}

      {(failure.kind === "unavailable" || failure.kind === "network") && (
        <>
          <p>
            {failure.kind === "network"
              ? "The evaluation service could not be reached."
              : "The evaluation service is unavailable."}{" "}
            No threshold or recommendation has been calculated, and no earlier result applies to
            this assessment.
          </p>
          <p>
            <strong>
              Use your locally approved procedure for assessing and managing neonatal jaundice.
            </strong>
          </p>
          <p>
            Your entries are still on this page. You can try again, or clear the assessment.
          </p>
          <p>
            <button type="button" className="btn" onClick={onRetry}>
              Try again
            </button>{" "}
            <button type="button" className="btn btn-secondary" onClick={onClear}>
              Clear assessment
            </button>
          </p>
        </>
      )}
    </section>
  );
}
