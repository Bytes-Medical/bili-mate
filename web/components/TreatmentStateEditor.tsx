"use client";

// Treatment state (spec 03 invariants): mode plus the local times the
// invariants require. Times are converted to elapsed minutes on submission
// and never sent as timestamps.

import type { TreatmentMode } from "@/lib/api/client";
import type { FormState } from "@/lib/form";

import TimestampField from "./TimestampField";

const MODES: { value: TreatmentMode; label: string; hint: string }[] = [
  { value: "none", label: "No phototherapy or exchange transfusion", hint: "The baby is not being treated and has not been treated." },
  { value: "phototherapy", label: "Phototherapy in progress", hint: "Standard phototherapy is currently running." },
  { value: "intensified_phototherapy", label: "Intensified phototherapy in progress", hint: "Intensified (multiple-light) phototherapy is currently running." },
  { value: "post_phototherapy", label: "Phototherapy stopped", hint: "Phototherapy has been given and stopped." },
  { value: "post_exchange", label: "Exchange transfusion completed", hint: "A double-volume exchange transfusion has been completed." },
];

export default function TreatmentStateEditor({
  state,
  errors,
  onChange,
}: {
  state: FormState;
  errors: Map<string, string>;
  onChange: (patch: Partial<FormState>) => void;
}) {
  const mode = state.treatmentMode;
  const needsStart =
    mode === "phototherapy" || mode === "intensified_phototherapy" || mode === "post_phototherapy";
  return (
    <div>
      <fieldset className="tri-field" id="field-treatment">
        <legend>Current treatment state</legend>
        <div>
          {MODES.map((option) => (
            <div key={option.value} style={{ margin: "0.375rem 0" }}>
              <label style={{ display: "flex", gap: "0.625rem", alignItems: "flex-start", minHeight: 44 }}>
                <input
                  type="radio"
                  name="treatment-mode"
                  checked={mode === option.value}
                  onChange={() => onChange({ treatmentMode: option.value })}
                />
                <span>
                  {option.label}
                  <br />
                  <span className="small muted">{option.hint}</span>
                </span>
              </label>
            </div>
          ))}
        </div>
        {errors.get("treatment") && <p className="field-error">{errors.get("treatment")}</p>}
      </fieldset>
      {needsStart && (
        <TimestampField
          id="treatment-started"
          label="Phototherapy start date and time"
          value={state.treatmentStarted}
          error={errors.get("treatment-started")}
          onChange={(treatmentStarted) => onChange({ treatmentStarted })}
        />
      )}
      {mode === "post_phototherapy" && (
        <TimestampField
          id="treatment-stopped"
          label="Phototherapy stop date and time"
          value={state.treatmentStopped}
          error={errors.get("treatment-stopped")}
          onChange={(treatmentStopped) => onChange({ treatmentStopped })}
        />
      )}
      {mode === "post_exchange" && (
        <TimestampField
          id="treatment-exchange"
          label="Exchange transfusion completion date and time"
          value={state.exchangeCompleted}
          error={errors.get("treatment-exchange")}
          onChange={(exchangeCompleted) => onChange({ exchangeCompleted })}
        />
      )}
    </div>
  );
}
