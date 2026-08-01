"use client";

// Professional-use acknowledgement gate (WEB-001, WEB-002): the assessment
// is inaccessible until every point is acknowledged for this browser
// session. This is not consent for patient-data processing.

import { useState } from "react";

import { recordAcknowledgement } from "@/lib/ack";

const POINTS = [
  "I am a healthcare professional using Bili Mate in the United Kingdom.",
  "The result does not replace clinical judgement or local policy.",
  "Bilirubin assays vary; I will interpret results with local pathology laboratory advice.",
  "Bili Mate must not be used by parents or carers to make treatment decisions.",
] as const;

export function browserSupported(): boolean {
  return (
    typeof Intl !== "undefined" &&
    typeof Intl.DateTimeFormat().resolvedOptions().timeZone === "string"
  );
}

export default function AckGate({ onAcknowledged }: { onAcknowledged: () => void }) {
  const [checked, setChecked] = useState<boolean[]>(POINTS.map(() => false));
  const allChecked = checked.every(Boolean);

  if (!browserSupported()) {
    return (
      <section className="panel" aria-labelledby="browser-gate-heading">
        <h2 id="browser-gate-heading">This browser is not supported</h2>
        <p>
          Bili Mate needs a current browser with timezone support to derive ages safely. Use a
          current version of Safari, Chrome, Edge or Firefox.
        </p>
      </section>
    );
  }

  return (
    <section className="panel" aria-labelledby="ack-heading">
      <p className="eyebrow">Before you start</p>
      <h2 id="ack-heading">Confirm professional use for this session</h2>
      <p>
        This confirmation lasts for your browser session only and stores no clinical data. It is
        not consent for processing patient data.
      </p>
      {POINTS.map((point, index) => (
        <div className="field" key={point}>
          <label style={{ display: "flex", gap: "0.75rem", alignItems: "flex-start", fontWeight: 400 }}>
            <input
              type="checkbox"
              checked={checked[index]}
              onChange={(event) => {
                const next = [...checked];
                next[index] = event.target.checked;
                setChecked(next);
              }}
            />
            <span>{point}</span>
          </label>
        </div>
      ))}
      <button
        type="button"
        className="btn"
        disabled={!allChecked}
        onClick={() => {
          recordAcknowledgement();
          onAcknowledged();
        }}
      >
        Confirm and continue
      </button>
    </section>
  );
}
