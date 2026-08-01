# Reference web application

## Purpose

The reference web application proves that the API contract can support a safe professional workflow. It is not a simplified parent tool. Its behaviour is normative for future native-client safety and presentation requirements.

Implementation stack: React, TypeScript, a generated API client, a schema-aware form library, a client-side temporal library with IANA timezone support, accessible SVG charting, and browser end-to-end tests. Exact package versions are selected from current stable releases and pinned at implementation.

## Information architecture

Routes:

| Route | Purpose |
|---|---|
| `/` | Product introduction and intended-user warning |
| `/assessment` | Single transient clinical assessment |
| `/about` | Intended purpose, sources, rule-pack version, licence and safety information |
| `/privacy` | Data processing and retention notice |
| `/accessibility` | Accessibility statement |
| `/service-status` | Availability and current release metadata, with no patient information |

No clinical input may appear in a URL, query parameter, fragment, page title or browser history state.

## Professional-use acknowledgement

Before entering the assessment, the user must acknowledge:

- they are a healthcare professional using the product in the UK;
- the result does not replace clinical judgement or local policy;
- bilirubin assay variation requires local laboratory advice; and
- the product must not be used by parents to decide treatment.

The acknowledgement may be stored in `sessionStorage` only. It expires when the browser session ends and contains no clinical data.

| ID | Requirement |
|---|---|
| WEB-001 | The assessment route MUST be inaccessible until the session acknowledgement is given. |
| WEB-002 | Acknowledgement MUST NOT be presented as consent for patient-data processing. |
| WEB-003 | The lack of login MUST remain visible through a professional-use banner rather than imply verified clinician identity. |

## Assessment workflow

The assessment is a step-based form with a persistent review summary:

1. **Birth details kept locally**: completed gestational weeks, local birth date/time, assessment date/time and timezone.
2. **Recognition and clinical state**: required tri-state fields with plain definitions.
3. **Risk factors**: previous sibling and feeding intention.
4. **Measurements**: zero or more bilirubin results with local collection time, value and method.
5. **Treatment state**: mode and local start/stop time where relevant.
6. **Review**: elapsed ages, units, methods, unknowns and professional warning.
7. **Result**: primary action, urgent banner where applicable, thresholds, chart, trend, supporting recommendations, warnings, sources and receipt.

Local timestamps are converted to instants using the explicitly displayed IANA timezone. Elapsed minutes are calculated as actual elapsed time, including daylight-saving transitions. Negative ages, future measurement times and ambiguous local times must be corrected before submission.

| ID | Requirement |
|---|---|
| WEB-004 | Birth and collection timestamps MUST remain in volatile browser state and MUST NOT be sent to the API. |
| WEB-005 | The UI MUST show both the entered local time and derived elapsed age before submission. |
| WEB-006 | The UI MUST label gestation as completed weeks and state that it is not corrected gestation. |
| WEB-007 | Bilirubin inputs MUST display a fixed `µmol/L` suffix and MUST NOT offer unit conversion. |
| WEB-008 | Measurement method MUST be explicit for every result and serum/TcB MUST never share an ambiguous abbreviation-only label. |
| WEB-009 | Tri-state questions MUST require an explicit present, absent or unknown selection. |
| WEB-010 | The user MUST be able to clear all assessment data with one confirmed action. |

## Result presentation

Order:

1. mode and professional-use status;
2. emergency/immediate banner if applicable;
3. primary action and timeframe;
4. serum-confirmation status;
5. threshold relationship and signed distance;
6. serial trend and reliability;
7. supporting recommendations;
8. missing information and suppressed-scope limitations;
9. chart with textual equivalent;
10. local assay warning;
11. rule-pack/source information and legal notice;
12. print and clear actions.

| ID | Requirement |
|---|---|
| WEB-011 | The UI MUST render recommendation priority from the response and MUST NOT recalculate it. |
| WEB-012 | Emergency and immediate states MUST use text, iconography, heading hierarchy and live-region announcement in addition to colour. |
| WEB-013 | A TcB result MUST visibly state that it is not eligible for a definitive treatment decision when the API says confirmation is required. |
| WEB-014 | Display rounding MUST use server values; the browser MUST NOT recalculate line values. |
| WEB-015 | The chart and textual threshold table MUST show identical server-provided points. |
| WEB-016 | The current rule-pack ID and source update date MUST be visible without opening a secondary screen. |
| WEB-017 | The full required NICE attribution and non-endorsement statement MUST appear in the result footer and print view. |

## Chart

The chart plots age on the horizontal axis and µmol/L on the vertical axis. It includes phototherapy and exchange lines, measurement markers distinguished by method, and treatment periods where supplied.

- The chart fetches display curves from the server for the selected rule pack and gestation.
- Measurements are plotted locally but never compared locally.
- Each point has an accessible text alternative containing age, value, method and server relationship.
- Lines use different dash patterns as well as colours.
- Zooming must not hide the source, unit or selected gestation.
- A tabular equivalent is available adjacent to the chart.

## Failure and stale-data behaviour

| Condition | Required behaviour |
|---|---|
| Network timeout/offline | State that no result was produced; show local-protocol instruction; allow retry or clear |
| `409` rule-pack conflict | Fetch active metadata, show that guidance changed, require review and explicit resubmission |
| `422` validation | Map field pointers to form errors; do not clear entries |
| `429` | Show retry time; do not repeatedly auto-retry |
| `500/503` | Fail closed; no threshold or previous result displayed as current |
| Browser refresh | Clinical form/result is lost by design; show a privacy explanation before starting |

The application MUST NOT silently retry a clinical `POST`. A user-triggered retry creates a new evaluation ID.

## Storage, caching and telemetry

| ID | Requirement |
|---|---|
| WEB-018 | Clinical inputs and results MUST exist only in in-memory component state. |
| WEB-019 | `localStorage`, IndexedDB, Cache API and cookies MUST NOT contain clinical data. |
| WEB-020 | The service worker MAY cache static shell assets and legal/metadata GET responses but MUST NOT cache evaluation traffic. |
| WEB-021 | Analytics and error reporting MUST exclude form state, URLs containing inputs, DOM text from results, request/response bodies and evaluation IDs. |
| WEB-022 | Print output MUST omit birth date/time and measurement clock times; it may include derived ages, values, rule-pack data and evaluation ID. |

## Accessibility and usability

The application MUST meet WCAG 2.2 AA and be tested with keyboard-only operation, 200% zoom, high-contrast mode and representative screen readers.

- Focus moves to the error summary after invalid submission.
- Focus moves to the result heading after a successful response.
- Dynamic emergency content uses an assertive live region without repeatedly announcing the full result.
- Inputs have visible labels, instructions and error associations.
- Touch targets are at least 24 by 24 CSS pixels, with larger targets preferred for primary clinical actions.
- Motion is non-essential and respects reduced-motion preferences.
- Plain UK English accompanies clinical terminology.

## Browser support

Support the current and previous major versions of Safari on iOS/macOS, Chrome on Android/desktop, Edge and Firefox at the time of release. Unsupported browsers receive an informational gate before the assessment, not a partially functional form.
