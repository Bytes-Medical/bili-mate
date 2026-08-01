"use client";

// Bilirubin results: local collection time, integer value with a fixed
// µmol/L suffix (WEB-007 — no unit conversion exists), and an explicit,
// unabbreviated measurement method (WEB-008).

import type { MeasurementMethod } from "@/lib/api/client";
import type { FormMeasurement } from "@/lib/form";
import type { LocalTimestamp } from "@/lib/time";

import TimestampField from "./TimestampField";

const METHODS: { value: MeasurementMethod; label: string }[] = [
  { value: "serum", label: "Serum (laboratory blood test)" },
  { value: "transcutaneous", label: "Transcutaneous (skin bilirubinometer)" },
];

export default function MeasurementsEditor({
  measurements,
  errors,
  onChange,
}: {
  measurements: FormMeasurement[];
  errors: Map<string, string>;
  onChange: (measurements: FormMeasurement[]) => void;
}) {
  const update = (key: string, patch: Partial<FormMeasurement>) => {
    onChange(measurements.map((m) => (m.key === key ? { ...m, ...patch } : m)));
  };

  return (
    <div>
      {measurements.length === 0 && (
        <p className="muted">
          No results entered. An assessment without a bilirubin result can still provide
          recognition, measurement-method and prolonged-jaundice support.
        </p>
      )}
      {measurements.map((measurement, index) => (
        <fieldset key={measurement.key} className="panel" data-measurement={index}>
          <legend>
            Result {index + 1}
          </legend>
          <TimestampField
            id={`measurement-${measurement.key}`}
            label="Collection date and time"
            value={measurement.collected}
            onChange={(collected: LocalTimestamp) => update(measurement.key, { collected })}
          />
          <div className="field">
            <label htmlFor={`value-${measurement.key}`}>Total bilirubin</label>
            <p className="field-hint">
              Whole number. This service accepts µmol/L only and never converts units.
            </p>
            <span className="input-suffix">
              <input
                id={`value-${measurement.key}`}
                type="number"
                inputMode="numeric"
                min={0}
                max={1000}
                step={1}
                value={measurement.value}
                onChange={(event) => update(measurement.key, { value: event.target.value })}
              />
              <span className="suffix" aria-hidden="true">
                µmol/L
              </span>
            </span>
          </div>
          <fieldset className="tri-field">
            <legend>Measurement method</legend>
            <div className="tri-options">
              {METHODS.map((method) => (
                <label key={method.value}>
                  <input
                    type="radio"
                    name={`method-${measurement.key}`}
                    checked={measurement.method === method.value}
                    onChange={() => update(measurement.key, { method: method.value })}
                  />
                  {method.label}
                </label>
              ))}
            </div>
          </fieldset>
          {errors.get(`measurement-${measurement.key}`) && (
            <p className="field-error">{errors.get(`measurement-${measurement.key}`)}</p>
          )}
          <button
            type="button"
            className="btn btn-secondary"
            onClick={() => onChange(measurements.filter((m) => m.key !== measurement.key))}
          >
            Remove result {index + 1}
          </button>
        </fieldset>
      ))}
      <button
        type="button"
        className="btn btn-secondary"
        onClick={() =>
          onChange([
            ...measurements,
            {
              key: `k${Date.now()}-${measurements.length}`,
              collected: { date: "", time: "" },
              value: "",
              method: null,
            },
          ])
        }
      >
        Add a bilirubin result
      </button>
    </div>
  );
}
