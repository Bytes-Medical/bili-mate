//! Deterministic rule evaluation (spec 02, spec 05 rule-evaluation flow):
//! thresholds → measurement classification → serum trend → rules in stable
//! order → priority sort → suppression → primary action and trace.

use core::cmp::Ordering;

use crate::catalog::RuleCode;
use crate::error::SafetyError;
use crate::input::Assessment;
use crate::output::{
    DecisionTrace, Display1Dp, EvaluationOutcome, ExactFraction, ExactThresholdTrace,
    MissingInformation, NormalisedInput, Recommendation, ThresholdAssessment, TrendAssessment,
    Warning, WarningCategory,
};
use crate::thresholds::{assess_against_line, treatment_thresholds, LineAssessment};
use crate::trend::{calculate_trend, Trend};
use crate::types::{
    MeasurementMethod, Mode, ThresholdRelation, TreatmentMode, TriState, FIRST_48_HOURS_MINUTES,
    PROLONGED_PRETERM_MINUTES, PROLONGED_TERM_MINUTES, TREATMENT_LINE_MAX_AGE_MINUTES,
};

/// Evaluation context supplied by the caller. Rule-pack resolution and
/// release gating happen outside the core (spec 05).
#[derive(Debug, Clone, Copy)]
pub struct EvaluationContext {
    pub mode: Mode,
}

/// Danger-sign fields whose `unknown` state blocks a reassuring primary
/// action (CLIN-030, PRD-025).
///
/// Product-policy interpretation pending clinical rule-pack review: the set
/// covers encephalopathy, overall clinical wellbeing, the liver danger signs
/// and suspected infection. `clinically_well` is included because CLIN-033
/// requires an unknown clinical state to block the no-routine-repeat branch.
const DANGER_SIGN_FIELDS: &[&str] = &[
    "acute_bilirubin_encephalopathy",
    "clinically_well",
    "pale_chalky_stools",
    "dark_urine_stains_nappy",
    "infection_suspected",
];

#[derive(Debug, Clone)]
struct Classified {
    id: String,
    age: u32,
    value: u16,
    method: MeasurementMethod,
    photo: Option<LineAssessment>,
    exch: Option<LineAssessment>,
}

/// Every predicate the rule table needs, precomputed so that exact-arithmetic
/// failures surface before any rule fires (API-011).
struct Facts {
    first_day: bool,
    any_measurement_exists: bool,
    any_serum_exists: bool,
    jaundice_suspected: bool,
    suspected_absent: bool,
    visible_jaundice: bool,
    visible_absent: bool,
    breastfeeding_intended: bool,
    abe_present: bool,
    gestation: u8,
    age: u32,

    latest: Option<Classified>,

    latest_photo_at: bool,
    latest_photo_below: bool,
    latest_exch_at: bool,
    latest_exch_above: bool,
    latest_below_within_50_photo: bool,
    latest_more_than_50_below_photo: bool,

    latest_serum_photo_above: bool,
    latest_serum_exch_above: bool,
    latest_serum_at_least_50_below_photo: bool,
    latest_serum_at_least_50_below_exch: bool,
    latest_serum_within_50_of_exch: bool,
    latest_serum_over_340: bool,

    any_serum_above_a_line: bool,
    any_at_or_above_photo: bool,

    trend: Option<Trend>,
    rapid_rise_confirmed: bool,
    rapid_rise_at_boundary: bool,
    trend_serum_stable_or_falling: bool,
    trend_stable_or_falling: bool,

    treatment_mode: TreatmentMode,
    phototherapy_active: bool,
    elapsed_since_start: Option<u32>,
    timely_post_start_serum_exists: bool,
    any_post_start_serum_exists: bool,
    response_comparison_available: bool,
    nonresponse_confirmed: bool,

    eligible_term_retest: bool,
    risk_factor_for_18h: bool,

    prolonged_criteria_met: bool,
    conjugated_over_25: bool,
    conjugated_exactly_25: bool,
    haemolytic_disease: bool,

    danger_unknown: Vec<&'static str>,
}

impl Facts {
    fn build(
        assessment: &Assessment,
        trend: Option<Trend>,
        classified: &[Classified],
    ) -> Result<Self, SafetyError> {
        let features = &assessment.clinical_features;
        let risks = &assessment.risk_factors;
        let age = assessment.assessment_age.value();
        let gestation = assessment.gestational_age.value();

        let latest = classified.last().cloned();
        let latest_serum = classified
            .iter()
            .rev()
            .find(|c| c.method == MeasurementMethod::Serum)
            .cloned();

        // Distance predicates on exact rationals. `distance` is measurement
        // minus threshold, so "X below the line" is a negative distance.
        let at_most_50_below = |line: &Option<LineAssessment>| -> Result<bool, SafetyError> {
            match line {
                Some(l) => Ok(l.distance.cmp_int(-50)? != Ordering::Less),
                None => Ok(false),
            }
        };
        let at_least_50_below = |line: &Option<LineAssessment>| -> Result<bool, SafetyError> {
            match line {
                Some(l) => Ok(l.distance.cmp_int(-50)? != Ordering::Greater),
                None => Ok(false),
            }
        };

        let latest_photo = latest.as_ref().and_then(|c| c.photo);
        let latest_exch = latest.as_ref().and_then(|c| c.exch);
        let latest_serum_photo = latest_serum.as_ref().and_then(|c| c.photo);
        let latest_serum_exch = latest_serum.as_ref().and_then(|c| c.exch);

        let latest_photo_below = latest_photo.map(|l| l.relation) == Some(ThresholdRelation::Below);
        // "Within 50" means strictly below the line by no more than 50
        // (spec 02: at exactly 50 below, the within-50 branch applies).
        let latest_below_within_50_photo = latest_photo_below && at_most_50_below(&latest_photo)?;
        let latest_more_than_50_below_photo =
            latest_photo_below && !at_most_50_below(&latest_photo)?;

        let treatment_mode = assessment.treatment_state.mode;
        let phototherapy_active = matches!(
            treatment_mode,
            TreatmentMode::Phototherapy | TreatmentMode::IntensifiedPhototherapy
        );
        let start = assessment
            .treatment_state
            .started_age_minutes
            .map(|a| a.value());
        let elapsed_since_start = match (phototherapy_active, start) {
            (true, Some(s)) => Some(age - s),
            _ => None,
        };

        // "Qualifying" post-start serum reconciles spec 02 with the rule
        // pack: a post-start result obtained after the six-hour deadline
        // still supports response assessment but does not clear the overdue
        // flag, so qualifying means obtained within six hours of the start.
        let (timely_post_start_serum_exists, any_post_start_serum_exists) = match start {
            Some(s) => {
                let timely = classified
                    .iter()
                    .any(|c| c.method == MeasurementMethod::Serum && c.age > s && c.age <= s + 360);
                let any = classified
                    .iter()
                    .any(|c| c.method == MeasurementMethod::Serum && c.age > s);
                (timely, any)
            }
            None => (false, false),
        };

        // Failure to respond (CLIN-039/040): compare the latest serum at or
        // before the start with the first serum after the start.
        let baseline_serum = start.and_then(|s| {
            classified
                .iter()
                .rev()
                .find(|c| c.method == MeasurementMethod::Serum && c.age <= s)
        });
        let first_post_start_serum = start.and_then(|s| {
            classified
                .iter()
                .find(|c| c.method == MeasurementMethod::Serum && c.age > s)
        });
        let response_comparison_available =
            phototherapy_active && baseline_serum.is_some() && first_post_start_serum.is_some();
        let nonresponse_confirmed = phototherapy_active
            && match (baseline_serum, first_post_start_serum) {
                (Some(base), Some(post)) => post.value >= base.value,
                _ => false,
            };

        let rapid_rise_confirmed = trend.as_ref().is_some_and(Trend::rapid_rise_confirmed);
        let rapid_rise_at_boundary = trend
            .as_ref()
            .is_some_and(|t| t.rapid_rise_relation == ThresholdRelation::At);
        let trend_stable_or_falling = trend.as_ref().is_some_and(Trend::stable_or_falling);
        let trend_serum_stable_or_falling = trend
            .as_ref()
            .is_some_and(|t| t.reliable_for_rapid_rise && t.stable_or_falling());

        // Below-line repeat eligibility (CLIN-032): clinically well, at least
        // 38 weeks, more than 24 hours old, below the phototherapy line.
        // The spec section is "below-line monitoring BEFORE phototherapy":
        // during treatment the phototherapy monitoring rules own the repeat
        // interval, and after stopping the rebound rule does, so these
        // intervals apply only with no treatment state at all.
        let eligible_term_retest = features.clinically_well.is_present()
            && gestation >= 38
            && age > 1440
            && latest_photo_below
            && treatment_mode == TreatmentMode::None;

        let mut any_serum_above_a_line = false;
        let mut any_at_or_above_photo = false;
        for c in classified {
            let photo_rel = c.photo.map(|l| l.relation);
            let exch_rel = c.exch.map(|l| l.relation);
            if c.method == MeasurementMethod::Serum
                && (photo_rel == Some(ThresholdRelation::Above)
                    || exch_rel == Some(ThresholdRelation::Above))
            {
                any_serum_above_a_line = true;
            }
            if matches!(
                photo_rel,
                Some(ThresholdRelation::At | ThresholdRelation::Above)
            ) {
                any_at_or_above_photo = true;
            }
        }

        let danger_unknown: Vec<&'static str> = DANGER_SIGN_FIELDS
            .iter()
            .filter(|f| feature_by_name(features, f).is_unknown())
            .copied()
            .collect();

        let conjugated = assessment.conjugated_bilirubin_umol_l.map(|v| v.value());

        Ok(Self {
            first_day: assessment.assessment_age.is_first_day(),
            any_measurement_exists: !classified.is_empty(),
            any_serum_exists: classified
                .iter()
                .any(|c| c.method == MeasurementMethod::Serum),
            jaundice_suspected: features.suspected_or_obvious_jaundice.is_present(),
            suspected_absent: features.suspected_or_obvious_jaundice.is_absent(),
            visible_jaundice: features.visible_jaundice.is_present(),
            visible_absent: features.visible_jaundice.is_absent(),
            breastfeeding_intended: risks.exclusive_breastfeeding_intended.is_present(),
            abe_present: features.acute_bilirubin_encephalopathy.is_present(),
            gestation,
            age,
            latest_photo_at: latest_photo.map(|l| l.relation) == Some(ThresholdRelation::At),
            latest_photo_below,
            latest_exch_at: latest_exch.map(|l| l.relation) == Some(ThresholdRelation::At),
            latest_exch_above: latest_exch.map(|l| l.relation) == Some(ThresholdRelation::Above),
            latest_below_within_50_photo,
            latest_more_than_50_below_photo,
            latest_serum_photo_above: latest_serum_photo.map(|l| l.relation)
                == Some(ThresholdRelation::Above),
            latest_serum_exch_above: latest_serum_exch.map(|l| l.relation)
                == Some(ThresholdRelation::Above),
            latest_serum_at_least_50_below_photo: at_least_50_below(&latest_serum_photo)?,
            latest_serum_at_least_50_below_exch: at_least_50_below(&latest_serum_exch)?,
            latest_serum_within_50_of_exch: at_most_50_below(&latest_serum_exch)?,
            latest_serum_over_340: latest_serum.as_ref().is_some_and(|c| c.value > 340),
            any_serum_above_a_line,
            any_at_or_above_photo,
            latest,
            trend,
            rapid_rise_confirmed,
            rapid_rise_at_boundary,
            trend_serum_stable_or_falling,
            trend_stable_or_falling,
            treatment_mode,
            phototherapy_active,
            elapsed_since_start,
            timely_post_start_serum_exists,
            any_post_start_serum_exists,
            response_comparison_available,
            nonresponse_confirmed,
            eligible_term_retest,
            risk_factor_for_18h: risks.previous_sibling_required_phototherapy.is_present()
                || risks.exclusive_breastfeeding_intended.is_present(),
            prolonged_criteria_met: features.visible_jaundice.is_present()
                && ((gestation >= 37 && age > PROLONGED_TERM_MINUTES)
                    || (gestation < 37 && age > PROLONGED_PRETERM_MINUTES)),
            conjugated_over_25: conjugated.is_some_and(|v| v > 25),
            conjugated_exactly_25: conjugated == Some(25),
            haemolytic_disease: features.rhesus_haemolytic_disease.is_present()
                || features.abo_haemolytic_disease.is_present(),
            danger_unknown,
        })
    }

    /// Intensified-phototherapy proximity trigger (CLIN-038): requires age of
    /// at least 72 hours and serum within 50 umol/L of the exchange line.
    fn exchange_proximity_after_72h(&self) -> bool {
        self.age >= 4320 && self.latest_serum_within_50_of_exch
    }
}

fn feature_by_name(features: &crate::input::ClinicalFeatures, name: &str) -> TriState {
    match name {
        "suspected_or_obvious_jaundice" => features.suspected_or_obvious_jaundice,
        "visible_jaundice" => features.visible_jaundice,
        "clinically_well" => features.clinically_well,
        "acute_bilirubin_encephalopathy" => features.acute_bilirubin_encephalopathy,
        "pale_chalky_stools" => features.pale_chalky_stools,
        "dark_urine_stains_nappy" => features.dark_urine_stains_nappy,
        "rhesus_haemolytic_disease" => features.rhesus_haemolytic_disease,
        "abo_haemolytic_disease" => features.abo_haemolytic_disease,
        "infection_suspected" => features.infection_suspected,
        "urinary_tract_infection_suspected" => features.urinary_tract_infection_suspected,
        "routine_metabolic_screen_completed" => features.routine_metabolic_screen_completed,
        _ => unreachable!("unknown clinical feature field"),
    }
}

/// First-pass activation for rules that do not depend on other rules.
fn activates(code: RuleCode, f: &Facts) -> bool {
    use RuleCode::*;
    match code {
        AcuteBilirubinEncephalopathyEmergency => f.abe_present,
        ExchangeTransfusionEscalation => f.latest_serum_exch_above || f.abe_present,
        AtExchangeLineEmergencyReview => f.latest_exch_at,
        ExpertLiverAdvice => f.conjugated_over_25,
        IncreasedKernicterusRisk => {
            (f.latest_serum_over_340 && f.gestation >= 37)
                || f.rapid_rise_confirmed
                || f.abe_present
        }
        // The obtain-a-measurement rules (1.2.10, 1.2.14) apply while no
        // qualifying measurement has been supplied; once one exists, the
        // repeat and monitoring rules govern. Product interpretation for
        // clinical rule-pack review; it matches the specification's
        // normal-below-threshold example.
        EarlyJaundiceMeasure2h => f.jaundice_suspected && f.first_day && !f.any_serum_exists,
        EarlyJaundiceRepeat6h => {
            f.jaundice_suspected
                && f.first_day
                && !(f.latest_photo_below && f.trend_stable_or_falling)
        }
        EarlyJaundiceMedicalReview6h => f.jaundice_suspected && f.first_day,
        ConsiderIntensifiedPhototherapy => {
            f.rapid_rise_confirmed || f.exchange_proximity_after_72h() || f.nonresponse_confirmed
        }
        PhototherapyCheckOverdue => {
            f.phototherapy_active
                && f.elapsed_since_start.is_some_and(|e| e > 360)
                && !f.timely_post_start_serum_exists
        }
        PhototherapyResponseIncomplete => f.phototherapy_active && !f.response_comparison_available,
        // "Start" applies only when phototherapy is not already running; a
        // serum above the line during treatment is handled by the monitoring
        // and intensification rules.
        StartPhototherapy => {
            !f.phototherapy_active && f.latest_serum_photo_above && !f.latest_serum_exch_above
        }
        PhototherapyCheck46h => {
            f.phototherapy_active
                && f.elapsed_since_start.is_some_and(|e| e <= 360)
                && !f.any_post_start_serum_exists
        }
        PhototherapyCheck612h => f.phototherapy_active && f.trend_serum_stable_or_falling,
        StopPhototherapy => f.phototherapy_active && f.latest_serum_at_least_50_below_photo,
        ReboundCheck1218h => f.treatment_mode == TreatmentMode::PostPhototherapy,
        JaundiceMeasure6h => f.jaundice_suspected && !f.first_day && !f.any_measurement_exists,
        SerumRequiredAge => f.first_day,
        SerumRequiredGestation => f.gestation < 35,
        SerumConfirmTcb250 => f
            .latest
            .as_ref()
            .is_some_and(|c| c.method == MeasurementMethod::Transcutaneous && c.value > 250),
        SerumConfirmTreatmentLine => {
            f.latest
                .as_ref()
                .is_some_and(|c| c.method == MeasurementMethod::Transcutaneous)
                && (f.latest_photo_at
                    || f.latest_exch_at
                    || f.latest_exch_above
                    || f.latest.as_ref().is_some_and(|c| {
                        c.photo.map(|l| l.relation) == Some(ThresholdRelation::Above)
                    }))
        }
        RetestWithin18h => {
            f.eligible_term_retest && f.latest_below_within_50_photo && f.risk_factor_for_18h
        }
        RetestWithin24h => {
            f.eligible_term_retest && f.latest_below_within_50_photo && !f.risk_factor_for_18h
        }
        NoRoutineRepeat => f.eligible_term_retest && f.latest_more_than_50_below_photo,
        AssessUnderlyingDisease => f.any_serum_above_a_line,
        ProlongedJaundiceAssessment => f.prolonged_criteria_met,
        IvigSpecialistPathway => {
            f.haemolytic_disease
                && f.treatment_mode == TreatmentMode::IntensifiedPhototherapy
                && f.rapid_rise_confirmed
        }
        AtTreatmentLineReview => f.latest_photo_at && !f.latest_exch_above,
        AtConjugatedBoundaryReview => f.conjugated_exactly_25,
        ReducePhototherapyIntensity => {
            f.treatment_mode == TreatmentMode::IntensifiedPhototherapy
                && f.latest_serum_at_least_50_below_exch
        }
        IncompleteDangerAssessment => !f.danger_unknown.is_empty(),
        AtRapidRiseBoundary => f.rapid_rise_at_boundary,
        SerumRequiredSubsequent => {
            f.any_at_or_above_photo || f.treatment_mode != TreatmentMode::None
        }
        // Method guidance applies only when measuring is indicated at all;
        // otherwise it would contradict NO_ROUTINE_BILIRUBIN.
        TcbInitialAllowed => {
            (f.jaundice_suspected || f.visible_jaundice)
                && !f.first_day
                && f.gestation >= 35
                && !f.any_at_or_above_photo
                && f.treatment_mode == TreatmentMode::None
        }
        // Requires confirmed absence of both jaundice fields, never unknown
        // (PRD-008).
        NoRoutineBilirubin => f.suspected_absent && f.visible_absent,
        // The local-protocol fallback covers untreated babies outside the
        // 18/24-hour population; during or after treatment the phototherapy
        // monitoring and rebound rules define the interval instead.
        RetestIntervalLocalProtocol => {
            f.treatment_mode == TreatmentMode::None
                && f.latest_photo_below
                && !f.eligible_term_retest
        }
        AdditionalVisualInspection48h => {
            f.age < FIRST_48_HOURS_MINUTES
                && (f.gestation < 38
                    || f.risk_factor_for_18h
                    || (f.visible_jaundice && f.first_day))
        }
        VisualAssessmentLimitations => true,
        BreastfeedingSupport => f.breastfeeding_intended || f.visible_jaundice,
        ParentCarerInformation => true,
        // Second-pass rules never activate here.
        DoNotUseSunlight
        | PhototherapyCareInformation
        | IvigInformation
        | ExchangeTransfusionInformation
        | NoIcterometer
        | DoNotUsePredictionTests => false,
    }
}

/// Codes that contradict a reassuring or de-escalating message when an
/// escalation is the primary action (CLIN-048).
const REASSURING_OR_DE_ESCALATING: &[RuleCode] = &[
    RuleCode::NoRoutineRepeat,
    RuleCode::NoRoutineBilirubin,
    RuleCode::RetestWithin18h,
    RuleCode::RetestWithin24h,
    RuleCode::RetestIntervalLocalProtocol,
    RuleCode::StopPhototherapy,
    RuleCode::ReducePhototherapyIntensity,
    RuleCode::TcbInitialAllowed,
];

fn suppressed_by_primary(primary: RuleCode, candidate: RuleCode) -> bool {
    use RuleCode::*;
    match primary {
        AcuteBilirubinEncephalopathyEmergency
        | ExchangeTransfusionEscalation
        | AtExchangeLineEmergencyReview
        | IncreasedKernicterusRisk
        | ConsiderIntensifiedPhototherapy => REASSURING_OR_DE_ESCALATING.contains(&candidate),
        StartPhototherapy => matches!(
            candidate,
            NoRoutineRepeat
                | NoRoutineBilirubin
                | RetestWithin18h
                | RetestWithin24h
                | RetestIntervalLocalProtocol
                | TcbInitialAllowed
        ),
        StopPhototherapy => matches!(candidate, PhototherapyCheck46h | PhototherapyCheck612h),
        _ => false,
    }
}

pub fn evaluate(
    assessment: &Assessment,
    context: &EvaluationContext,
) -> Result<EvaluationOutcome, SafetyError> {
    // 1. Classify every measurement against exact treatment lines.
    let mut classified = Vec::with_capacity(assessment.measurements.len());
    for m in &assessment.measurements {
        let pair = treatment_thresholds(assessment.gestational_age, m.age_minutes)?;
        let (photo, exch) = match pair {
            Some(p) => (
                Some(assess_against_line(
                    m.total_bilirubin_umol_l,
                    &p.phototherapy,
                )?),
                Some(assess_against_line(m.total_bilirubin_umol_l, &p.exchange)?),
            ),
            None => (None, None),
        };
        classified.push(Classified {
            id: m.id.clone(),
            age: m.age_minutes.value(),
            value: m.total_bilirubin_umol_l.value(),
            method: m.method,
            photo,
            exch,
        });
    }

    // 2. Serial trend.
    let trend = calculate_trend(&assessment.measurements)?;

    // 3. Facts.
    let facts = Facts::build(assessment, trend, &classified)?;

    // 4. First-pass rule activation in stable rule order.
    let mut activated: Vec<RuleCode> = RuleCode::all()
        .iter()
        .copied()
        .filter(|code| activates(*code, &facts))
        .collect();

    // 5. Second pass: informational rules that depend on which rules fired.
    let has = |set: &[RuleCode], list: &[RuleCode]| list.iter().any(|c| set.contains(c));
    let phototherapy_displayed = facts.phototherapy_active
        || facts.treatment_mode == TreatmentMode::PostPhototherapy
        || has(
            &activated,
            &[
                RuleCode::StartPhototherapy,
                RuleCode::ConsiderIntensifiedPhototherapy,
            ],
        );
    let phototherapy_care = facts.phototherapy_active
        || has(
            &activated,
            &[
                RuleCode::StartPhototherapy,
                RuleCode::ConsiderIntensifiedPhototherapy,
            ],
        );
    let measurement_guidance = has(
        &activated,
        &[
            RuleCode::SerumRequiredAge,
            RuleCode::SerumRequiredGestation,
            RuleCode::SerumConfirmTcb250,
            RuleCode::SerumConfirmTreatmentLine,
            RuleCode::SerumRequiredSubsequent,
            RuleCode::TcbInitialAllowed,
            RuleCode::EarlyJaundiceMeasure2h,
            RuleCode::EarlyJaundiceRepeat6h,
            RuleCode::JaundiceMeasure6h,
        ],
    );
    let second_pass: &[(RuleCode, bool)] = &[
        (RuleCode::DoNotUseSunlight, phototherapy_displayed),
        (RuleCode::PhototherapyCareInformation, phototherapy_care),
        (
            RuleCode::IvigInformation,
            activated.contains(&RuleCode::IvigSpecialistPathway),
        ),
        (
            RuleCode::ExchangeTransfusionInformation,
            has(
                &activated,
                &[
                    RuleCode::ExchangeTransfusionEscalation,
                    RuleCode::AtExchangeLineEmergencyReview,
                ],
            ),
        ),
        (RuleCode::NoIcterometer, measurement_guidance),
        (RuleCode::DoNotUsePredictionTests, measurement_guidance),
    ];
    for (code, active) in second_pass {
        if *active {
            activated.push(*code);
        }
    }
    // Restore stable rule order after the second pass.
    activated.sort_by_key(|c| c.spec().order);

    // 6. Priority sort: fixed priority order, then stable rule order
    // (CLIN-047). `activated` is non-empty because the informational rules
    // VISUAL_ASSESSMENT_LIMITATIONS and PARENT_CARER_INFORMATION always fire,
    // so a primary action always exists (DATA-019).
    let mut ranked = activated.clone();
    ranked.sort_by_key(|c| {
        let spec = c.spec();
        (spec.priority, spec.order)
    });
    let primary_code = ranked[0];

    // 7. Suppress lower-priority recommendations that contradict the primary
    // action (CLIN-048), plus the CLIN-030 safety policy: no reassuring
    // no-routine-action output while a danger-sign assessment is incomplete.
    let danger_incomplete = activated.contains(&RuleCode::IncompleteDangerAssessment);
    let mut suppressed: Vec<String> = Vec::new();
    let kept: Vec<RuleCode> = ranked
        .iter()
        .copied()
        .filter(|code| {
            if *code == primary_code {
                return true;
            }
            let contradicted = suppressed_by_primary(primary_code, *code)
                || (danger_incomplete
                    && matches!(
                        code,
                        RuleCode::NoRoutineRepeat | RuleCode::NoRoutineBilirubin
                    ));
            if contradicted {
                suppressed.push(code.as_str().to_string());
            }
            !contradicted
        })
        .collect();

    // 8. Build recommendation instances (primary first, DATA-020).
    let latest_is_tcb = facts
        .latest
        .as_ref()
        .is_some_and(|c| c.method == MeasurementMethod::Transcutaneous);
    let recommendations: Vec<Recommendation> = kept
        .iter()
        .map(|code| build_recommendation(*code, latest_is_tcb))
        .collect();
    let primary_action = recommendations[0].clone();

    // 9. Threshold assessments and decision trace.
    let mut threshold_rows = Vec::with_capacity(classified.len());
    let mut trace_rows = Vec::with_capacity(classified.len());
    for c in &classified {
        threshold_rows.push(threshold_assessment(c)?);
        trace_rows.push(ExactThresholdTrace {
            measurement_id: c.id.clone(),
            phototherapy: c.photo.as_ref().map(|l| ExactFraction::from(&l.threshold)),
            exchange: c.exch.as_ref().map(|l| ExactFraction::from(&l.threshold)),
            phototherapy_distance: c.photo.as_ref().map(|l| ExactFraction::from(&l.distance)),
            exchange_distance: c.exch.as_ref().map(|l| ExactFraction::from(&l.distance)),
        });
    }

    let trend_assessment = match &facts.trend {
        Some(t) => Some(TrendAssessment {
            older_measurement_id: t.older_measurement_id.clone(),
            newer_measurement_id: t.newer_measurement_id.clone(),
            interval_minutes: t.interval_minutes,
            rate_umol_l_per_hour: Display1Dp(t.rate.display_tenths()?),
            direction: t.direction,
            reliable_for_rapid_rise: t.reliable_for_rapid_rise,
            rapid_rise_relation: t.rapid_rise_relation,
        }),
        None => None,
    };

    let outcome = EvaluationOutcome {
        normalised_input: NormalisedInput {
            gestational_age_completed_weeks: assessment.gestational_age.value(),
            assessment_age_minutes: assessment.assessment_age.value(),
            measurement_count: assessment.measurements.len(),
            latest_measurement_id: facts.latest.as_ref().map(|c| c.id.clone()),
        },
        thresholds: threshold_rows,
        trend: trend_assessment,
        primary_action,
        recommendations,
        warnings: build_warnings(assessment, &facts, context),
        missing_information: build_missing_information(assessment, &facts),
        suppressed_rules: suppressed,
        decision_trace: DecisionTrace {
            exact_thresholds: trace_rows,
            exact_rate: facts.trend.as_ref().map(|t| ExactFraction::from(&t.rate)),
            activated_rules: activated.iter().map(|c| c.as_str().to_string()).collect(),
        },
    };
    Ok(outcome)
}

fn build_recommendation(code: RuleCode, latest_is_tcb: bool) -> Recommendation {
    let spec = code.spec();
    // At-line reviews require serum confirmation when the triggering value is
    // transcutaneous (spec 02 equality policy).
    let requires_serum_confirmation = match code {
        RuleCode::AtTreatmentLineReview | RuleCode::AtExchangeLineEmergencyReview => latest_is_tcb,
        _ => spec.requires_serum_confirmation,
    };
    Recommendation {
        code: code.as_str().to_string(),
        priority: spec.priority,
        category: spec.category,
        action: spec.action.to_string(),
        timeframe: spec.timeframe,
        rationale: spec.rationale.to_string(),
        source_refs: code.source_references(),
        requires_serum_confirmation,
        requires_clinician_confirmation: true,
    }
}

fn threshold_assessment(c: &Classified) -> Result<ThresholdAssessment, SafetyError> {
    let display = |line: &Option<LineAssessment>| -> Result<
        (Option<Display1Dp>, ThresholdRelation, Option<Display1Dp>),
        SafetyError,
    > {
        match line {
            Some(l) => Ok((
                Some(Display1Dp(l.threshold.display_tenths()?)),
                l.relation,
                Some(Display1Dp(l.distance.display_tenths()?)),
            )),
            None => Ok((None, ThresholdRelation::NotAvailable, None)),
        }
    };
    let (photo_line, photo_rel, photo_dist) = display(&c.photo)?;
    let (exch_line, exch_rel, exch_dist) = display(&c.exch)?;
    Ok(ThresholdAssessment {
        measurement_id: c.id.clone(),
        age_minutes: c.age,
        phototherapy_threshold_umol_l: photo_line,
        phototherapy_relation: photo_rel,
        phototherapy_distance_umol_l: photo_dist,
        exchange_threshold_umol_l: exch_line,
        exchange_relation: exch_rel,
        exchange_distance_umol_l: exch_dist,
        treatment_decision_eligible: c.method == MeasurementMethod::Serum,
    })
}

fn build_warnings(
    assessment: &Assessment,
    facts: &Facts,
    context: &EvaluationContext,
) -> Vec<Warning> {
    let mut warnings = Vec::new();
    if context.mode == Mode::Demonstration {
        warnings.push(Warning {
            code: "DEMONSTRATION_ONLY".into(),
            category: WarningCategory::System,
            message: "This result was produced in demonstration mode and must not be used for patient care.".into(),
        });
    }
    // Universal assay warning (PRD-023, spec 02 universal notices).
    warnings.push(Warning {
        code: "LOCAL_PATHOLOGY_ASSAY_WARNING".into(),
        category: WarningCategory::Assay,
        message: "Bilirubin results vary between assays. Interpret this result with advice from the local pathology laboratory.".into(),
    });
    // Darker-skin recognition warning wherever visual guidance appears
    // (CLIN-031, PRD-022).
    warnings.push(Warning {
        code: "DARKER_SKIN_RECOGNITION".into(),
        category: WarningCategory::Clinical,
        message: "Changes in skin colour caused by hyperbilirubinaemia can be harder to see in darker skin tones. Do not rely on visual inspection alone to estimate the bilirubin level.".into(),
    });
    // Out-of-scope ages never read as "normal" (PRD-003, PRD-009, PRD-024).
    if assessment.assessment_age.value() > TREATMENT_LINE_MAX_AGE_MINUTES {
        warnings.push(Warning {
            code: "THRESHOLDS_NOT_CALCULATED_AFTER_336_HOURS".into(),
            category: WarningCategory::Scope,
            message: "Treatment thresholds are defined only from birth through 336 hours (14 days). No treatment line was calculated for this assessment; this is not a statement that the bilirubin level is normal or safe.".into(),
        });
    }
    // A transcutaneous value at or above a line cannot drive treatment
    // (CLIN-017, CLIN-029, PRD-026).
    let latest_tcb_at_or_above = facts
        .latest
        .as_ref()
        .is_some_and(|c| c.method == MeasurementMethod::Transcutaneous)
        && (facts.latest_photo_at
            || facts.latest_exch_at
            || facts.latest_exch_above
            || facts
                .latest
                .as_ref()
                .is_some_and(|c| c.photo.map(|l| l.relation) == Some(ThresholdRelation::Above)));
    if latest_tcb_at_or_above {
        warnings.push(Warning {
            code: "TCB_NOT_TREATMENT_DECISION_ELIGIBLE".into(),
            category: WarningCategory::Clinical,
            message: "The most recent value is a transcutaneous measurement. A definitive treatment decision must not be based on it; confirm with a serum bilirubin measurement.".into(),
        });
    }
    warnings
}

fn build_missing_information(assessment: &Assessment, facts: &Facts) -> Vec<MissingInformation> {
    let features = &assessment.clinical_features;
    let risks = &assessment.risk_factors;
    let mut missing = Vec::new();

    let feature_fields: &[(&str, TriState, &str)] = &[
        ("suspected_or_obvious_jaundice", features.suspected_or_obvious_jaundice, "Recognition rules could not distinguish suspected jaundice from confirmed absence."),
        ("visible_jaundice", features.visible_jaundice, "Visible-jaundice rules, including prolonged-jaundice assessment, could not be evaluated."),
        ("clinically_well", features.clinically_well, "The 18- and 24-hour repeat-interval rules and the no-routine-repeat rule were not evaluated."),
        ("acute_bilirubin_encephalopathy", features.acute_bilirubin_encephalopathy, "Emergency encephalopathy rules could not be evaluated. A reassuring primary action is blocked."),
        ("pale_chalky_stools", features.pale_chalky_stools, "Liver danger signs could not be assessed. A reassuring primary action is blocked."),
        ("dark_urine_stains_nappy", features.dark_urine_stains_nappy, "Liver danger signs could not be assessed. A reassuring primary action is blocked."),
        ("rhesus_haemolytic_disease", features.rhesus_haemolytic_disease, "Haemolytic-disease pathways, including intravenous immunoglobulin, could not be evaluated."),
        ("abo_haemolytic_disease", features.abo_haemolytic_disease, "Haemolytic-disease pathways, including intravenous immunoglobulin, could not be evaluated."),
        ("infection_suspected", features.infection_suspected, "Infection-dependent assessments could not be evaluated. A reassuring primary action is blocked."),
        ("urinary_tract_infection_suspected", features.urinary_tract_infection_suspected, "The urine-culture recommendation could not be evaluated."),
        ("routine_metabolic_screen_completed", features.routine_metabolic_screen_completed, "Confirmation of routine metabolic screening could not be evaluated."),
    ];
    for (name, value, impact) in feature_fields {
        if value.is_unknown() {
            missing.push(MissingInformation {
                pointer: format!("/clinical_features/{name}"),
                code: "FIELD_UNKNOWN".into(),
                impact: (*impact).to_string(),
            });
        }
    }

    let risk_fields: &[(&str, TriState)] = &[
        (
            "previous_sibling_required_phototherapy",
            risks.previous_sibling_required_phototherapy,
        ),
        (
            "exclusive_breastfeeding_intended",
            risks.exclusive_breastfeeding_intended,
        ),
    ];
    for (name, value) in risk_fields {
        if value.is_unknown() {
            missing.push(MissingInformation {
                pointer: format!("/risk_factors/{name}"),
                code: "FIELD_UNKNOWN".into(),
                impact: "The 18-hour repeat-interval branch could not be selected; the 24-hour interval applies where eligible.".into(),
            });
        }
    }

    if facts.phototherapy_active && !facts.response_comparison_available {
        missing.push(MissingInformation {
            pointer: "/measurements".into(),
            code: "PHOTOTHERAPY_RESPONSE_COMPARISON_UNAVAILABLE".into(),
            impact: "The submitted history cannot compare a baseline serum result with a post-start serum result, so failure to respond to phototherapy could not be classified.".into(),
        });
    }

    if facts.prolonged_criteria_met && assessment.conjugated_bilirubin_umol_l.is_none() {
        missing.push(MissingInformation {
            pointer: "/conjugated_bilirubin_umol_l".into(),
            code: "NOT_SUPPLIED".into(),
            impact: "The conjugated-bilirubin rules of the prolonged-jaundice assessment could not be evaluated.".into(),
        });
    }

    missing
}
