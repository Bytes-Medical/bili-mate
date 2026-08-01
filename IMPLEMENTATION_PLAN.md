# Bili Mate implementation plan

Status: Approved; milestones M0–M2 implemented (2026-08-01)
Based on: specification `0.1.0`, rule pack `nice-cg98-2023-10-31.1`
Date: 2026-08-01

> **Progress:** M0 (workspace, CI, ADR-009/010) and the Stage 1 engineering
> deliverables of M1–M2 are implemented: `crates/clinical-core` (exact
> arithmetic, thresholds, full rule engine, oracle/exhaustive/boundary/
> property test suites), `crates/guideline-data` (embedded pack, integrity
> self-tests, pack diff tool) and `apps/cli` (`bili-eval`, validated against
> the `spec/examples` fixtures). Stage 1 exit still requires the two-person
> source-data reconciliation and clinical review of mathematics and
> comparators (see risks 4 and 7). Next engineering milestone: M3 (Axum API).

This plan turns the normative specification in [`spec/`](spec/README.md) into a sequenced engineering programme. It follows the delivery stages in [11-delivery-and-rollout](spec/11-delivery-and-rollout.md) and the ordering principle stated there: **clinical behaviour is validated before interface polish can conceal errors**. The clinical core is built and proven first, the API second, the web client third, and hardening/operations last before the governance-heavy validation stages.

## Plan-level decisions

Two decisions in this plan go beyond the spec and should be confirmed at review:

1. **Frontend framework: Next.js (App Router) with static export.**
   [Spec 05](spec/05-system-architecture.md) and [Spec 06](spec/06-reference-web.md) require a "separately built static React/TypeScript application". Next.js with `output: 'export'` produces exactly that — a static bundle served from S3/CloudFront with no server runtime. Constraints this imposes (all compatible with the spec, and several actively enforce it):
   - No Next.js API routes, server actions, or server components that touch clinical data. All clinical state is client-side React state only (WEB-018).
   - `next/image` optimisation disabled (`unoptimized: true`); no image CDN needed for this UI.
   - Next.js static export emits inline bootstrap scripts. SEC-016 prohibits `unsafe-inline` for scripts, so the CSP must use **hash-based** script-src entries generated at build time. This is a build-pipeline work item (M4), not an afterthought.
   - Routing uses the six fixed routes from Spec 06. No clinical input may ever appear in a URL, and static export makes accidental server-side leakage of form state structurally impossible.

2. **Visual design: strict black-and-white (monochrome) design system.**
   This is a product/aesthetic choice that happens to align with normative requirements: PRD-032 and WEB-012 forbid communicating clinical state through colour alone, and TEST-025 requires emergency presentation to be understandable in monochrome. Going monochrome by design means the accessible encoding (typography scale, weight, borders, iconography, dash patterns, spacing) *is* the design rather than a fallback. Design tokens:
   - Palette: `#000` / `#fff` plus a grayscale ramp (e.g. 5 greys) for hierarchy. No hue anywhere, including the chart and print output.
   - Priority encoding: `emergency`/`immediate` use heavy black banners with white text, warning iconography, uppercase labels and assertive live regions; `urgent`/`treatment` use bordered panels and bold headings; `timed`/`routine` use plain panels. Every priority level is also always written out as text from the server response (WEB-011).
   - Chart (Spec 06): phototherapy line solid black, exchange line dashed black, serum measurements filled markers, TcB measurements hollow markers — dash pattern and marker shape carry the meaning, satisfying the chart rules without colour.
   - Contrast trivially exceeds WCAG 2.2 AA; focus states use thick black outlines with offset.

## Repository layout

Per [Spec 05](spec/05-system-architecture.md):

```text
bili-mate/
├── Cargo.toml                 # workspace
├── crates/
│   ├── clinical-core/         # pure deterministic engine, no I/O
│   └── guideline-data/        # rule-pack schema, loading, hashes, self-tests
├── apps/
│   └── api/                   # Axum HTTP service
├── web/                       # Next.js static-export reference client
├── infrastructure/            # Terraform (AWS eu-west-2 reference topology)
├── clients/                   # generated TS/Swift/Kotlin clients + consumer fixtures
└── spec/                      # existing normative specification
```

## Milestones

Durations assume 1–2 engineers working full time and are engineering effort only; governance stages (M6+) are gated on clinical/regulatory actors and are not schedule-bound (OPS-016: elapsed time is never an exit criterion).

| Milestone | Maps to spec stage | Scope | Rough duration |
|---|---|---|---|
| M0 | Stage 0 (tail) | Repo scaffolding, CI, spec validator in CI | 1 week |
| M1 | Stage 1 | Domain types, exact arithmetic, thresholds + oracle tests | 2–3 weeks |
| M2 | Stage 1 | Full rule engine, rule pack loader, CLI evaluator — **Stage 1 exit** | 3–4 weeks |
| M3 | Stage 2 | Axum API, contract tests, generated clients | 3 weeks |
| M4 | Stage 2 | Next.js reference web (black & white), accessibility | 4 weeks |
| M5 | Stage 2 | Security/privacy hardening, performance, container, IaC — **Stage 2 exit** | 3 weeks |
| M6 | Stage 3 | Clinical & safety validation support tooling and evidence | governance-gated |
| M7 | Stages 4–5 | Shadow pilot then advisory pilot operations | governance-gated |

Engineering path to Stage 2 exit: roughly 16–18 weeks.

---

### M0 — Foundations (≈1 week)

**Goal:** a repository where every later milestone lands with tests, lint and traceability enforced from day one.

Work packages:

- Cargo workspace with `clinical-core`, `guideline-data`, `apps/api` stubs; pinned stable Rust toolchain (`rust-toolchain.toml`), `Cargo.lock` committed (ADR-001).
- CI pipeline (GitHub Actions or equivalent): `cargo fmt --check`, `clippy -D warnings` (TEST security list), `cargo test`, `ruby spec/validate.rb`, OpenAPI 3.1 lint (TEST-015), example/schema validation (TEST-016), secret scanning, dependency audit (`cargo-deny`: advisories + licences).
- `web/` scaffolded with Next.js + TypeScript, `output: 'export'`, ESLint/Prettier, Playwright configured for Chromium/Firefox/WebKit (TEST-023).
- Decision log: record the Next.js and monochrome decisions above as ADR-009/ADR-010 in `spec/decisions/`.

Exit criteria: CI green on an empty-but-wired workspace; spec validator runs on every push.

---

### M1 — Clinical core: types and exact threshold mathematics (≈2–3 weeks)

**Goal:** the safety-critical arithmetic, proven against the oracle before any rule logic exists.

Work packages:

1. **Domain types** ([Spec 03](spec/03-domain-model.md)): `GestationalWeeks`, `AgeMinutes`, `BilirubinUmolL`, `TriState`, `MeasurementMethod`, `TreatmentMode`, `Priority`, `ThresholdRelation` as newtypes/enums with constructor validation. Treatment-state invariants (DATA-013/014) encoded so invalid combinations are unrepresentable or rejected with field-specific errors.
2. **Exact rational arithmetic**: `ExactThreshold` as reduced `i64/u64` fraction; all comparison via checked cross-multiplication; overflow → typed safety error, never a result (DATA: checked arithmetic; API-011; TEST-012). No floating point anywhere in decision paths (PRD-031).
3. **Threshold calculation** (CLIN-018–CLIN-027): preterm formulas (23–37 weeks) and term control-point interpolation (38–42 weeks) in exact rationals; three-state relation (`below`/`at`/`above`) with exact equality preserved (CLIN-024); signed unrounded distances; display rounding (half-away-from-zero, 1 dp) applied only at the formatting boundary, never in decisions.
4. **Oracle test suite** (TEST-006–TEST-008): every control point, every gestation, ±1-minute boundaries around 72 h/96 h/14 d, midpoints, non-divisible ages, integral-line equality cases.
5. **Exhaustive + property tests** (Spec 09): all gestations × every minute 0–20,160 verifying existence, monotonicity, phototherapy ≤ exchange, plateaus, gestation ordering, display-rounding consistency. Property generators (`proptest`) for valid/invalid inputs. Determinism test: byte-equivalent canonical payloads for equivalent inputs (TEST-011).

Exit criteria: zero oracle divergence; exhaustive sweep passes; core crate has no I/O, clock, network or env dependency (DATA-024) — enforced by `#![no_std]`-adjacent discipline or a lint/dependency check in CI.

Risk to resolve early: confirm `i64` headroom for the cross-multiplications in the exhaustive sweep (worst-case products of value × denominator over the full domain). If tight, widen to `i128` internally — decide in M1, not later.

---

### M2 — Clinical core: rule engine and rule pack (≈3–4 weeks) — Stage 1 exit

**Goal:** the complete CG98 decision pathway, evaluated deterministically from a validated input and an embedded rule pack.

Work packages:

1. **`guideline-data` crate**: serde schema for the rule-pack YAML; build-time embedding of the approved pack; SHA-256 manifest verification; startup self-test vectors; candidate-vs-predecessor diff tool (Spec 05). No network access ever (production MUST NOT scrape NICE).
2. **Recognition and measurement-method rules** (CLIN-028–CLIN-031, CLIN-051): the eleven recognition codes, the exact 1,440/1,441-minute day-one boundary, darker-skin warning, `INCOMPLETE_DANGER_ASSESSMENT` blocking reassurance on `unknown`.
3. **Trend calculation** (CLIN-035–CLIN-037): exact rational rate vs 8.5 µmol/L/h, `AT_RAPID_RISE_BOUNDARY` at exact equality, serum-only reliability flag.
4. **Below-line monitoring** (CLIN-032–CLIN-034): 18 h/24 h/no-repeat branches with full eligibility conditions; `RETEST_INTERVAL_LOCAL_PROTOCOL` for ineligible populations.
5. **Treatment pathway** (CLIN-038–CLIN-043): phototherapy start/monitoring/stop/rebound, intensified-phototherapy triggers (including the three failure-to-respond variants), kernicterus risk, exchange escalation and at-line equality codes, IVIG conjunctive preconditions, underlying-disease assessment, prolonged jaundice with strict 14/21-day boundaries and the 25 µmol/L conjugated boundary.
6. **Priority and conflict resolution** (CLIN-047–CLIN-050): fixed six-level priority, stable rule order from the YAML, exactly one `primary_action`, suppression trace, missing-information reporting.
7. **Test battery** (Spec 09): the full boundary matrix (every named cell with expected code and priority, TEST-013), rule-branch tests for every code (positive, nearest-negative, unknown-input, source-ref, priority, conflict, suppression), and the listed multi-rule scenarios.
8. **CLI synthetic evaluator** (Stage 1 deliverable): feeds JSON fixtures through the core for engineering and clinical-review use; never shipped to production.

Exit criteria (= Stage 1 exit, Spec 11): zero oracle divergence; complete control-point and boundary coverage; two-person source-data reconciliation recorded in the rule-pack manifest (TEST-001–TEST-005 — needs the clinical owner, schedule this hand-off now); no critical static/security finding; clinical review of mathematics and comparators booked.

---

### M3 — API service (≈3 weeks)

**Goal:** the Axum service implementing `openapi.yaml` exactly, with contract drift impossible.

Work packages:

1. **HTTP layer**: `POST /v1/evaluations` (the ten-step processing order from Spec 04), `GET /v1/guidelines/active`, `GET /v1/threshold-curves/{rule_pack_id}`, `GET /v1/legal`, `GET /health/live`, `GET /health/ready` with full integrity self-checks.
2. **Strict parsing**: JSON with **duplicate-key rejection** and unknown-property rejection (SEC-006, API-018 tests). Note: `serde_json` silently keeps the last duplicate key — this needs a custom deserializer pass or a validating parse step; treat it as a named work item, not a serde default.
3. **Validation**: schema layer + domain layer, all field errors aggregated, RFC 6901 pointers, `application/problem+json` with stable codes; no submitted value echoed (API-016/017).
4. **Middleware**: body-size limit (64 KiB, reject content-encoding), timeouts (read 5 s / evaluation 2 s), per-IP rate limiting with `Retry-After` (API-005/006; e.g. `tower_governor` — edge WAF also enforces, but the server MUST itself), CORS allowlist, security headers (SEC-015, API JSON CSP), `Cache-Control: no-store` on evaluations, `X-Request-ID` echo/generation.
5. **Privacy-safe observability**: tracing with an explicit field allowlist (OPS log list); metrics without clinical labels (SEC-008); panic handler at request boundary → 500 + alert metric, then process exit on invariant panic (Spec 05 failure model).
6. **Configuration**: typed immutable startup config; clinical mode refuses readiness without active pack + release-authorisation reference (OPS-004, OPS-011).
7. **Contract testing** (TEST-015–TEST-022): runtime route schema compared against committed `openapi.yaml`; every `spec/examples/*.json` fixture exercised end-to-end; error-path tests (409 stale pack with active-pack ID, 413, 422, 429, 500/503 leakage checks).
8. **Client generation** (PRD-010, TEST-017): CI job generating TypeScript, Swift and Kotlin clients from the committed contract plus minimal consumer fixtures that must compile (metadata fetch, request build, exhaustive enums, 409 refresh, fail-closed handling).

Exit criteria: contract tests green; all spec examples validate against the running service; generated clients compile their fixtures in CI.

---

### M4 — Reference web client: Next.js, black and white (≈4 weeks)

**Goal:** the full professional workflow from [Spec 06](spec/06-reference-web.md), on the generated TypeScript client, with zero clinical logic in the browser.

Work packages:

1. **Design system**: monochrome tokens as described in plan-level decision 2; typography scale; priority-banner components; print stylesheet (WEB-022: receipt without absolute timestamps).
2. **Shell and routes**: `/`, `/assessment`, `/about`, `/privacy`, `/accessibility`, `/service-status`; professional-use acknowledgement gate in `sessionStorage` only (WEB-001–WEB-003); unsupported-browser gate.
3. **Assessment workflow**: seven-step form with persistent review summary; local IANA-timezone date handling (Temporal API or `@js-temporal/polyfill`) deriving elapsed minutes including DST transitions (WEB-004/005); tri-state controls requiring explicit selection (WEB-009); fixed `µmol/L` labelling (WEB-007); explicit serum/TcB method per measurement (WEB-008); one-action confirmed clear (WEB-010). Birth/collection timestamps never leave the browser.
4. **Result presentation**: the twelve-item order from Spec 06; server-provided priorities and rounding only (WEB-011, WEB-014); TcB ineligibility statement (WEB-013); rule-pack ID and source date always visible (WEB-016); NICE attribution in footer and print (WEB-017); assertive live region + focus management for emergency states (WEB-012).
5. **Chart**: accessible SVG plotting server curve points from `/v1/threshold-curves`; dash-pattern/marker encoding per the design system; per-point text alternatives; adjacent tabular equivalent showing identical server data (WEB-015).
6. **Failure states**: offline/timeout fail-closed screen with local-protocol direction; 409 refresh-and-review flow; 422 pointer→field mapping; 429 with retry time; no silent retry of clinical POSTs; refresh loses state by design with upfront privacy explanation.
7. **Storage/telemetry discipline**: in-memory state only; no localStorage/IndexedDB/cookies for clinical data (WEB-019); service worker (if any) caches shell + metadata GETs only (WEB-020); error reporting configured with bodies/breadcrumbs/DOM capture off (SEC-009, WEB-021).
8. **CSP build step**: generate script hashes for Next.js inline bootstrap scripts; emit final CSP headers/meta per SEC-016; automated header test.
9. **Test suite** (TEST-023–TEST-026): Playwright across three engines covering the full Spec 09 web list (gate expiry, DST cases, pointer mapping, emergency/incomplete/stale/unavailable states, chart-table equality, print content, no-cache verification, mocked-analytics leak test); axe-based automated accessibility scans with zero serious/critical findings; manual keyboard/screen-reader/200 %-zoom review checklist.

Exit criteria: all web tests green on Chromium/Firefox/WebKit; accessibility scan clean; monochrome emergency presentation validated at 200 % zoom (TEST-025 falls out of the design for free — still test it).

---

### M5 — Hardening, performance, packaging and infrastructure (≈3 weeks) — Stage 2 exit

**Goal:** a deployable, observable, privacy-proven demonstration system meeting every Stage 2 exit criterion.

Work packages:

1. **Sentinel leak tests** (TEST-028): submit sentinel clinical values through the full stack; assert absence from app logs, traces, metrics, error reporting, and (in staging) WAF/ALB logs.
2. **Performance** (PRD-028, Spec 09 load profile): warm-service load at 100 rps for 15 min with 1/2/10/64-measurement payloads; p95 < 100 ms, p99 < 250 ms; burst test; no memory growth; profiling and fixes as needed.
3. **Container** (Spec 10 contract): multi-stage build, distroless/scratch base, non-root fixed UID, read-only rootfs, bounded tmpfs, SIGTERM drain ≤ 10 s, OCI labels, SBOM (syft), signed provenance (cosign/SLSA), scan gate (SEC-011–SEC-013, TEST-029).
4. **Terraform reference deployment** (`infrastructure/`): eu-west-2 topology from Spec 10 — Route 53/ACM, CloudFront + OAC + UK geo restriction for the static web bucket, WAF (UK allow, managed rules, size/rate), ALB, ECS Fargate ×2 AZ, ECR immutable tags, CloudWatch dashboards + the alarm table, Secrets Manager, remote state. Deployed to a non-clinical test environment.
5. **Deployment pipeline**: the ten-step release process (build → artefact verification → signing → reviewed plan → canary → staged traffic → recorded rollback target); rollback rehearsal meeting the 15/30-minute targets.
6. **Resilience tests**: instance kill, AZ removal, corrupt rule pack, mismatched release authorisation, expired staging cert, client interruption — all fail closed.
7. **DAST + header/CORS/cache verification**; fuzzing of the JSON/domain boundary (cargo-fuzz on the parser/validator).
8. **Formative usability review** with intended professional users (Stage 2 exit item — needs clinical participants; schedule alongside engineering).

Exit criteria (= Stage 2 exit, Spec 11): OpenAPI/runtime match; all examples validate; no serious/critical accessibility finding; no sentinel data in telemetry; performance criteria met; usability review completed.

---

### M6 — Clinical and safety validation support (Stage 3, governance-gated)

Engineering is a supporting actor here; most deliverables belong to the clinical owner, CSO, regulatory lead and licensing owner. Engineering work packages:

- **Scenario harness**: runnable, versioned encoding of the ≥60 clinical validation scenarios (inputs, expected primary/supporting actions, source refs) with signed-review export for the two clinical reviewers.
- **Weekly guideline monitor** (Spec 08): scheduled job checking CG98 pages/resources for update date, ETag/content hash and withdrawal; notify-only.
- **Rule-pack promotion tooling**: candidate → reviewed → `active` status flow with reviewer identities recorded in the manifest; clinical-output diff report between packs for the release checklist.
- **Evidence automation**: traceability-matrix check in CI (every requirement ID ↦ named test), release-evidence bundle generator (versions, digests, test results, OpenAPI diff).
- Support for: penetration test remediation, DPIA data-flow map, DTAC pack, hazard-control verification evidence (SAFE-017–SAFE-022).

Exit: Stage 3 exit criteria in Spec 11 — all gates SAFE-017–SAFE-022, no open critical/major issue, CSO authorises shadow evaluation.

---

### M7 — Shadow pilot and advisory pilot (Stages 4–5, governance-gated)

Engineering responsibilities: shadow-pilot environment with active release candidate; heightened monitoring and weekly metric reports (availability, latency, incident routes) without clinical content; discrepancy-review support tooling; immediate disable/rollback capability drills (OPS-011); pilot training material input (OPS-019). Clinical mode remains off until every Stage 5 gate — including local DCB0160, pathology approval and the regulatory route — is signed.

---

## Cross-cutting workstreams (run continuously)

| Workstream | Cadence | Owner |
|---|---|---|
| Hazard Log maintenance (SAFE-005) — from first executable prototype (M1) onward | Every PR touching clinical behaviour | CSO + engineering |
| Traceability matrix updates with every requirement-affecting change | Every PR | Engineering |
| Dependency/vulnerability/licence scanning (SEC-013) | Every build | CI |
| Claims register and intended-purpose wording consistency (SAFE-001) | Every user-facing text change | Product + regulatory |
| NICE content-rights determination (SAFE-012) — start immediately; long lead time, blocks Stage 3 | Now | Licensing owner |
| Regulatory qualification (MHRA SaMD assessment) — start immediately; blocks Stage 3 | Now | Regulatory lead |

## Key risks and early decisions

| # | Risk / open item | Mitigation / when |
|---|---|---|
| 1 | `i64` rational overflow headroom in exact arithmetic | Analyse worst-case products in M1; move to `i128` internally if tight. Checked ops make failure safe either way. |
| 2 | Duplicate-JSON-key rejection is not serde default behaviour | Named work item in M3; custom parse/validate pass with dedicated tests. |
| 3 | Next.js inline bootstrap scripts vs hash-based CSP (SEC-016) | Build-time hash generation in M4; automated header test. If it proves brittle, fall back to Vite + React (still spec-compliant) — decide by end of M4, not later. |
| 4 | Two-person source transcription (TEST-001) needs a clinician early | Schedule the double-entry reconciliation during M2, before Stage 1 exit review. |
| 5 | Rate limiting semantics (server-enforced per API-005 while WAF also limits) | Implement in app middleware in M3; integration-test both layers in M5 staging. |
| 6 | Formative usability review needs real professional users (Stage 2 exit) | Recruit during M3–M4 so review lands inside M5. |
| 7 | Content rights and regulatory qualification are the long poles for Stage 3 | Start both workstreams now (Stage 0 obligations), in parallel with all engineering milestones. |

## Definition of done (engineering, per milestone)

A milestone is complete only when: all its named requirement IDs have passing automated tests linked in the traceability matrix; CI is green including lint, audit and spec validation; the Hazard Log reflects any new or changed hazard controls; and the milestone review is recorded. Per Spec 09, "tested elsewhere" or unsigned evidence does not count.
