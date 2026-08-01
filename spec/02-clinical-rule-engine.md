# Clinical rule engine

## Clinical baseline

The normative clinical baseline is NICE guideline CG98, “Jaundice in newborn babies under 28 days”, published 19 May 2010 and last updated 31 October 2023. The rule pack implements current recommendations and the official treatment threshold graphs.

The engine is deterministic decision support. It reports what follows from supplied facts; it does not infer facts that were not supplied, diagnose an underlying disease, or resolve differences between NICE and an approved local protocol.

## Rule-pack identity and lifecycle

| ID | Requirement |
|---|---|
| CLIN-001 | Clinical rules MUST be loaded from one immutable rule pack identified by `nice-cg98-2023-10-31.1`. |
| CLIN-002 | A rule pack MUST record its source URLs, source update date, retrieval date, source hashes, author, reviewers, clinical status and superseded pack where applicable. |
| CLIN-003 | Only a pack with status `active` may serve clinical-mode evaluations. |
| CLIN-004 | Demonstration mode MAY use a `draft` pack but MUST label every response as not for patient care. |
| CLIN-005 | Changing a threshold, comparator, formula, recommendation mapping, priority or display action MUST create a new rule-pack revision. |
| CLIN-006 | The engine MUST evaluate the exact rule-pack ID supplied by the client; unavailable, retired or non-active packs MUST produce a conflict response and identify the active pack. |

## Input interpretation

### Age and gestation

| ID | Requirement |
|---|---|
| CLIN-007 | Gestational age MUST be supplied as completed whole weeks at birth from 23 through 42. |
| CLIN-008 | Gestational age MUST NOT be rounded upward based on additional gestational days. |
| CLIN-009 | The baby's actual gestational age at birth MUST select the treatment curve through age 14 days; corrected gestational age MUST NOT be used. |
| CLIN-010 | Assessment and measurement ages MUST be elapsed whole minutes from birth. |
| CLIN-011 | Assessment age MUST be from 0 through 40,319 minutes. |
| CLIN-012 | A treatment line MUST be returned only from 0 through 20,160 minutes inclusive. |
| CLIN-013 | Measurement ages MUST be unique, non-negative and no later than assessment age. |

### Bilirubin values

| ID | Requirement |
|---|---|
| CLIN-014 | Bilirubin input MUST be total bilirubin in integer µmol/L from 0 through 1,000. |
| CLIN-015 | The engine MUST NOT subtract conjugated bilirubin from total bilirubin for treatment decisions. |
| CLIN-016 | The engine MUST NOT use the albumin/bilirubin ratio for management decisions. |
| CLIN-017 | A treatment-line action MUST NOT be based on a transcutaneous measurement alone. |

## Treatment threshold mathematics

All internal threshold calculations use a rational value `(numerator, denominator)`. A bilirubin measurement `b` is compared using `b × denominator` against `numerator`; a rounded display threshold MUST NOT drive the decision.

The API displays thresholds rounded to one decimal place using round-half-away-from-zero. It also returns the exact relationship and signed distance calculated before display rounding.

### Gestational age 23–37 weeks

Let:

- `g` be completed gestational weeks;
- `m` be postnatal age in minutes;
- `P72 = (10 × g) − 100`; and
- `E72 = 10 × g`.

Phototherapy threshold:

```text
if 0 <= m < 4320:
    P(m,g) = 40 + ((P72 - 40) × m / 4320)
if 4320 <= m <= 20160:
    P(m,g) = P72
```

Exchange-transfusion threshold:

```text
if 0 <= m < 4320:
    E(m,g) = 80 + ((E72 - 80) × m / 4320)
if 4320 <= m <= 20160:
    E(m,g) = E72
```

| ID | Requirement |
|---|---|
| CLIN-018 | The engine MUST implement the preterm formulas exactly as shown without intermediate rounding. |
| CLIN-019 | At birth the preterm phototherapy and exchange thresholds MUST be 40 and 80 µmol/L respectively for every supported gestation below 38 weeks. |
| CLIN-020 | At and after 72 hours the thresholds MUST remain at `P72` and `E72` through 14 days. |

### Gestational age 38 weeks or more

All gestations from 38 through 42 use the same control points.

| Age (hours) | Phototherapy | Exchange transfusion |
|---:|---:|---:|
| 0 | 100 | 100 |
| 6 | 125 | 150 |
| 12 | 150 | 200 |
| 18 | 175 | 250 |
| 24 | 200 | 300 |
| 30 | 212 | 350 |
| 36 | 225 | 400 |
| 42 | 237 | 450 |
| 48 | 250 | 450 |
| 54 | 262 | 450 |
| 60 | 275 | 450 |
| 66 | 287 | 450 |
| 72 | 300 | 450 |
| 78 | 312 | 450 |
| 84 | 325 | 450 |
| 90 | 337 | 450 |
| 96 | 350 | 450 |
| 96–336 | 350 | 450 |

Between control points, the threshold is the straight-line interpolation used by the treatment graph. For two points `(m1,v1)` and `(m2,v2)`, the exact value is `v1 + ((v2-v1) × (m-m1)/(m2-m1))`.

| ID | Requirement |
|---|---|
| CLIN-021 | The engine MUST implement every term control point exactly as shown. |
| CLIN-022 | The engine MUST interpolate between control points using elapsed minutes and exact rational arithmetic. |
| CLIN-023 | The term phototherapy line MUST remain 350 and the exchange line 450 µmol/L after 96 hours through 14 days. |

### Threshold relationship and equality

The NICE threshold table expresses treatment actions for a bilirubin value greater than the line. Bili Mate preserves three states to make the boundary visible:

| Relationship | Exact comparison | Behaviour |
|---|---|---|
| `below` | value < line | Apply below-line monitoring rules |
| `at` | value = line | Return `AT_TREATMENT_LINE_REVIEW`; require serum confirmation if input is TcB and urgent clinician review |
| `above` | value > line | Apply the treatment action for the exceeded line, using a serum value |

| ID | Requirement |
|---|---|
| CLIN-024 | Exact equality MUST NOT be rounded into either `below` or `above`. |
| CLIN-025 | If both lines are exceeded, exchange-transfusion escalation MUST take precedence over phototherapy. |
| CLIN-026 | The response MUST include the unrounded signed distance to each applicable line. |
| CLIN-027 | No line or distance MUST be extrapolated after 20,160 minutes. |

## Recognition and measurement pathway

| Code | Condition | Required engine output | NICE reference |
|---|---|---|---|
| `EARLY_JAUNDICE_MEASURE_2H` | Suspected or obvious jaundice and age ≤24 h | Urgent serum bilirubin within 2 h | 1.2.10 |
| `EARLY_JAUNDICE_REPEAT_6H` | Same, until below threshold and stable/falling | Serum bilirubin every 6 h | 1.2.11 |
| `EARLY_JAUNDICE_MEDICAL_REVIEW_6H` | Suspected or obvious jaundice and age ≤24 h | Medical review as soon as possible and within 6 h | 1.2.12 |
| `JAUNDICE_MEASURE_6H` | Suspected or obvious jaundice and age >24 h | Bilirubin measurement within 6 h | 1.2.14 |
| `SERUM_REQUIRED_AGE` | Age ≤24 h | Use serum | 1.2.15 |
| `SERUM_REQUIRED_GESTATION` | Gestation <35 weeks | Use serum | 1.2.15 |
| `TCB_INITIAL_ALLOWED` | Gestation ≥35 weeks, age >24 h and no later disqualifier | TcB preferred; serum if unavailable | 1.2.16 |
| `SERUM_CONFIRM_TCB_250` | TcB >250 µmol/L | Confirm with serum | 1.2.16 |
| `SERUM_CONFIRM_TREATMENT_LINE` | TcB at or above relevant treatment line | Confirm with serum | 1.2.16 |
| `SERUM_REQUIRED_SUBSEQUENT` | A result has reached a treatment line or treatment has begun | Use serum for all subsequent measurements | 1.2.16 |
| `NO_ROUTINE_BILIRUBIN` | No visible/suspected jaundice | Do not routinely measure | 1.2.7 |

Additional risk factors reported to the user are gestation under 38 weeks, previous sibling requiring phototherapy, intention to breastfeed exclusively, and visible jaundice in the first 24 hours. Exclusive breastfeeding MUST produce supportive feeding advice, not advice to stop breastfeeding.

To avoid an unhandled instant between NICE's “first 24 hours” and “more than 24 hours” wording, Bili Mate assigns exact age 1,440 minutes to the conservative first-day pathway. The more-than-24-hours pathway starts at 1,441 minutes. This product interpretation requires explicit approval in the clinical rule-pack review.

| ID | Requirement |
|---|---|
| CLIN-028 | Measurement-method rules MUST be evaluated before treatment recommendations. |
| CLIN-029 | A TcB at or above a line MUST produce confirmation and escalation guidance but MUST NOT produce a definitive treatment instruction. |
| CLIN-030 | If a danger-sign field is `unknown`, the engine MUST return `INCOMPLETE_DANGER_ASSESSMENT` and MUST NOT make “no routine action” primary. |
| CLIN-031 | The engine MUST include the darker-skin recognition warning whenever visual assessment guidance is shown. |
| CLIN-051 | At exactly 1,440 minutes, the engine MUST apply the first-day serum and urgent-review pathway; it MUST apply the more-than-24-hours pathway from 1,441 minutes. |

## Below-line monitoring before phototherapy

These specific repeat rules apply only when the baby is clinically well, gestation is at least 38 weeks, age is greater than 24 hours, and bilirubin is below the phototherapy threshold.

| Condition | Output |
|---|---|
| Distance below line is greater than 0 and no more than 50 µmol/L, and previous sibling needed phototherapy or exclusive breastfeeding is intended | Repeat within 18 h |
| Distance below line is greater than 0 and no more than 50 µmol/L, without either risk factor | Repeat within 24 h |
| Distance below line is greater than 50 µmol/L | Do not routinely repeat |

At exactly 50 µmol/L below the line, the “within 50” branch applies. A value exactly on the line is not a below-line case and follows the equality policy instead. For a baby outside this population, the engine MUST NOT extrapolate these intervals and returns `RETEST_INTERVAL_LOCAL_PROTOCOL`.

| ID | Requirement |
|---|---|
| CLIN-032 | The 18/24-hour rules MUST require all eligibility conditions above. |
| CLIN-033 | Clinical state `unknown` MUST prevent the engine from choosing the no-routine-repeat branch. |
| CLIN-034 | Phototherapy MUST NOT be recommended for a confirmed serum value below the phototherapy line solely because it is close to the line. |

## Trend calculation

Measurements are sorted by age after validation. The engine selects the two most recent measurements with distinct ages for display trend; only a pair in which both measurements are serum can confirm the clinical rapid-rise rule.

```text
rate_umol_l_per_hour = (new_value - old_value) × 60 / (new_age_minutes - old_age_minutes)
```

The engine compares an exact rational serum rate with 8.5 µmol/L/hour. A positive serum rate strictly greater than 8.5 is rapid. Exactly 8.5 is not greater and returns `AT_RAPID_RISE_BOUNDARY`. NICE does not specify a minimum elapsed interval for this comparison, so Bili Mate MUST NOT invent one; it displays the interval and applies the rule to any two serum measurements at distinct valid ages.

| ID | Requirement |
|---|---|
| CLIN-035 | Rate comparisons MUST use exact arithmetic without rounding. |
| CLIN-036 | TcB measurements MUST NOT be used to confirm the rapid-rise rule. |
| CLIN-037 | Equal consecutive serum values are `stable`; a lower value is `falling`; a higher value is `rising`. |

## Treatment pathway

### Phototherapy

| Code | Condition | Output | NICE reference |
|---|---|---|---|
| `START_PHOTOTHERAPY` | Serum bilirubin above phototherapy and below exchange line | Start phototherapy | 1.4.3, 1.4.8 |
| `PHOTOTHERAPY_CHECK_4_6H` | Phototherapy started less than or equal to 6 h ago and no qualifying post-start result exists | Repeat serum bilirubin 4–6 h after initiation | 1.4.4 |
| `PHOTOTHERAPY_CHECK_OVERDUE` | More than 6 h after start with no qualifying post-start serum result | Monitoring is overdue; obtain urgent serum bilirubin and review locally | 1.4.4, 1.4.9 |
| `PHOTOTHERAPY_RESPONSE_INCOMPLETE` | Treatment is active but the submitted history cannot compare a baseline with a post-start serum result | State that response cannot be assessed; obtain/review serum results and follow local escalation | 1.4.4, 1.4.9 |
| `PHOTOTHERAPY_CHECK_6_12H` | Serum is stable/falling during phototherapy | Repeat every 6–12 h | 1.4.4 |
| `STOP_PHOTOTHERAPY` | During phototherapy and serum is at least 50 µmol/L below phototherapy line | Stop phototherapy | 1.4.5 |
| `REBOUND_CHECK_12_18H` | Phototherapy stopped | Repeat serum bilirubin 12–18 h later | 1.4.6 |
| `DO_NOT_USE_SUNLIGHT` | Treatment information is displayed | Do not use sunlight as treatment | 1.4.7 |

The engine does not select equipment, irradiance or nursing configuration. It supplies the applicable CG98 care checklist as supporting information.

### Intensified phototherapy

Return `CONSIDER_INTENSIFIED_PHOTOTHERAPY` when any confirmed condition applies:

1. serum bilirubin is rising strictly faster than 8.5 µmol/L/hour;
2. age is at least 72 hours and serum bilirubin is no more than 50 µmol/L below the exchange line; or
3. the first submitted post-start serum result continues to rise or does not fall from the submitted baseline serum result. A result obtained after the six-hour monitoring deadline still activates this branch and also produces `PHOTOTHERAPY_CHECK_OVERDUE` where the timely result was missed.

At exactly 50 µmol/L below exchange, the proximity condition applies. If intensified phototherapy is active and serum falls to at least 50 µmol/L below exchange, return `REDUCE_PHOTOTHERAPY_INTENSITY`.

| ID | Requirement |
|---|---|
| CLIN-038 | A proximity-to-exchange trigger MUST require age of at least 72 hours. |
| CLIN-039 | Failure-to-respond MUST compare a baseline serum result with a later serum result after treatment starts; it MUST NOT wait until six hours if a valid earlier result already shows that bilirubin has risen or not fallen. |
| CLIN-040 | An unknown treatment start age, missing baseline or missing post-start serum result MUST prevent failure-to-respond classification and return missing-information or overdue-monitoring guidance. |

### Kernicterus and emergency escalation

The engine returns `INCREASED_KERNICTERUS_RISK` if any applies:

- serum bilirubin is greater than 340 µmol/L and gestation is at least 37 weeks;
- confirmed serum rise is greater than 8.5 µmol/L/hour; or
- clinical features of acute bilirubin encephalopathy are present.

Acute bilirubin encephalopathy is an emergency regardless of the calculated bilirubin line.

### Exchange transfusion

Return `EXCHANGE_TRANSFUSION_ESCALATION` if a serum bilirubin is above the exchange line or acute bilirubin encephalopathy is present. The action states that double-volume exchange transfusion is indicated and urgent neonatal intensive-care escalation is required. Supporting actions are continuous intensified phototherapy while preparing, and following exchange, continued intensified phototherapy plus serum bilirubin within two hours.

At the exchange line, return `AT_EXCHANGE_LINE_EMERGENCY_REVIEW`, immediate senior review and serum confirmation where needed; do not misclassify equality as above.

| ID | Requirement |
|---|---|
| CLIN-041 | Encephalopathy MUST outrank every numeric-line recommendation. |
| CLIN-042 | Exchange escalation MUST outrank intensified and standard phototherapy. |
| CLIN-043 | The engine MUST NOT generate a transfusion order, calculate blood volume or imply that preparation can wait for another API result. |

### IVIG

Return `IVIG_SPECIALIST_PATHWAY` only when all are present:

- rhesus or ABO haemolytic disease;
- continuous intensified phototherapy; and
- confirmed serum rise greater than 8.5 µmol/L/hour.

The referenced NICE information is 500 mg/kg over four hours as an adjunct. The result MUST be marked specialist-prescribing information, not an order, and MUST require neonatal specialist confirmation.

## Assessment for underlying disease

When significant hyperbilirubinaemia is present—that is, a serum result above a treatment threshold—the engine returns `ASSESS_UNDERLYING_DISEASE` with:

- serum bilirubin baseline;
- blood packed cell volume;
- mother and baby blood groups; and
- DAT, interpreted in the context of reaction strength and maternal prophylactic anti-D.

It also returns conditional considerations for full blood count and blood film, G6PD testing taking ethnic origin into account, and blood/urine/cerebrospinal-fluid cultures if infection is suspected.

## Prolonged jaundice

Prolonged-jaundice assessment applies when visible jaundice persists:

- beyond 14 days for gestation at least 37 weeks; or
- beyond 21 days for gestation below 37 weeks.

“Beyond” is strict: at exactly 14 or 21 days the corresponding rule is not yet active.

The output includes:

- check for pale chalky stools and dark urine staining the nappy;
- measure conjugated bilirubin;
- full blood count;
- mother and baby blood groups and DAT;
- urine culture only if urinary tract infection is clinically suspected; and
- confirmation that routine metabolic screening, including congenital hypothyroidism screening, occurred.

Conjugated bilirubin strictly greater than 25 µmol/L returns `EXPERT_LIVER_ADVICE`. Exactly 25 returns `AT_CONJUGATED_BOUNDARY_REVIEW`.

| ID | Requirement |
|---|---|
| CLIN-044 | Prolonged-jaundice recommendations MUST NOT include a treatment threshold after 14 days. |
| CLIN-045 | Pale stool, dark urine or conjugated bilirubin over 25 MUST prevent a reassuring primary action. |
| CLIN-046 | Urine culture MUST be suggested only when UTI is suspected. |

## Priority and conflict resolution

Priority order is fixed:

1. `emergency`: acute bilirubin encephalopathy or exchange escalation;
2. `immediate`: exchange-line equality, high kernicterus risk or specialist liver warning;
3. `urgent`: jaundice in first 24 hours, rapid rise or intensified phototherapy;
4. `treatment`: standard phototherapy and treatment monitoring;
5. `timed`: measurement and repeat-testing intervals;
6. `routine`: assessment, education and care checklists.

| ID | Requirement |
|---|---|
| CLIN-047 | Exactly one recommendation MUST be selected as `primary_action` using priority, then the stable rule order in the clinical YAML. |
| CLIN-048 | Lower-priority recommendations that contradict the primary action MUST be suppressed and recorded in the decision trace as suppressed rule codes. |
| CLIN-049 | Non-contradictory supporting actions MUST remain visible. |
| CLIN-050 | A result MUST include all supplied unknowns and all rules not evaluated because required information was missing. |

## Universal notices

Every successful evaluation MUST contain:

- the intended-user limitation;
- the current NICE attribution and non-endorsement disclaimer;
- the local assay-variability warning;
- a statement that clinical judgement and local policy remain necessary;
- the rule-pack source update date; and
- an explicit statement when demonstration mode is active.
