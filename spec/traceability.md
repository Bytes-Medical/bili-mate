# Traceability matrix

Status: Draft. Evidence references become immutable release-artefact links when a release candidate is built.

## Product and architecture requirements

| Requirement(s) | Source/decision | Implementation area | Verification | Safety link |
|---|---|---|---|---|
| PRD-001–PRD-003 | CG98 scope and ADR-008 | Domain validation, threshold service | TEST-006, TEST-008, TEST-013 | H-001, H-002 |
| PRD-004–PRD-009 | Clinical specification | Clinical core and response model | TEST-006–TEST-014 | H-005, H-007, H-008 |
| PRD-010–PRD-013 | ADR-002, ADR-007 | OpenAPI, generated clients, client failure state | TEST-015–TEST-026 | H-010 |
| PRD-014–PRD-020 | ADR-003, ADR-005 | Stateless API, receipts, release/rollback | TEST-011, TEST-019, TEST-028, TEST-029 | H-006, H-009, H-013, H-014 |
| PRD-021–PRD-027 | CG98 1.2.5–1.2.6, 1.2.15–1.2.16, 1.5.1, 1.9 | Clinical content and presentation | Rule scenarios; TEST-023–TEST-026 | H-004, H-007, H-011, H-012 |
| PRD-028–PRD-037 | Performance, security and deployment decisions | API/web/infrastructure | Load, accessibility, security and log tests | H-009, H-010, H-013–H-015 |
| PRD-038–PRD-047 | Intended purpose and ADR-006/008 | Claims, schemas, UI and distribution | Scope/claims review and schema tests | H-003, H-012 |
| CLIN-001–CLIN-006 | CG98 source governance and ADR-005/006 | Rule-pack ingestion, manifest and release gate | TEST-001–TEST-005, TEST-029 | H-005, H-006, H-013 |
| CLIN-007–CLIN-027 | NICE threshold table and graphs | Exact threshold core | TEST-001–TEST-014 | H-001, H-002, H-005 |
| CLIN-028–CLIN-034, CLIN-051 | CG98 1.2 and 1.4.1–1.4.2 | Recognition, method and pre-treatment rules | Branch, boundary and missing-state tests | H-001, H-004, H-007, H-008 |
| CLIN-035–CLIN-043 | CG98 1.4.4–1.5.1 and 1.8–1.9 | Trend, treatment and emergency rules | Trend/treatment/exchange scenarios | H-001, H-005, H-008 |
| CLIN-044–CLIN-046 | CG98 1.7 | Prolonged-jaundice rules | 14/21-day, liver and UTI scenarios | H-001, H-007, H-008 |
| CLIN-047–CLIN-050 | Product clinical-safety policy | Priority resolver and missing-information output | Pairwise conflicts and unknown-state tests | H-007, H-008, H-011 |
| DATA-001–DATA-006 | Domain model and ADR-003 | Request parsing/schema | TEST-016, TEST-018, TEST-028 | H-001–H-004, H-014 |
| DATA-007–DATA-014 | Clinical chronology and treatment-state model | Domain validation | TEST-009, TEST-010, TEST-013, TEST-018 | H-001, H-008 |
| DATA-015–DATA-023 | API response semantics | Clinical core/API mapper | Contract and recommendation branch tests | H-008, H-009, H-015 |
| DATA-024–DATA-030 | ADR-001, ADR-003, ADR-005 | Workspace boundaries and startup | TEST-011, TEST-012, TEST-029, readiness tests | H-005, H-013 |
| API-001–API-009 | HTTP contract | Axum/Tower/edge | TEST-015–TEST-022 | H-009, H-010, H-014 |
| API-010–API-018 | Versioning, error and privacy policy | API mapper/middleware | TEST-015–TEST-022, TEST-028 | H-006, H-008–H-010, H-014 |
| WEB-001–WEB-010 | ADR-002/003/004 | Web gate and assessment form | Component/E2E/accessibility tests | H-001–H-004, H-007, H-012 |
| WEB-011–WEB-017 | Response presentation | Result/chart components | TEST-023–TEST-026 | H-004, H-008, H-011, H-015 |
| WEB-018–WEB-022 | ADR-003 and privacy specification | State, service worker, telemetry, print | Cache/storage/log tests; TEST-028 | H-009, H-014 |
| SEC-001–SEC-006 | Internet threat model | Edge/TLS/WAF/API parsing | DAST, header, geo, parser and rate tests | H-010, H-012–H-014 |
| SEC-007–SEC-020 | Privacy, supply chain and incident model | Logs, CI, container, CSP, operations | TEST-027–TEST-029, penetration test | H-009–H-015 |
| SAFE-001–SAFE-004 | MHRA intended-purpose guidance | Claims and controlled documents | Regulatory/claims review | H-012 |
| SAFE-005–SAFE-009 | DCB0129 | Safety-management artefacts | CSO audit and safety case | All hazards |
| SAFE-010–SAFE-016 | NICE reuse policy and ADR-006/008 | Content pack, legal endpoint, geo controls | Rights and content review | H-005, H-006, H-012, H-013 |
| SAFE-017–SAFE-025 | Regulatory, clinical and release gates, including GB/NI territorial assessment | Readiness/release pipeline | Signed gate checklist and regulatory determination | All hazards |
| OPS-001–OPS-006 | Availability architecture | ECS/ALB/readiness | Resilience and SLO tests | H-010, H-013 |
| OPS-007–OPS-014 | Operations and rollback | Terraform, artifacts, logs, runbooks | Deployment rehearsal, TEST-028/029 | H-009, H-010, H-013, H-014 |
| OPS-015–OPS-021 | Rollout/decommissioning | Mode gate and controlled releases | Stage approval evidence | All hazards |
| TEST-001–TEST-005 | Source-control policy | Clinical-data ingestion | Signed double-entry evidence | H-005, H-006 |
| TEST-006–TEST-014 | Clinical maths/rules | Clinical-core test suites | CI results and clinical review | H-001–H-008, H-011 |
| TEST-015–TEST-022 | API contract | Contract test suite | CI results | H-008–H-010, H-014 |
| TEST-023–TEST-026 | Web workflow | Browser/accessibility suites | CI and manual review | H-001–H-004, H-009, H-010, H-015 |
| TEST-027–TEST-029 | Security/release integrity | Security pipeline | Scan, penetration and provenance evidence | H-013, H-014 |

## NICE clinical traceability

| Rule code/behaviour | NICE CG98 reference | Clinical requirement | Required test evidence | Hazard/control |
|---|---|---|---|---|
| Risk-factor display | 1.2.1, 1.2.9 | CLIN-028–CLIN-031 | At-risk display scenarios | H-007 |
| Visual inspection limitations and darker skin warning | 1.2.4–1.2.6 | PRD-021, PRD-022, CLIN-031 | Content and accessibility review | H-007, H-012 |
| `ADDITIONAL_VISUAL_INSPECTION_48H` | 1.2.1, 1.2.9 | CLIN-028–CLIN-031 | Every risk-factor combination | H-007 |
| `VISUAL_ASSESSMENT_LIMITATIONS` | 1.2.4–1.2.6 | PRD-021, PRD-022, CLIN-031 | Content, darker-skin and unknown-state cases | H-007, H-012 |
| `BREASTFEEDING_SUPPORT` | 1.2.2, 1.3.2–1.3.3 | Clinical supporting information | Breastfeeding present/absent/unknown cases | H-007, H-012 |
| `PARENT_CARER_INFORMATION` | 1.1.1, 1.3.1 | Clinical supporting information | Content and readability review | H-012, H-015 |
| `NO_ROUTINE_BILIRUBIN` | 1.2.7 | CLIN-028–CLIN-031 | Visible absent/present/unknown cases | H-007 |
| `DO_NOT_USE_PREDICTION_TESTS` | 1.2.8 | Measurement supporting information | Content and scope review | H-007, H-012 |
| `EARLY_JAUNDICE_MEASURE_2H` | 1.2.10 | CLIN-028, CLIN-051 | 1,439/1,440/1,441-minute cases | H-001, H-007 |
| `EARLY_JAUNDICE_REPEAT_6H` | 1.2.11 | CLIN-028, CLIN-037 | Below/rising/stable/falling cases | H-007, H-008 |
| `EARLY_JAUNDICE_MEDICAL_REVIEW_6H` | 1.2.12 | CLIN-028 | Early-jaundice scenario | H-007 |
| Threshold interpretation by age | 1.2.13, 1.3.4 | CLIN-007–CLIN-027 | TEST-001–TEST-014 | H-001, H-002, H-005 |
| `JAUNDICE_MEASURE_6H` | 1.2.14 | CLIN-028, CLIN-051 | 1,440/1,441-minute cases | H-001, H-007 |
| `SERUM_REQUIRED_AGE` | 1.2.15 | CLIN-028, CLIN-051 | 1,439/1,440/1,441-minute cases | H-004 |
| `SERUM_REQUIRED_GESTATION` | 1.2.15 | CLIN-028 | 34/35-week boundary | H-002, H-004 |
| `TCB_INITIAL_ALLOWED` | 1.2.16 | CLIN-028–CLIN-029 | Eligible/ineligible method cases | H-004 |
| `SERUM_CONFIRM_TCB_250` | 1.2.16 | CLIN-029 | 249/250/251 cases | H-004 |
| `SERUM_CONFIRM_TREATMENT_LINE` | 1.2.16 | CLIN-024, CLIN-029 | Below/at/above TcB cases | H-004, H-005 |
| `SERUM_REQUIRED_SUBSEQUENT` | 1.2.16 | CLIN-028–CLIN-029 | Prior line reached and each treatment state | H-004, H-008 |
| `NO_ICETEROMETER` | 1.2.17 | Measurement supporting information | Content review | H-004, H-012 |
| Total bilirubin/no albumin ratio | 1.3.5–1.3.6 | CLIN-014–CLIN-016 | Field and calculation tests | H-005 |
| `RETEST_WITHIN_18H` | 1.4.1 | CLIN-032–CLIN-034 | All eligibility and risk combinations | H-007, H-008 |
| `RETEST_WITHIN_24H` | 1.4.1 | CLIN-032–CLIN-034 | All eligibility and risk combinations | H-007, H-008 |
| `NO_ROUTINE_REPEAT` | 1.4.2 | CLIN-032–CLIN-034 | 50/51-below and unknown-well cases | H-007, H-008 |
| `RETEST_INTERVAL_LOCAL_PROTOCOL` | 1.4.1–1.4.2 | CLIN-032–CLIN-034 | Every ineligible repeat population | H-007, H-008 |
| `START_PHOTOTHERAPY` | 1.4.3, 1.4.8 | CLIN-024–CLIN-026, CLIN-038–CLIN-040 | Serum below/at/above lines | H-004, H-005, H-008 |
| `AT_TREATMENT_LINE_REVIEW` | Threshold table, 1.2.16, 1.3.4 | CLIN-024, CLIN-029 | Exact rational equality for serum and TcB | H-004, H-005 |
| `PHOTOTHERAPY_CHECK_4_6H` | 1.4.4 | CLIN-038–CLIN-040 | Start/time boundary scenarios | H-001, H-008 |
| `PHOTOTHERAPY_CHECK_OVERDUE` | 1.4.4, 1.4.9 | CLIN-039–CLIN-040 | No-result cases at 359/360/361 minutes after start | H-001, H-008 |
| `PHOTOTHERAPY_RESPONSE_INCOMPLETE` | 1.4.4, 1.4.9 | CLIN-039–CLIN-040 | Missing baseline/start/post-start combinations | H-001, H-007, H-008 |
| `PHOTOTHERAPY_CHECK_6_12H` | 1.4.4 | CLIN-037 | Stable/falling/rising scenarios | H-008 |
| `STOP_PHOTOTHERAPY` | 1.4.5 | CLIN-038–CLIN-040 | 49/50/51-below scenarios | H-005, H-008 |
| `REBOUND_CHECK_12_18H` | 1.4.6 | CLIN-038–CLIN-040 | Post-treatment-state scenarios | H-008 |
| `DO_NOT_USE_SUNLIGHT` | 1.4.7 | Treatment information | Content review | H-012 |
| `CONSIDER_INTENSIFIED_PHOTOTHERAPY` | 1.4.9 | CLIN-035–CLIN-040 | Rapid/proximity/failure branches | H-005, H-008 |
| `REDUCE_PHOTOTHERAPY_INTENSITY` | 1.4.10 | CLIN-038–CLIN-040 | 49/50/51-below exchange | H-008 |
| `PHOTOTHERAPY_CARE_INFORMATION` | 1.4.11–1.4.19 | Supporting information | Clinical content review | H-012, H-015 |
| `INCREASED_KERNICTERUS_RISK` | 1.5.1 | CLIN-035–CLIN-037, CLIN-041 | 340/341, gestation and trend cases | H-005, H-008 |
| `AT_RAPID_RISE_BOUNDARY` | 1.4.9, 1.5.1 | CLIN-035–CLIN-036 | Exact rate of 8.5 µmol/L/hour | H-005, H-008 |
| `INCOMPLETE_DANGER_ASSESSMENT` | 1.2.3, 1.5.1 plus product safety policy | CLIN-030, CLIN-050 | Each danger field unknown | H-007, H-008 |
| `ACUTE_BILIRUBIN_ENCEPHALOPATHY_EMERGENCY` | 1.5.1, 1.9.2 | CLIN-041–CLIN-043 | Present/absent/unknown and line combinations | H-007, H-008 |
| `ASSESS_UNDERLYING_DISEASE` | 1.6.1–1.6.2 | Underlying-disease section | Above-line and infection scenarios | H-008 |
| `PROLONGED_JAUNDICE_ASSESSMENT` | 1.7.1 | CLIN-044–CLIN-046 | 14/21-day and UTI cases | H-001, H-007 |
| `EXPERT_LIVER_ADVICE` | 1.7.2 | CLIN-044–CLIN-046 | 24/25/26 cases | H-007, H-008 |
| `AT_CONJUGATED_BOUNDARY_REVIEW` | 1.7.2 plus equality policy | CLIN-044–CLIN-046 | Exact value 25 µmol/L | H-007, H-008 |
| `IVIG_SPECIALIST_PATHWAY` | 1.8.1 | CLIN-035–CLIN-040 | All conjunctive preconditions | H-008 |
| `IVIG_INFORMATION` | 1.8.2 | Supporting information | Content and pathway-presence review | H-012, H-015 |
| `EXCHANGE_TRANSFUSION_ESCALATION` | 1.9.2–1.9.4 | CLIN-041–CLIN-043 | Numeric/encephalopathy/post-exchange cases | H-005, H-008 |
| `AT_EXCHANGE_LINE_EMERGENCY_REVIEW` | Threshold table, 1.2.16 | CLIN-024–CLIN-026, CLIN-041–CLIN-043 | Exact rational equality for serum and TcB | H-004, H-005, H-008 |
| `EXCHANGE_TRANSFUSION_INFORMATION` | 1.9.1 | Supporting information | Content and pathway-presence review | H-012, H-015 |

## Source and decision references

- NICE CG98 recommendations: <https://www.nice.org.uk/guidance/cg98/chapter/recommendations>
- NICE threshold resource: <https://www.nice.org.uk/guidance/cg98/resources/treatment-threshold-graphs-excel-544300525>
- NICE UK Open Content Licence: <https://www.nice.org.uk/reusing-our-content/nice-uk-open-content-licence>
- MHRA software guidance: <https://www.gov.uk/government/publications/software-and-artificial-intelligence-ai-as-a-medical-device>
- DCB0129: <https://digital.nhs.uk/data-and-information/information-standards/governance/latest-activity/standards-and-collections/dcb0129-clinical-risk-management-its-application-in-the-manufacture-of-health-it-systems/>
- DCB0160: <https://digital.nhs.uk/data-and-information/information-standards/governance/latest-activity/standards-and-collections/dcb0160-clinical-risk-management-its-application-in-the-deployment-and-use-of-health-it-systems>
- Architecture decisions: [`decisions/`](decisions/README.md)
