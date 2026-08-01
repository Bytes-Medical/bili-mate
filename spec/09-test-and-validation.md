# Test and validation

## Strategy

Testing is layered so a UI or API test never substitutes for proof of clinical mathematics, and unit tests never substitute for clinical workflow validation.

| Layer | Purpose |
|---|---|
| Source verification | Prove normalised data matches approved NICE sources |
| Clinical-core unit/oracle | Prove exact calculations and every decision branch |
| Property and exhaustive tests | Prove invariants over the complete supported domain |
| API contract | Prove wire validation, versioning and safe errors |
| Web integration | Prove form, presentation and failure behaviour |
| Security and privacy | Prove controls and absence of data leakage |
| Performance/resilience | Prove service objectives and fail-closed operation |
| Clinical scenario review | Prove end-to-end suitability with intended users |
| Shadow pilot | Compare product output with normal clinical workflow without influencing care |

## Source-data verification

| ID | Requirement |
|---|---|
| TEST-001 | Two people MUST independently transcribe or derive source threshold data, and discrepancies MUST be resolved against the official source. |
| TEST-002 | The source workbook, recommendation page and normalised pack MUST have recorded retrieval dates and cryptographic hashes. |
| TEST-003 | A clinician MUST verify the meaning, comparator and action associated with every threshold column and graph. |
| TEST-004 | Source verification MUST cover every supported gestation, the actual-gestation rule and the 14-day limit. |
| TEST-005 | The approved source-data review MUST be stored as controlled release evidence. |

## Clinical-core oracle tests

### Threshold control points

For each gestation 23–37, test:

- age 0: phototherapy 40, exchange 80;
- age 72 hours: phototherapy `(10g−100)`, exchange `10g`;
- age 14 days: same plateau values;
- ages one minute before and after 72 hours; and
- midpoint and non-divisible ages to prove exact rational comparison.

For gestation 38, 39, 40, 41 and 42, test every control point in the term table, all midpoints, one minute before/after each point, age 96 hours, and age 14 days.

For every threshold test, evaluate bilirubin:

- at least one integer below the exact line;
- exactly at the line when the line is integral;
- the first integer above the exact line; and
- both serum and TcB eligibility.

| ID | Requirement |
|---|---|
| TEST-006 | Golden threshold tests MUST have zero difference from the approved oracle. |
| TEST-007 | Decisions MUST be tested against exact fractions, not only one-decimal display values. |
| TEST-008 | There MUST be an explicit test proving no threshold is returned after minute 20,160. |

### Exhaustive and property tests

Run an exhaustive threshold generation test for every supported gestation and every minute from 0 through 20,160. At each point verify:

- both lines exist and are finite exact rationals;
- phototherapy line does not exceed exchange line;
- each line is non-decreasing with age;
- preterm lines plateau from 72 hours;
- term phototherapy plateaus from 96 hours and exchange from 42 hours;
- a higher preterm gestation never has a lower line at the same age; and
- every displayed value matches the defined rounding of the exact value.

Property-based generators cover valid and invalid assessments, measurement permutations, treatment-state combinations and extreme values.

| ID | Requirement |
|---|---|
| TEST-009 | Permuting a valid measurement array MUST NOT change the normalised clinical result. |
| TEST-010 | Duplicate measurement ages MUST always be rejected. |
| TEST-011 | Equivalent valid inputs and rule packs MUST produce byte-equivalent canonical clinical payloads before operational metadata. |
| TEST-012 | Checked-arithmetic failures MUST produce a typed safety failure and no partial result. |

### Boundary matrix

Required boundaries:

| Boundary | Cases |
|---|---|
| Gestation | 22/23, 34/35, 36/37, 37/38, 42/43 |
| Assessment age | −1/0, 1,439/1,440/1,441, 4,319/4,320/4,321, 5,759/5,760/5,761, 20,159/20,160/20,161, 40,319/40,320 |
| TcB confirmation | 249/250/251 |
| Treatment distance | 49/50/51 below both lines |
| Rapid rise | exact rates below/at/above 8.5, serum/TcB method combinations, and the minimum one-minute distinct-age interval |
| Kernicterus | bilirubin 339/340/341 and gestation 36/37 |
| Conjugated bilirubin | 24/25/26 |
| Prolonged jaundice | exact minute at and one minute beyond 14 or 21 days |

| ID | Requirement |
|---|---|
| TEST-013 | Every boundary cell MUST have a named test describing the expected rule code and priority. |
| TEST-014 | Equality behaviour MUST be independently reviewed because it is safety-critical product policy. |

## Rule-branch tests

Every rule in the clinical YAML needs:

- positive activation;
- nearest negative case;
- unknown required input;
- source reference assertion;
- expected priority;
- conflict with a higher-priority rule where possible; and
- expected suppression/non-suppression behaviour.

Specific multi-rule scenarios include:

- early jaundice plus above exchange line;
- TcB above treatment line and over 250;
- rapid rise during standard phototherapy;
- phototherapy non-response identified before six hours, exactly at six hours, and from a late result;
- missing baseline, missing post-start result, and monitoring overdue after six hours;
- acute encephalopathy below numeric lines;
- intensified phototherapy with IVIG preconditions partly unknown;
- prolonged jaundice with dark urine and conjugated bilirubin above 25; and
- clinically well term baby more than 50 below the phototherapy line.

## API contract tests

| ID | Requirement |
|---|---|
| TEST-015 | The committed OpenAPI document MUST pass a standards-compliant OpenAPI 3.1 linter. |
| TEST-016 | Every example request and response MUST validate against its schema. |
| TEST-017 | Generated TypeScript, Swift and Kotlin clients MUST compile minimal consumer fixtures. |
| TEST-018 | Unknown properties, duplicate JSON keys, invalid enum values, invalid treatment states and all numerical range failures MUST be rejected. |
| TEST-019 | `409` MUST never include a clinical result and MUST identify the active pack. |
| TEST-020 | `500` and `503` MUST never include stack traces, paths, input values or partial recommendations. |
| TEST-021 | Evaluation responses MUST include `Cache-Control: no-store` and request ID. |
| TEST-022 | Rate and size controls MUST return the documented status and headers. |

Contract testing compares the runtime route schema with the committed OpenAPI document so implementation annotations cannot drift.

## Web tests

Automated component and browser tests cover:

- professional-use gate and session expiry;
- completed-week and fixed-unit labelling;
- DST-forward, DST-back and timezone conversion;
- adding, ordering and removing measurements;
- field-level mapping of RFC 6901 validation pointers;
- successful, emergency, incomplete, stale-pack and unavailable states;
- chart and textual-table equality;
- serum/TcB presentation;
- print receipt content;
- clear action and reload data loss;
- no service-worker/cache storage of evaluation traffic; and
- no clinical values passed to mocked analytics/error reporting.

| ID | Requirement |
|---|---|
| TEST-023 | Browser end-to-end tests MUST run against current supported Chromium, Firefox and WebKit engines. |
| TEST-024 | Automated accessibility checks MUST have no serious or critical findings, and manual keyboard/screen-reader review MUST be completed. |
| TEST-025 | Emergency presentation MUST be understandable in monochrome and at 200% zoom. |
| TEST-026 | Offline/network failures MUST never leave a previous result presented as current. |

## Security and privacy tests

- static analysis and Rust linting with warnings denied in release CI;
- dependency vulnerability and licence scans;
- container and infrastructure-as-code scans;
- secret scanning;
- property/fuzz testing of JSON/domain boundaries;
- OWASP-oriented dynamic testing;
- CSP and security-header checks;
- CORS allowlist tests;
- log-capture tests submitting sentinel clinical values and proving they never appear;
- cache inspection in browser and edge layers;
- SBOM and provenance verification; and
- independent penetration test before advisory pilot.

| ID | Requirement |
|---|---|
| TEST-027 | A release MUST have no unresolved critical or high vulnerability without a documented, approved and time-bounded exception. |
| TEST-028 | Sentinel clinical values MUST be absent from application, WAF, load-balancer, tracing, error-reporting and analytics outputs. |
| TEST-029 | The production image signature, SBOM and rule-pack hash MUST verify before deployment. |

## Performance and resilience

Load profile:

- warm service;
- realistic valid requests containing 1, 2, 10 and 64 measurements;
- sustained 100 requests/second for 15 minutes;
- a two-minute burst at twice the expected allowed throughput behind controlled bypass of per-IP test throttling.

Acceptance:

- p95 server latency under 100 ms and p99 under 250 ms;
- no incorrect result, panic, timeout or memory growth trend;
- error rate below 0.1% excluding intentional limits;
- autoscaling does not cause loss of readiness integrity; and
- rate limiting protects service capacity.

Resilience tests terminate instances, remove an availability zone, present a corrupt rule pack, mismatch release authorisation, expire a certificate in staging and interrupt client connectivity. Clinical failures must remain fail closed.

## Clinical scenario validation

Two clinicians approve at least 60 scenarios distributed across:

- gestations 23–37 and ≥38;
- first-day and later recognition;
- serum/TcB method selection;
- below/at/above both lines;
- serial stable, falling and rapid-rise trends;
- standard/intensified phototherapy and stopping/rebound;
- exchange and encephalopathy;
- underlying disease and haemolysis;
- prolonged jaundice; and
- missing, conflicting and out-of-scope information.

Each scenario records inputs, expected primary/supporting actions, source references, actual output, reviewer identity, outcome and discrepancy disposition.

## Shadow pilot

For a minimum of two weeks, trained pilot clinicians enter consecutive eligible assessments after completing normal clinical decision-making. Bili Mate output is hidden until the normal decision is recorded and must not influence care.

Release to advisory pilot requires:

- no unresolved critical or major discrepancy;
- every minor discrepancy understood and accepted;
- no privacy or safety incident;
- usability issues have safe mitigations;
- operational objectives met; and
- Clinical Safety Officer and pilot clinical owner approval.

## Definition of verified

A requirement is verified only when its traceability row identifies a passing automated or controlled manual test and the evidence is attached to the exact release candidate. “Tested elsewhere”, an unsigned screenshot or review of a different rule pack is not sufficient.
