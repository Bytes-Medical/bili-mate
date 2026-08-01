# Delivery and rollout

## Delivery principles

- Technical completion and authorisation for patient-care use are separate decisions.
- Clinical behaviour is validated before interface polish can conceal errors.
- Every stage uses the exact rule-pack and engine pairing intended for the next stage.
- A stage cannot begin with an open critical/major safety discrepancy.
- Clinical mode defaults off.

## Roles

| Role | Accountability |
|---|---|
| Product owner | Scope, intended users, funding and claims |
| Engineering lead | Architecture, implementation, verification and operations |
| Clinical owner/neonatologist | Clinical interpretation, source approval, scenarios and pilot |
| Clinical Safety Officer | DCB0129 process, Hazard Log, safety case and release safety approval |
| Regulatory lead | MHRA qualification/classification and conformity route |
| Security/privacy lead | Threat model, DPIA support, security evidence and incident process |
| Content/licensing owner | NICE/third-party permissions, attribution and source surveillance |
| Pilot organisation CSO | Local DCB0160 assessment and deployment authority |
| Pathology representative | Local assay and threshold-workflow review |

One person may hold multiple roles only where competence and independence requirements remain satisfied. The two clinical rule reviewers must be independent of the original transcription.

## Stage 0: specification and governance

Deliverables:

- approved specification and traceability baseline;
- named legal manufacturer and accountable roles;
- intended-purpose and claims register;
- NICE content-rights request;
- initial regulatory assessment;
- Clinical Risk Management Plan and initial Hazard Log; and
- source acquisition and change-monitor plan.

Exit criteria:

- no unresolved product or clinical boundary decision;
- clinical owner and CSO accept the planned controls;
- rights/regulatory work is active with no known fatal blocker; and
- implementation requirements are testable.

## Stage 1: deterministic engine demonstration

Deliverables:

- Rust domain types and exact arithmetic;
- candidate rule-pack loader and integrity checks;
- all threshold/rule unit, property and exhaustive tests;
- command-line synthetic evaluator for engineering only; and
- source transcription/review evidence.

Mode: demonstration only; synthetic data only.

Exit criteria:

- zero oracle divergence;
- complete control-point and boundary coverage;
- two-person source-data reconciliation;
- no critical static/security finding; and
- clinical review of mathematics and comparators.

## Stage 2: API and reference web

Deliverables:

- Axum API implementing `openapi.yaml`;
- generated TypeScript, Swift and Kotlin clients plus consumer fixtures;
- reference web assessment and result workflow;
- privacy-safe logging and security controls;
- portable container and test environment; and
- automated contract, web, accessibility and load tests.

Mode: demonstration only; synthetic data only.

Exit criteria:

- OpenAPI/runtime contract matches;
- all examples validate;
- no serious/critical accessibility finding;
- no sentinel clinical data in telemetry;
- performance criteria met; and
- formative usability review completed with intended professional users.

## Stage 3: clinical and safety validation

Deliverables:

- two clinical rule-pack approvals;
- at least 60 signed end-to-end scenarios;
- completed hazard-control verification;
- draft Clinical Safety Case Report;
- regulatory qualification/classification decision;
- NICE and third-party content permission;
- penetration test and remediations;
- DTAC evidence pack; and
- clinical-validation environment.

Mode: clinical output visible only to authorised validation reviewers; not used for care.

Exit criteria:

- all SAFE-017 through SAFE-022 gates met;
- no open critical/major clinical, security or privacy issue;
- known limitations and residual risks approved; and
- manufacturer CSO authorises shadow evaluation.

## Stage 4: shadow pilot

Preparation:

- pilot site performs DCB0160 assessment;
- pathology approves local assay interpretation warning and workflow;
- clinicians receive training and downtime procedure;
- support and incident routes are tested;
- DPIA and local governance are approved; and
- output remains hidden until normal clinical decision is recorded.

Operation:

- minimum two weeks;
- consecutive eligible assessments where practicable;
- no product output influences care;
- discrepancies reviewed promptly by two clinicians;
- availability, usability and incident metrics reviewed weekly; and
- no patient assessment retained by the product.

Exit criteria:

- no unresolved critical/major discrepancy;
- minor discrepancies dispositioned;
- no unresolved safety/privacy incident;
- local CSO, clinical owner and manufacturer CSO jointly approve; and
- complete clinical-mode release authorisation exists.

## Stage 5: advisory clinical pilot

The product may support decisions only after every safety release gate, including local DCB0160 and pathology approval.

Before any output can influence care, the regulatory lead must document and complete every applicable legal route for that pilot, including any conformity assessment, registration, clinical-investigation or health-institution requirements. A “pilot” label is not an exemption from regulation.

Controls:

- limited trained clinical cohort defined by pilot governance even though the endpoint has no login;
- public interface retains professional-use acknowledgement and UK geo restriction;
- heightened support and incident monitoring;
- weekly multidisciplinary review;
- immediate disable/rollback authority available at all times;
- source guidance checked weekly; and
- feedback is analysed without retaining assessment content in product telemetry.

The pilot does not establish market readiness or authorise wider NHS deployment.

## Stage 6: wider release assessment

Wider release is a separate programme requiring:

- completed applicable medical-device conformity and MHRA registration;
- mature quality-management and post-market surveillance;
- final DTAC and procurement evidence;
- pilot benefit/risk and usability evaluation;
- operating/support capacity;
- updated clinical safety case;
- product and content licensing review; and
- explicit approval for the proposed distribution and organisations.

## Release checklist

Every release records:

- source commit and protected tag;
- API, engine, web and rule-pack versions;
- image and rule-pack digests;
- OpenAPI diff and compatibility assessment;
- clinical-output diff from previous release;
- automated test evidence;
- vulnerability, SBOM, provenance and penetration status;
- accessibility status;
- Hazard Log and safety-case impact;
- source/licence currency;
- regulatory impact assessment;
- named engineering, clinical, CSO, security and operations approvals; and
- exact rollback target.

## Rollout requirements

| ID | Requirement |
|---|---|
| OPS-015 | Clinical mode MUST remain disabled through Stages 0–4. |
| OPS-016 | Each stage MUST have signed entry and exit evidence; elapsed schedule time is not an exit criterion. |
| OPS-017 | A rule-pack, engine or intended-purpose change MUST return to the earliest affected validation stage. |
| OPS-018 | A critical discrepancy or integrity incident MUST suspend clinical mode pending assessment. |
| OPS-019 | Pilot training MUST include scope, age/gestation entry, units, TcB/serum distinction, assay warning, downtime and incident reporting. |
| OPS-020 | Rollout communications MUST identify current versions, changes, known limitations and whether clinical use is authorised. |
| OPS-021 | Decommissioning MUST remove access, preserve required controlled artefacts and communicate an alternative clinical workflow. |

## Deferred roadmap

Only after the first advisory pilot is accepted:

1. production iOS and Android apps generated from the v1 contract;
2. optional organisation OIDC access;
3. carefully scoped local-protocol profiles;
4. EPR/FHIR or laboratory integration;
5. patient episode persistence; and
6. additional jurisdictions or guidelines.

Each item reopens privacy, clinical safety, regulatory, architecture and validation decisions. None is implicitly authorised by this specification.
