# Clinical safety and regulation

## Safety status

Bili Mate influences assessment, monitoring and treatment decisions. Development therefore treats it as potential Software as a Medical Device and health IT capable of contributing to patient harm. The final regulatory qualification and class must be documented by a competent UK regulatory professional; it is not assumed by this specification.

No disclaimer can replace correct design, clinical evidence, risk management or regulatory compliance.

## Intended purpose and claims control

| ID | Requirement |
|---|---|
| SAFE-001 | The intended-purpose statement in [product requirements](01-product-requirements.md) MUST be identical in labelling, UI, API, clinical evaluation and regulatory documentation. |
| SAFE-002 | Marketing MUST NOT claim autonomous diagnosis, guaranteed error prevention, NICE/NHS endorsement, parent suitability or use outside the validated population. |
| SAFE-003 | Every product claim MUST map to documented verification and, where clinical, clinical evidence. |
| SAFE-004 | Expansion to another audience, jurisdiction, age, guideline, unit, input source or clinical integration is a change to intended purpose and requires fresh regulatory and safety assessment. |

## Regulatory qualification gate

Before any advisory pilot that can influence care:

1. document the intended purpose, indications, users, patient population, use environment, inputs, outputs and clinical workflow;
2. complete the current MHRA software qualification and classification assessment;
3. obtain external regulatory review and retain its rationale;
4. define the legal manufacturer;
5. determine required conformity assessment, registration, quality-management and post-market obligations; and
6. ensure every obligation is represented in the release checklist.

If qualification is unresolved, the service remains demonstration/shadow-only and output cannot influence patient care.

## DCB0129 manufacturer safety case

The legal manufacturer must appoint a trained Clinical Safety Officer and produce:

- Clinical Risk Management Plan;
- Clinical Risk Management System evidence;
- hazard identification workshop records;
- Hazard Log;
- risk acceptability matrix and rationale;
- Clinical Safety Case Report;
- safety requirements and traceability;
- verification evidence for risk controls;
- known limitations and residual risks;
- release safety statement; and
- lifecycle, incident, change and decommissioning procedures.

| ID | Requirement |
|---|---|
| SAFE-005 | The Hazard Log MUST be maintained from the first executable prototype through decommissioning. |
| SAFE-006 | Each hazard MUST record cause, hazardous situation, foreseeable sequence, harm, initial risk, controls, verification, residual risk, owner and status. |
| SAFE-007 | Software requirements that control a hazard MUST be referenced from the Hazard Log and [traceability matrix](traceability.md). |
| SAFE-008 | The Clinical Safety Officer MUST approve the safety case and each clinical-mode release. |
| SAFE-009 | Residual risk requiring a deploying organisation's control MUST be stated in deployment documentation and communicated for DCB0160 review. |

## Initial hazard set

| Hazard ID | Hazardous situation | Potential harm | Principal controls |
|---|---|---|---|
| H-001 | Wrong postnatal age used | Incorrect threshold and delayed/unnecessary treatment | Local timestamp review, elapsed-minute display, range checks, tests |
| H-002 | Gestation rounded or corrected | Wrong treatment curve | Completed-week label, validation, actual-gestation rule |
| H-003 | mg/dL entered as µmol/L | Grossly incorrect classification | One unit only, fixed suffix, plausible-range warning, review summary |
| H-004 | TcB treated as definitive serum | Treatment error | Typed method, serum eligibility flag, confirmation rules |
| H-005 | Threshold data/formula incorrect | Systematic incorrect recommendations | Double entry, exact oracle tests, hashes, clinical sign-off |
| H-006 | Guidance becomes stale | Obsolete care advice | Weekly monitor, visible source date, governed pack update |
| H-007 | Missing/unknown danger sign treated absent | False reassurance | Required tri-state, unknown blocks reassuring primary action |
| H-008 | Rule conflict hides urgent action | Delayed escalation | Fixed priorities, suppression trace, branch and scenario tests |
| H-009 | Cached/previous response reused | Wrong patient/context applied | No-store, no local persistence, unique receipt/version display |
| H-010 | API outage appears as normal result | Clinician delays local workflow | Fail closed, explicit no-result screen, local-protocol direction |
| H-011 | Assay variation ignored | Threshold misinterpretation | Universal assay warning, local pathology approval |
| H-012 | Parent uses public calculator | Unsafe self-management | Intended-user gate, no parent workflow, professional labelling |
| H-013 | Rule pack tampered or wrong pack active | Incorrect output | Artifact hash, startup self-test, release authorisation, rollback |
| H-014 | Clinical content leaks through logs | Confidentiality harm | Schema excludes identifiers, body logging prohibited, log tests |
| H-015 | Display chart disagrees with textual result | Misinterpretation | Same server data, accessible table, visual regression tests |

The full Hazard Log is a controlled lifecycle artefact, not replaced by this initial list.

## Clinical validation

Clinical validation must demonstrate:

- faithful representation of current CG98 recommendations;
- exact threshold equivalence to the official source across the supported range;
- correct rule priority and safe handling of ambiguity;
- correct presentation to each intended professional group;
- correct limitations for assay variation, local protocol and out-of-scope cases; and
- acceptable performance in the intended clinical workflow.

The clinical owner and a second independent neonatal clinician must approve the normalised rule data and at least 60 end-to-end scenarios. Reviewers may not both be the engineer who transcribed the rules.

## NICE content and provenance

| ID | Requirement |
|---|---|
| SAFE-010 | CG98 recommendations and the official treatment resource MUST be the only NICE clinical content represented in v1. |
| SAFE-011 | CKS wording MUST NOT be copied without a separate licence from its rights holder. |
| SAFE-012 | A written content-rights determination MUST confirm how the product may encode NICE decision logic, present computed or paraphrased action text, and encode/display the threshold workbook, including any third-party material. |
| SAFE-013 | The prescribed NICE attribution and disclaimer MUST be reproduced as required by the current UK Open Content Licence. |
| SAFE-014 | The product MUST state the NICE information was accurate for the source version and may be updated or withdrawn. |
| SAFE-015 | NICE logos and language implying endorsement MUST NOT be used. |
| SAFE-016 | NICE-derived content and the clinical API MUST be distributed for UK use only unless an international licence is obtained. |

The release owner should use NICE's content assurance service before pilot where feasible. Any required verbatim recommendation text is stored in the licensed content pack, separated from Bili Mate's computed statements.

## Guideline surveillance and update

A weekly automated monitor checks the CG98 overview, recommendations and resources metadata for update date, ETag/content hash and withdrawal state. It may only notify; it never changes production rules.

When a change is detected:

1. place the alert in clinical review within one working day;
2. clinical owner assesses safety impact and urgency;
3. suspend clinical mode if current advice may be unsafe;
4. create a new candidate rule pack;
5. repeat source transcription, clinical review, safety analysis and regression testing;
6. update user-facing content and traceability;
7. approve and deploy through canary release; and
8. retain the previous pack and decision compatibility information.

## Deploying organisation: DCB0160

The pilot organisation owns its DCB0160 deployment and must:

- appoint its Clinical Safety Officer;
- assess local workflow, users, devices, connectivity, pathology assays and local protocols;
- review the manufacturer safety case and residual risks;
- define training, standard operating procedures, downtime and escalation;
- validate the product in the intended unit;
- approve deployment and monitor use; and
- manage local incidents, changes and decommissioning.

Bili Mate must supply a deployment safety pack sufficient for this assessment. Manufacturer DCB0129 evidence does not replace local DCB0160 obligations.

## DTAC readiness

The supplier evidence pack covers:

- clinical safety;
- data protection, including DPIA support and data-flow map;
- technical security and penetration-test summary;
- interoperability statement, including the explicit absence of integration in v1;
- usability evidence with intended users; and
- WCAG 2.2 AA accessibility evidence.

## Safety release gate

Clinical mode cannot be enabled until all are true:

| ID | Gate |
|---|---|
| SAFE-017 | Regulatory qualification/classification rationale is approved. |
| SAFE-018 | NICE and third-party content rights are confirmed. |
| SAFE-019 | Rule pack has two clinical approvals and status `active`. |
| SAFE-020 | DCB0129 safety case and Hazard Log are approved by the manufacturer CSO. |
| SAFE-021 | All critical and major safety defects are closed; residual risks are accepted. |
| SAFE-022 | Independent penetration, accessibility and clinical validation evidence is accepted. |
| SAFE-023 | Pilot organisation completes DCB0160 approval, local pathology review and downtime procedure. |
| SAFE-024 | Release authorisation identifies exact source commit, image digest, engine version and rule-pack digest. |
| SAFE-025 | The regulatory plan MUST separately assess placing the product into service or on the market in Great Britain and Northern Ireland; a release is enabled only in each territory whose current route and obligations are satisfied. |

The application validates the release-authorisation reference at startup. Absence or mismatch keeps readiness false.

## Post-release surveillance

The pilot provides visible routes for clinical-safety feedback and incidents. The manufacturer reviews:

- service and safety faults immediately;
- reported clinical discrepancies within one working day;
- aggregate operational trends weekly during pilot;
- hazard and residual-risk status at every release; and
- continued guidance and regulatory currency at least monthly.

Any incident that may have produced unsafe advice triggers clinical-mode suspension or rollback while impact is assessed.
