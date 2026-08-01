"use client";

// Local date and time entry. Values stay in browser memory and are never
// submitted (WEB-004); the caller derives elapsed minutes.

import type { LocalTimestamp } from "@/lib/time";

export default function TimestampField({
  id,
  label,
  value,
  error,
  onChange,
}: {
  id: string;
  label: string;
  value: LocalTimestamp;
  error?: string;
  onChange: (value: LocalTimestamp) => void;
}) {
  return (
    <div className="field" id={`field-${id}`}>
      <span className="field-label" id={`label-${id}`}>
        {label}
      </span>
      <div className="timestamp-inputs">
        <label>
          <span className="small muted">Date </span>
          <input
            type="date"
            aria-label={`${label} date`}
            value={value.date}
            onChange={(event) => onChange({ ...value, date: event.target.value })}
          />
        </label>
        <label>
          <span className="small muted">Time </span>
          <input
            type="time"
            aria-label={`${label} time (24 hour)`}
            value={value.time}
            onChange={(event) => onChange({ ...value, time: event.target.value })}
          />
        </label>
      </div>
      {error && <p className="field-error">{error}</p>}
    </div>
  );
}
