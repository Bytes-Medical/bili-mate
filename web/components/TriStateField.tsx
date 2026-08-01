"use client";

// Required tri-state question (WEB-009): an explicit present / absent /
// unknown selection with no default. Unknown is data, not absence (PRD-008).

import type { TriState } from "@/lib/api/client";

const OPTIONS: { value: TriState; label: string }[] = [
  { value: "present", label: "Present" },
  { value: "absent", label: "Absent" },
  { value: "unknown", label: "Unknown" },
];

export default function TriStateField({
  id,
  label,
  hint,
  value,
  error,
  onChange,
}: {
  id: string;
  label: string;
  hint: string;
  value: TriState | null;
  error?: string;
  onChange: (value: TriState) => void;
}) {
  return (
    <fieldset className="tri-field" id={`field-${id}`} aria-describedby={`hint-${id}`}>
      <legend>{label}</legend>
      <p className="field-hint" id={`hint-${id}`}>
        {hint}
      </p>
      <div className="tri-options">
        {OPTIONS.map((option) => (
          <label key={option.value}>
            <input
              type="radio"
              name={id}
              value={option.value}
              checked={value === option.value}
              onChange={() => onChange(option.value)}
            />
            {option.label}
          </label>
        ))}
      </div>
      {error && <p className="field-error">{error}</p>}
    </fieldset>
  );
}
