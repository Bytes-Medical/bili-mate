# Product requirements

## Purpose

Bili Mate provides deterministic clinical decision support for registered healthcare professionals assessing jaundice in newborn babies in the United Kingdom. It combines postnatal age, completed gestational age, bilirubin measurements, clinical features, risk factors, and treatment state to return applicable NICE CG98 thresholds and an ordered set of referenced recommendations.

The software supports a professional decision; it does not independently diagnose, prescribe, order treatment, or replace clinical judgement. All outputs require review in the baby's clinical context and against local pathology advice.

## Intended purpose statement

> Bili Mate is software intended to support registered healthcare professionals in the United Kingdom with assessment and management decisions for jaundice in newborn babies from birth to less than 28 days of age. It calculates and presents bilirubin treatment thresholds up to and including 14 days of age and applies relevant recommendations from NICE guideline CG98. It accepts manually entered clinical observations and does not directly acquire data from a medical device. Its output is advisory and must be reviewed by a suitably trained healthcare professional before action.

The intended-purpose statement MUST be used consistently in product labelling, the API legal endpoint, the web application, technical documentation, clinical evaluation, and regulatory submissions.

## Users and use environment

| User | Supported use |
|---|---|
| Midwife, neonatal nurse, health visitor | Recognition, measurement-method guidance, escalation and follow-up support within competence and local policy |
| Paediatrician or neonatologist | Threshold interpretation, treatment monitoring, escalation and prolonged-jaundice support |
| Clinical safety, governance or audit staff | Verification of rule provenance, decision receipts and release documentation |
| Parent or carer | Not an intended direct user; may be shown clinician-mediated educational material only |

The reference deployment is accessed from modern desktop or mobile browsers in hospitals, maternity services, community care, and home visits where a clinician has reliable internet access.

## Functional requirements

| ID | Requirement |
|---|---|
| PRD-001 | The product MUST evaluate babies with completed gestational age from 23 through 42 weeks and assessment age from birth through 27 days, 23 hours and 59 minutes. |
| PRD-002 | The product MUST calculate treatment thresholds only from birth through 336 hours inclusive. |
| PRD-003 | For assessments after 336 hours, the product MUST withhold treatment-line calculations and may return prolonged-jaundice recommendations where applicable. |
| PRD-004 | The product MUST implement the clinical behaviour defined in [the clinical engine specification](02-clinical-rule-engine.md). |
| PRD-005 | Every clinical result MUST identify the API version, engine version, rule-pack ID, source update date and evaluation ID. |
| PRD-006 | Every recommendation MUST contain a stable code, priority, human-readable action, NICE source reference and clinician-confirmation flag. |
| PRD-007 | The product MUST return one highest-priority primary action and all other non-contradictory applicable recommendations. |
| PRD-008 | The product MUST distinguish missing or unknown clinical information from confirmed absence. |
| PRD-009 | The product MUST return explicit unsupported-scope warnings instead of extrapolating NICE rules. |
| PRD-010 | The product MUST expose an OpenAPI 3.1 contract from which TypeScript, Swift and Kotlin clients can be generated. |
| PRD-011 | The first delivery MUST include a responsive reference web client proving the complete API workflow. |
| PRD-012 | Clinical calculations MUST be performed by the server. Clients MUST NOT reproduce or modify treatment logic. |
| PRD-013 | The product MUST fail closed when a clinical evaluation cannot be obtained from the server. |
| PRD-014 | The service MUST NOT retain evaluation request or response content after the request completes. |
| PRD-015 | The public interface MUST NOT accept patient names, NHS numbers, hospital numbers, addresses, free text or other direct identifiers. |
| PRD-016 | The web client MUST calculate elapsed age locally from user-entered dates and submit only elapsed minutes. |
| PRD-017 | The product MUST provide a printable, non-identifying decision receipt to the requesting client. |
| PRD-018 | The product MUST provide current legal, source, intended-user and professional-use notices through both UI and API. |
| PRD-019 | A rule-pack update MUST never silently alter a previously issued decision receipt. |
| PRD-020 | The product MUST support immediate rollback to the previous clinically approved rule pack. |

## Clinical safety-facing behaviour

| ID | Requirement |
|---|---|
| PRD-021 | The product MUST show that visual inspection alone cannot estimate bilirubin level. |
| PRD-022 | The product MUST warn that hyperbilirubinaemia-related pigmentation changes can be harder to see in darker skin. |
| PRD-023 | Every threshold result MUST carry the warning that bilirubin assays vary and the local pathology laboratory must be consulted. |
| PRD-024 | The product MUST NOT present an out-of-scope result as “normal”, “safe” or “no action needed”. |
| PRD-025 | Unknown danger signs MUST prevent a reassuring primary action. |
| PRD-026 | The product MUST clearly distinguish serum from transcutaneous measurements and prohibit unsupported treatment decisions from TcB. |
| PRD-027 | Recommendations involving exchange transfusion, acute bilirubin encephalopathy or specialist treatment MUST be visually and semantically marked emergency or immediate escalation. |

## Non-functional requirements

| ID | Requirement |
|---|---|
| PRD-028 | A warmed API instance MUST complete 95% of valid evaluations within 100 ms under a sustained load of 100 requests per second. |
| PRD-029 | The pilot service MUST target 99.9% monthly availability, excluding agreed maintenance. |
| PRD-030 | Equivalent normalised inputs and the same rule pack MUST always produce equivalent clinical outputs. |
| PRD-031 | Clinical comparisons MUST use exact arithmetic and MUST NOT depend on floating-point rounding. |
| PRD-032 | The web interface MUST meet WCAG 2.2 AA and MUST NOT communicate clinical state through colour alone. |
| PRD-033 | All production traffic MUST use TLS 1.2 or newer; TLS 1.3 SHOULD be preferred. |
| PRD-034 | Evaluation responses MUST use `Cache-Control: no-store`. |
| PRD-035 | The public deployment MUST be restricted to UK access to support the NICE content licence and intended market. |
| PRD-036 | The production container MUST run as a non-root user with a read-only root filesystem. |
| PRD-037 | Request and response bodies, bilirubin values and clinical flags MUST NOT appear in application or infrastructure logs. |

## Explicit exclusions

| ID | Exclusion |
|---|---|
| PRD-038 | The first release MUST NOT diagnose jaundice from photographs, video, skin colour or scleral images. |
| PRD-039 | The first release MUST NOT contain AI, machine learning or probabilistic prediction. |
| PRD-040 | The first release MUST NOT provide parent-directed treatment recommendations. |
| PRD-041 | The first release MUST NOT integrate with an EPR, FHIR server, laboratory feed or treatment-ordering system. |
| PRD-042 | The first release MUST NOT persist longitudinal patient episodes or clinician accounts. |
| PRD-043 | The first release MUST NOT calculate while offline or execute downloaded rule packs on clients. |
| PRD-044 | The first release MUST NOT convert or accept bilirubin values in mg/dL. |
| PRD-045 | The first release MUST NOT calculate phototherapy or exchange thresholds after 14 days. |
| PRD-046 | The first release MUST NOT claim NICE or NHS endorsement or use their logos. |
| PRD-047 | The first release MUST NOT be marketed or distributed for clinical use outside the United Kingdom. |

## Primary user journeys

### New assessment

1. The clinician acknowledges professional-use terms for the browser session.
2. The clinician enters birth and assessment times locally; the client displays calculated age in hours and minutes.
3. The clinician enters completed gestational weeks, clinical features, risk factors, measurements and treatment state.
4. The client displays a pre-submission summary, including measurement units and age.
5. The API validates the request and evaluates the active rule pack.
6. The client displays the primary action, supporting recommendations, thresholds, chart, missing information, warnings and sources.
7. The clinician reviews the output against the patient, local laboratory and local protocol.
8. The clinician may print the non-identifying receipt and then clears the assessment.

### Serial measurement

The clinician enters at least two measurements with distinct ages. The engine orders them, validates that no age is in the future relative to the assessment, calculates a trend, and states whether the measurement methods permit confirmation of the serum rapid-rise rule.

### Service unavailable

The client MUST show that no clinical result was produced, preserve the currently entered values only in volatile memory, offer retry and clear actions, and direct the clinician to use approved local procedures. It MUST NOT show the last successful result as current.

## Success criteria

The release is successful when:

- all requirements map to tests and safety controls;
- automated clinical oracle tests have zero divergence;
- two authorised clinical reviewers approve the rule pack and clinical scenarios;
- no critical or major discrepancy remains from shadow-mode comparison;
- regulatory qualification and local clinical-safety gates are complete; and
- the service meets accessibility, security, latency and availability acceptance criteria.
