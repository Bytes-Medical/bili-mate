// Local time handling (WEB-004, WEB-005): birth and collection timestamps
// stay in browser memory; only elapsed whole minutes are ever submitted.
// Conversion uses the explicitly displayed IANA timezone and actual elapsed
// time, including daylight-saving transitions (spec 06).

import { DateTime } from "luxon";

export interface LocalTimestamp {
  /** ISO date, e.g. 2026-03-28 */
  date: string;
  /** 24-hour local time, e.g. 14:30 */
  time: string;
}

export type TimestampIssue =
  | "incomplete"
  | "invalid"
  | "nonexistent_local_time";

export interface InstantResult {
  instant: DateTime | null;
  issue: TimestampIssue | null;
}

/** Resolve a local wall-clock timestamp in a zone to an instant. A local
 * time skipped by a DST spring-forward transition does not exist and must
 * be corrected by the clinician rather than silently adjusted (spec 06). */
export function toInstant(value: LocalTimestamp, zone: string): InstantResult {
  if (!value.date || !value.time) {
    return { instant: null, issue: "incomplete" };
  }
  const dt = DateTime.fromISO(`${value.date}T${value.time}`, { zone });
  if (!dt.isValid) {
    return { instant: null, issue: "invalid" };
  }
  // Round-trip check: Luxon maps nonexistent local times forward, so a
  // changed wall clock means the entered time does not exist in this zone.
  const roundTrip = dt.toFormat("yyyy-MM-dd'T'HH:mm");
  if (roundTrip !== `${value.date}T${value.time}`) {
    return { instant: null, issue: "nonexistent_local_time" };
  }
  return { instant: dt, issue: null };
}

/** Elapsed whole minutes between two instants (floor). */
export function elapsedMinutes(birth: DateTime, later: DateTime): number {
  return Math.floor(later.diff(birth, "minutes").minutes);
}

/** Format an elapsed age for display, e.g. "47 h 59 min (2,879 minutes)". */
export function formatElapsed(minutes: number): string {
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  const days = Math.floor(hours / 24);
  const dayPart = days > 0 ? `${days} d ` : "";
  return `${dayPart}${hours % 24} h ${rest} min (${minutes.toLocaleString("en-GB")} minutes)`;
}

/** The browser's IANA zone, displayed explicitly and changeable. */
export function defaultZone(): string {
  return Intl.DateTimeFormat().resolvedOptions().timeZone ?? "Europe/London";
}

/** IANA zones offered in the selector; the browser zone is merged in. */
export function zoneOptions(): string[] {
  const supported =
    typeof Intl.supportedValuesOf === "function"
      ? Intl.supportedValuesOf("timeZone")
      : ["Europe/London"];
  const preferred = ["Europe/London", "Europe/Dublin", "Atlantic/Stanley", "UTC"];
  const rest = supported.filter((z) => !preferred.includes(z));
  return [...preferred.filter((z) => supported.includes(z) || z === "UTC"), ...rest];
}
