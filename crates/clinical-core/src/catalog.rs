//! Recommendation catalogue: stable codes, fixed priorities, stable rule
//! order, categories, action text and NICE source references, transcribed
//! from the rule pack `nice-cg98-2023-10-31.1` and spec 02.
//!
//! Codes are stable within API v1 and are never reused with a different
//! meaning (DATA-015). Action text uses UK English and must not weaken the
//! mapped NICE recommendation (DATA-016).

use serde::Serialize;

use crate::types::Priority;

pub const NICE_PRODUCT_ID: &str = "NICE-CG98";
pub const NICE_RECOMMENDATIONS_URL: &str =
    "https://www.nice.org.uk/guidance/cg98/chapter/recommendations";
pub const NICE_THRESHOLD_RESOURCE_URL: &str =
    "https://www.nice.org.uk/guidance/cg98/resources/treatment-threshold-graphs-excel-544300525";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RuleCode {
    AcuteBilirubinEncephalopathyEmergency,
    ExchangeTransfusionEscalation,
    AtExchangeLineEmergencyReview,
    ExpertLiverAdvice,
    IncreasedKernicterusRisk,
    EarlyJaundiceMeasure2h,
    EarlyJaundiceRepeat6h,
    EarlyJaundiceMedicalReview6h,
    ConsiderIntensifiedPhototherapy,
    PhototherapyCheckOverdue,
    PhototherapyResponseIncomplete,
    StartPhototherapy,
    PhototherapyCheck46h,
    PhototherapyCheck612h,
    StopPhototherapy,
    ReboundCheck1218h,
    JaundiceMeasure6h,
    SerumRequiredAge,
    SerumRequiredGestation,
    SerumConfirmTcb250,
    SerumConfirmTreatmentLine,
    RetestWithin18h,
    RetestWithin24h,
    NoRoutineRepeat,
    AssessUnderlyingDisease,
    ProlongedJaundiceAssessment,
    IvigSpecialistPathway,
    AtTreatmentLineReview,
    AtConjugatedBoundaryReview,
    ReducePhototherapyIntensity,
    IncompleteDangerAssessment,
    AtRapidRiseBoundary,
    SerumRequiredSubsequent,
    TcbInitialAllowed,
    NoRoutineBilirubin,
    RetestIntervalLocalProtocol,
    DoNotUseSunlight,
    AdditionalVisualInspection48h,
    VisualAssessmentLimitations,
    BreastfeedingSupport,
    ParentCarerInformation,
    PhototherapyCareInformation,
    IvigInformation,
    ExchangeTransfusionInformation,
    NoIcterometer,
    DoNotUsePredictionTests,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Emergency,
    Measurement,
    Monitoring,
    Phototherapy,
    ExchangeTransfusion,
    UnderlyingDisease,
    ProlongedJaundice,
    FeedingSupport,
    Information,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeUnit {
    Hours,
}

/// A timeframe with a unit and either an exact value or an inclusive range.
/// Never converted into an appointment timestamp by the server (spec 03).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Timeframe {
    pub unit: TimeUnit,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exact_value: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<u32>,
}

impl Timeframe {
    const fn within(maximum: u32) -> Self {
        Self {
            unit: TimeUnit::Hours,
            exact_value: None,
            minimum: None,
            maximum: Some(maximum),
        }
    }
    const fn every(exact: u32) -> Self {
        Self {
            unit: TimeUnit::Hours,
            exact_value: Some(exact),
            minimum: None,
            maximum: None,
        }
    }
    const fn range(minimum: u32, maximum: u32) -> Self {
        Self {
            unit: TimeUnit::Hours,
            exact_value: None,
            minimum: Some(minimum),
            maximum: Some(maximum),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceReference {
    pub product_id: String,
    pub reference: String,
    pub url: String,
}

/// Static definition of one rule in the catalogue.
pub struct RuleSpec {
    /// Stable rule order from the clinical YAML; ties in priority are broken
    /// by this order (CLIN-047).
    pub order: u32,
    pub priority: Priority,
    pub category: Category,
    pub action: &'static str,
    pub rationale: &'static str,
    pub timeframe: Option<Timeframe>,
    /// CG98 recommendation numbers; `"threshold-table"` maps to the official
    /// threshold resource URL.
    pub refs: &'static [&'static str],
    /// Default serum-confirmation flag; rules that depend on the measurement
    /// method override this at activation time.
    pub requires_serum_confirmation: bool,
}

impl RuleCode {
    /// Every catalogue code in stable rule order.
    pub fn all() -> &'static [RuleCode] {
        use RuleCode::*;
        &[
            AcuteBilirubinEncephalopathyEmergency,
            ExchangeTransfusionEscalation,
            AtExchangeLineEmergencyReview,
            ExpertLiverAdvice,
            IncreasedKernicterusRisk,
            EarlyJaundiceMeasure2h,
            EarlyJaundiceRepeat6h,
            EarlyJaundiceMedicalReview6h,
            ConsiderIntensifiedPhototherapy,
            PhototherapyCheckOverdue,
            PhototherapyResponseIncomplete,
            StartPhototherapy,
            PhototherapyCheck46h,
            PhototherapyCheck612h,
            StopPhototherapy,
            ReboundCheck1218h,
            JaundiceMeasure6h,
            SerumRequiredAge,
            SerumRequiredGestation,
            SerumConfirmTcb250,
            SerumConfirmTreatmentLine,
            RetestWithin18h,
            RetestWithin24h,
            NoRoutineRepeat,
            AssessUnderlyingDisease,
            ProlongedJaundiceAssessment,
            IvigSpecialistPathway,
            AtTreatmentLineReview,
            AtConjugatedBoundaryReview,
            ReducePhototherapyIntensity,
            IncompleteDangerAssessment,
            AtRapidRiseBoundary,
            SerumRequiredSubsequent,
            TcbInitialAllowed,
            NoRoutineBilirubin,
            RetestIntervalLocalProtocol,
            DoNotUseSunlight,
            AdditionalVisualInspection48h,
            VisualAssessmentLimitations,
            BreastfeedingSupport,
            ParentCarerInformation,
            PhototherapyCareInformation,
            IvigInformation,
            ExchangeTransfusionInformation,
            NoIcterometer,
            DoNotUsePredictionTests,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        use RuleCode::*;
        match self {
            AcuteBilirubinEncephalopathyEmergency => "ACUTE_BILIRUBIN_ENCEPHALOPATHY_EMERGENCY",
            ExchangeTransfusionEscalation => "EXCHANGE_TRANSFUSION_ESCALATION",
            AtExchangeLineEmergencyReview => "AT_EXCHANGE_LINE_EMERGENCY_REVIEW",
            ExpertLiverAdvice => "EXPERT_LIVER_ADVICE",
            IncreasedKernicterusRisk => "INCREASED_KERNICTERUS_RISK",
            EarlyJaundiceMeasure2h => "EARLY_JAUNDICE_MEASURE_2H",
            EarlyJaundiceRepeat6h => "EARLY_JAUNDICE_REPEAT_6H",
            EarlyJaundiceMedicalReview6h => "EARLY_JAUNDICE_MEDICAL_REVIEW_6H",
            ConsiderIntensifiedPhototherapy => "CONSIDER_INTENSIFIED_PHOTOTHERAPY",
            PhototherapyCheckOverdue => "PHOTOTHERAPY_CHECK_OVERDUE",
            PhototherapyResponseIncomplete => "PHOTOTHERAPY_RESPONSE_INCOMPLETE",
            StartPhototherapy => "START_PHOTOTHERAPY",
            PhototherapyCheck46h => "PHOTOTHERAPY_CHECK_4_6H",
            PhototherapyCheck612h => "PHOTOTHERAPY_CHECK_6_12H",
            StopPhototherapy => "STOP_PHOTOTHERAPY",
            ReboundCheck1218h => "REBOUND_CHECK_12_18H",
            JaundiceMeasure6h => "JAUNDICE_MEASURE_6H",
            SerumRequiredAge => "SERUM_REQUIRED_AGE",
            SerumRequiredGestation => "SERUM_REQUIRED_GESTATION",
            SerumConfirmTcb250 => "SERUM_CONFIRM_TCB_250",
            SerumConfirmTreatmentLine => "SERUM_CONFIRM_TREATMENT_LINE",
            RetestWithin18h => "RETEST_WITHIN_18H",
            RetestWithin24h => "RETEST_WITHIN_24H",
            NoRoutineRepeat => "NO_ROUTINE_REPEAT",
            AssessUnderlyingDisease => "ASSESS_UNDERLYING_DISEASE",
            ProlongedJaundiceAssessment => "PROLONGED_JAUNDICE_ASSESSMENT",
            IvigSpecialistPathway => "IVIG_SPECIALIST_PATHWAY",
            AtTreatmentLineReview => "AT_TREATMENT_LINE_REVIEW",
            AtConjugatedBoundaryReview => "AT_CONJUGATED_BOUNDARY_REVIEW",
            ReducePhototherapyIntensity => "REDUCE_PHOTOTHERAPY_INTENSITY",
            IncompleteDangerAssessment => "INCOMPLETE_DANGER_ASSESSMENT",
            AtRapidRiseBoundary => "AT_RAPID_RISE_BOUNDARY",
            SerumRequiredSubsequent => "SERUM_REQUIRED_SUBSEQUENT",
            TcbInitialAllowed => "TCB_INITIAL_ALLOWED",
            NoRoutineBilirubin => "NO_ROUTINE_BILIRUBIN",
            RetestIntervalLocalProtocol => "RETEST_INTERVAL_LOCAL_PROTOCOL",
            DoNotUseSunlight => "DO_NOT_USE_SUNLIGHT",
            AdditionalVisualInspection48h => "ADDITIONAL_VISUAL_INSPECTION_48H",
            VisualAssessmentLimitations => "VISUAL_ASSESSMENT_LIMITATIONS",
            BreastfeedingSupport => "BREASTFEEDING_SUPPORT",
            ParentCarerInformation => "PARENT_CARER_INFORMATION",
            PhototherapyCareInformation => "PHOTOTHERAPY_CARE_INFORMATION",
            IvigInformation => "IVIG_INFORMATION",
            ExchangeTransfusionInformation => "EXCHANGE_TRANSFUSION_INFORMATION",
            NoIcterometer => "NO_ICETEROMETER",
            DoNotUsePredictionTests => "DO_NOT_USE_PREDICTION_TESTS",
        }
    }

    pub fn spec(&self) -> RuleSpec {
        use Category as C;
        use Priority as P;
        use RuleCode::*;
        match self {
            AcuteBilirubinEncephalopathyEmergency => RuleSpec {
                order: 10,
                priority: P::Emergency,
                category: C::Emergency,
                action: "Treat the clinical features of acute bilirubin encephalopathy as a medical emergency: escalate immediately to senior neonatal care and prepare for exchange transfusion.",
                rationale: "Acute bilirubin encephalopathy is an emergency regardless of the calculated bilirubin line.",
                timeframe: None,
                refs: &["1.5.1", "1.9.2"],
                requires_serum_confirmation: false,
            },
            ExchangeTransfusionEscalation => RuleSpec {
                order: 20,
                priority: P::Emergency,
                category: C::ExchangeTransfusion,
                action: "Double-volume exchange transfusion is indicated; escalate urgently to neonatal intensive care. Continue continuous intensified phototherapy while preparing. After exchange, continue intensified phototherapy and repeat serum bilirubin within 2 hours.",
                rationale: "Serum bilirubin is above the exchange transfusion line, or acute bilirubin encephalopathy is present.",
                timeframe: None,
                refs: &["1.9.2", "1.9.3", "1.9.4"],
                requires_serum_confirmation: false,
            },
            AtExchangeLineEmergencyReview => RuleSpec {
                order: 30,
                priority: P::Immediate,
                category: C::ExchangeTransfusion,
                action: "The bilirubin value is exactly at the exchange transfusion line: obtain immediate senior clinical review.",
                rationale: "Exact equality with the exchange line is not classified as above; it requires immediate review rather than automatic escalation.",
                timeframe: None,
                refs: &["threshold-table", "1.2.16"],
                requires_serum_confirmation: false,
            },
            ExpertLiverAdvice => RuleSpec {
                order: 40,
                priority: P::Immediate,
                category: C::ProlongedJaundice,
                action: "Conjugated bilirubin is greater than 25 micromol/L: seek expert advice from a specialist liver service.",
                rationale: "A raised conjugated bilirubin requires specialist hepatology assessment.",
                timeframe: None,
                refs: &["1.7.2"],
                requires_serum_confirmation: false,
            },
            IncreasedKernicterusRisk => RuleSpec {
                order: 50,
                priority: P::Immediate,
                category: C::Emergency,
                action: "Recognise an increased risk of kernicterus and obtain urgent senior clinical review.",
                rationale: "Serum bilirubin greater than 340 micromol/L at 37 weeks or more, a confirmed rise faster than 8.5 micromol/L per hour, or clinical features of acute bilirubin encephalopathy increase kernicterus risk.",
                timeframe: None,
                refs: &["1.5.1"],
                requires_serum_confirmation: false,
            },
            EarlyJaundiceMeasure2h => RuleSpec {
                order: 60,
                priority: P::Urgent,
                category: C::Measurement,
                action: "Measure and record serum bilirubin urgently, within 2 hours.",
                rationale: "Jaundice is suspected or obvious in the first 24 hours after birth.",
                timeframe: Some(Timeframe::within(2)),
                refs: &["1.2.10"],
                requires_serum_confirmation: false,
            },
            EarlyJaundiceRepeat6h => RuleSpec {
                order: 70,
                priority: P::Urgent,
                category: C::Measurement,
                action: "Continue measuring serum bilirubin every 6 hours until the level is below the treatment threshold and stable or falling.",
                rationale: "Early jaundice requires serial serum measurement until it is confirmed below threshold and stable or falling.",
                timeframe: Some(Timeframe::every(6)),
                refs: &["1.2.11"],
                requires_serum_confirmation: false,
            },
            EarlyJaundiceMedicalReview6h => RuleSpec {
                order: 80,
                priority: P::Urgent,
                category: C::Monitoring,
                action: "Arrange medical review as soon as possible, and within 6 hours.",
                rationale: "Jaundice in the first 24 hours requires medical review to exclude underlying disease.",
                timeframe: Some(Timeframe::within(6)),
                refs: &["1.2.12"],
                requires_serum_confirmation: false,
            },
            ConsiderIntensifiedPhototherapy => RuleSpec {
                order: 90,
                priority: P::Urgent,
                category: C::Phototherapy,
                action: "Consider intensified phototherapy.",
                rationale: "Serum bilirubin is rising faster than 8.5 micromol/L per hour, is within 50 micromol/L of the exchange line after 72 hours of age, or has not fallen in response to phototherapy.",
                timeframe: None,
                refs: &["1.4.9"],
                requires_serum_confirmation: false,
            },
            PhototherapyCheckOverdue => RuleSpec {
                order: 95,
                priority: P::Urgent,
                category: C::Monitoring,
                action: "Phototherapy monitoring is overdue: obtain an urgent serum bilirubin and review against the local escalation pathway.",
                rationale: "More than 6 hours have passed since phototherapy started without a qualifying post-start serum result.",
                timeframe: None,
                refs: &["1.4.4", "1.4.9"],
                requires_serum_confirmation: false,
            },
            PhototherapyResponseIncomplete => RuleSpec {
                order: 96,
                priority: P::Urgent,
                category: C::Monitoring,
                action: "The response to phototherapy cannot be assessed from the submitted results: obtain or review serum bilirubin results and follow the local escalation pathway.",
                rationale: "The submitted history cannot compare a baseline serum result with a post-start serum result.",
                timeframe: None,
                refs: &["1.4.4", "1.4.9"],
                requires_serum_confirmation: false,
            },
            StartPhototherapy => RuleSpec {
                order: 100,
                priority: P::Treatment,
                category: C::Phototherapy,
                action: "Start phototherapy.",
                rationale: "Serum bilirubin is above the phototherapy threshold and below the exchange transfusion threshold.",
                timeframe: None,
                refs: &["1.4.3", "1.4.8"],
                requires_serum_confirmation: false,
            },
            PhototherapyCheck46h => RuleSpec {
                order: 110,
                priority: P::Timed,
                category: C::Monitoring,
                action: "Repeat serum bilirubin 4 to 6 hours after phototherapy started.",
                rationale: "The response to phototherapy must be checked 4 to 6 hours after initiation.",
                timeframe: Some(Timeframe::range(4, 6)),
                refs: &["1.4.4"],
                requires_serum_confirmation: false,
            },
            PhototherapyCheck612h => RuleSpec {
                order: 120,
                priority: P::Timed,
                category: C::Monitoring,
                action: "Repeat serum bilirubin every 6 to 12 hours while the level is stable or falling during phototherapy.",
                rationale: "Serum bilirubin is stable or falling during phototherapy.",
                timeframe: Some(Timeframe::range(6, 12)),
                refs: &["1.4.4"],
                requires_serum_confirmation: false,
            },
            StopPhototherapy => RuleSpec {
                order: 130,
                priority: P::Treatment,
                category: C::Phototherapy,
                action: "Stop phototherapy: serum bilirubin is at least 50 micromol/L below the phototherapy threshold.",
                rationale: "Phototherapy may be stopped once serum bilirubin has fallen at least 50 micromol/L below the treatment threshold.",
                timeframe: None,
                refs: &["1.4.5"],
                requires_serum_confirmation: false,
            },
            ReboundCheck1218h => RuleSpec {
                order: 140,
                priority: P::Timed,
                category: C::Monitoring,
                action: "Repeat serum bilirubin 12 to 18 hours after stopping phototherapy to check for rebound.",
                rationale: "Bilirubin can rebound after phototherapy stops.",
                timeframe: Some(Timeframe::range(12, 18)),
                refs: &["1.4.6"],
                requires_serum_confirmation: false,
            },
            JaundiceMeasure6h => RuleSpec {
                order: 150,
                priority: P::Timed,
                category: C::Measurement,
                action: "Measure and record the bilirubin level within 6 hours.",
                rationale: "Jaundice is suspected or obvious in a baby more than 24 hours old.",
                timeframe: Some(Timeframe::within(6)),
                refs: &["1.2.14"],
                requires_serum_confirmation: false,
            },
            SerumRequiredAge => RuleSpec {
                order: 160,
                priority: P::Timed,
                category: C::Measurement,
                action: "Use serum bilirubin for all bilirubin measurements in the first 24 hours after birth.",
                rationale: "Transcutaneous measurement is not validated in the first 24 hours.",
                timeframe: None,
                refs: &["1.2.15"],
                requires_serum_confirmation: false,
            },
            SerumRequiredGestation => RuleSpec {
                order: 170,
                priority: P::Timed,
                category: C::Measurement,
                action: "Use serum bilirubin for all bilirubin measurements in a baby born before 35 weeks' gestation.",
                rationale: "Transcutaneous measurement is not validated below 35 weeks' gestation.",
                timeframe: None,
                refs: &["1.2.15"],
                requires_serum_confirmation: false,
            },
            SerumConfirmTcb250 => RuleSpec {
                order: 180,
                priority: P::Timed,
                category: C::Measurement,
                action: "The transcutaneous bilirubin is greater than 250 micromol/L: confirm the level with a serum bilirubin measurement.",
                rationale: "A transcutaneous result above 250 micromol/L must be checked by measuring serum bilirubin.",
                timeframe: None,
                refs: &["1.2.16"],
                requires_serum_confirmation: true,
            },
            SerumConfirmTreatmentLine => RuleSpec {
                order: 190,
                priority: P::Timed,
                category: C::Measurement,
                action: "The transcutaneous bilirubin is at or above the relevant treatment line: confirm the level with a serum bilirubin measurement.",
                rationale: "A treatment decision must not be based on a transcutaneous measurement alone.",
                timeframe: None,
                refs: &["1.2.16"],
                requires_serum_confirmation: true,
            },
            RetestWithin18h => RuleSpec {
                order: 200,
                priority: P::Timed,
                category: C::Monitoring,
                action: "Repeat the bilirubin measurement within 18 hours.",
                rationale: "The result is within 50 micromol/L of the phototherapy threshold and a risk factor is present: a previous sibling needed phototherapy or exclusive breastfeeding is intended.",
                timeframe: Some(Timeframe::within(18)),
                refs: &["1.4.1"],
                requires_serum_confirmation: false,
            },
            RetestWithin24h => RuleSpec {
                order: 210,
                priority: P::Timed,
                category: C::Monitoring,
                action: "Repeat the bilirubin measurement within 24 hours.",
                rationale: "The result is within 50 micromol/L of the phototherapy threshold without an additional risk factor.",
                timeframe: Some(Timeframe::within(24)),
                refs: &["1.4.1"],
                requires_serum_confirmation: false,
            },
            NoRoutineRepeat => RuleSpec {
                order: 220,
                priority: P::Routine,
                category: C::Monitoring,
                action: "Do not routinely repeat the bilirubin measurement solely on the basis of this result.",
                rationale: "The baby is clinically well, at least 38 weeks' gestation, over 24 hours old and more than 50 micromol/L below the phototherapy threshold.",
                timeframe: None,
                refs: &["1.4.2"],
                requires_serum_confirmation: false,
            },
            AssessUnderlyingDisease => RuleSpec {
                order: 230,
                priority: P::Routine,
                category: C::UnderlyingDisease,
                action: "Assess for underlying disease: record the serum bilirubin baseline, blood packed cell volume, mother's and baby's blood groups, and a direct antiglobulin test interpreted with the strength of reaction and any maternal prophylactic anti-D. Consider a full blood count and blood film, glucose-6-phosphate dehydrogenase testing taking account of ethnic origin, and blood, urine and cerebrospinal fluid cultures if infection is suspected.",
                rationale: "Significant hyperbilirubinaemia requires assessment for an underlying cause.",
                timeframe: None,
                refs: &["1.6.1", "1.6.2"],
                requires_serum_confirmation: false,
            },
            ProlongedJaundiceAssessment => RuleSpec {
                order: 240,
                priority: P::Urgent,
                category: C::ProlongedJaundice,
                action: "Carry out a prolonged jaundice assessment: look for pale chalky stools and dark urine that stains the nappy, measure conjugated bilirubin, obtain a full blood count, check mother's and baby's blood groups and a direct antiglobulin test, and confirm that routine metabolic screening including congenital hypothyroidism screening has been completed. Perform a urine culture only if urinary tract infection is clinically suspected.",
                rationale: "Visible jaundice has persisted beyond 14 days at 37 or more weeks' gestation, or beyond 21 days below 37 weeks.",
                timeframe: None,
                refs: &["1.7.1"],
                requires_serum_confirmation: false,
            },
            IvigSpecialistPathway => RuleSpec {
                order: 250,
                priority: P::Urgent,
                category: C::UnderlyingDisease,
                action: "Specialist prescribing information, not an order: intravenous immunoglobulin 500 mg/kg over 4 hours may be used as an adjunct to continuous intensified phototherapy in rhesus or ABO haemolytic disease when serum bilirubin continues to rise by more than 8.5 micromol/L per hour. Confirmation by a neonatal specialist is required.",
                rationale: "Rhesus or ABO haemolytic disease with continuous intensified phototherapy and a confirmed rapid serum rise meets the NICE adjunct criteria.",
                timeframe: None,
                refs: &["1.8.1"],
                requires_serum_confirmation: false,
            },
            AtTreatmentLineReview => RuleSpec {
                order: 260,
                priority: P::Urgent,
                category: C::Monitoring,
                action: "The bilirubin value is exactly at the phototherapy treatment line: obtain urgent clinician review.",
                rationale: "Exact equality with the treatment line is preserved rather than rounded into below or above, and requires clinical review.",
                timeframe: None,
                refs: &["threshold-table", "1.2.16", "1.3.4"],
                requires_serum_confirmation: false,
            },
            AtConjugatedBoundaryReview => RuleSpec {
                order: 270,
                priority: P::Urgent,
                category: C::ProlongedJaundice,
                action: "Conjugated bilirubin is exactly 25 micromol/L: obtain clinical review before deciding whether specialist liver advice is required.",
                rationale: "The specialist-liver rule applies strictly above 25 micromol/L; exact equality requires review.",
                timeframe: None,
                refs: &["1.7.2"],
                requires_serum_confirmation: false,
            },
            ReducePhototherapyIntensity => RuleSpec {
                order: 280,
                priority: P::Treatment,
                category: C::Phototherapy,
                action: "Reduce the intensity of phototherapy: serum bilirubin is at least 50 micromol/L below the exchange transfusion threshold during intensified phototherapy.",
                rationale: "Intensified phototherapy may be stepped down once serum bilirubin is at least 50 micromol/L below the exchange line.",
                timeframe: None,
                refs: &["1.4.10"],
                requires_serum_confirmation: false,
            },
            IncompleteDangerAssessment => RuleSpec {
                order: 290,
                priority: P::Urgent,
                category: C::Monitoring,
                action: "Complete the outstanding danger-sign assessment before relying on any reassuring finding in this result.",
                rationale: "One or more danger-sign fields were submitted as unknown; unknown is not evidence of absence.",
                timeframe: None,
                refs: &["1.2.3", "1.5.1"],
                requires_serum_confirmation: false,
            },
            AtRapidRiseBoundary => RuleSpec {
                order: 310,
                priority: P::Timed,
                category: C::Monitoring,
                action: "The serum bilirubin rise is exactly 8.5 micromol/L per hour: review closely. The rapid-rise rule applies only to a rise strictly greater than 8.5 micromol/L per hour.",
                rationale: "Exact equality with the rapid-rise rate is preserved and flagged for review rather than classified as rapid.",
                timeframe: None,
                refs: &["1.4.9", "1.5.1"],
                requires_serum_confirmation: false,
            },
            SerumRequiredSubsequent => RuleSpec {
                order: 320,
                priority: P::Timed,
                category: C::Measurement,
                action: "Use serum bilirubin for all subsequent measurements.",
                rationale: "A result has reached a treatment line or treatment has begun; transcutaneous measurement is no longer appropriate.",
                timeframe: None,
                refs: &["1.2.16"],
                requires_serum_confirmation: false,
            },
            TcbInitialAllowed => RuleSpec {
                order: 330,
                priority: P::Routine,
                category: C::Measurement,
                action: "A transcutaneous bilirubinometer may be used for this baby's measurement; use serum bilirubin if a transcutaneous bilirubinometer is not available.",
                rationale: "The baby is more than 24 hours old, at least 35 weeks' gestation, and no rule requires serum measurement.",
                timeframe: None,
                refs: &["1.2.16"],
                requires_serum_confirmation: false,
            },
            NoRoutineBilirubin => RuleSpec {
                order: 340,
                priority: P::Routine,
                category: C::Measurement,
                action: "Do not routinely measure the bilirubin level in a baby without suspected or visible jaundice.",
                rationale: "Jaundice is neither suspected nor visible.",
                timeframe: None,
                refs: &["1.2.7"],
                requires_serum_confirmation: false,
            },
            RetestIntervalLocalProtocol => RuleSpec {
                order: 350,
                priority: P::Timed,
                category: C::Monitoring,
                action: "NICE CG98 does not define a repeat-measurement interval for this baby: follow the locally approved protocol for repeat bilirubin testing.",
                rationale: "The 18- and 24-hour repeat rules apply only to clinically well babies of at least 38 weeks' gestation who are more than 24 hours old; the engine does not extrapolate them.",
                timeframe: None,
                refs: &["1.4.1", "1.4.2"],
                requires_serum_confirmation: false,
            },
            DoNotUseSunlight => RuleSpec {
                order: 360,
                priority: P::Routine,
                category: C::Phototherapy,
                action: "Do not use sunlight to treat hyperbilirubinaemia.",
                rationale: "Sunlight exposure is not a safe or effective treatment.",
                timeframe: None,
                refs: &["1.4.7"],
                requires_serum_confirmation: false,
            },
            AdditionalVisualInspection48h => RuleSpec {
                order: 370,
                priority: P::Routine,
                category: C::Monitoring,
                action: "This baby has a risk factor for significant hyperbilirubinaemia: ensure an additional visual inspection for jaundice during the first 48 hours.",
                rationale: "Gestation below 38 weeks, a previous sibling who needed phototherapy, intended exclusive breastfeeding, or visible jaundice in the first 24 hours increases the likelihood of significant hyperbilirubinaemia.",
                timeframe: Some(Timeframe::within(48)),
                refs: &["1.2.1", "1.2.9"],
                requires_serum_confirmation: false,
            },
            VisualAssessmentLimitations => RuleSpec {
                order: 380,
                priority: P::Routine,
                category: C::Information,
                action: "Visual inspection alone cannot estimate the bilirubin level. Examine the baby in bright, preferably natural light, including the sclerae, gums and blanched skin. Changes in skin colour caused by hyperbilirubinaemia can be harder to see in darker skin tones.",
                rationale: "NICE requires visual assessment guidance to state its limitations, including reduced visibility in darker skin.",
                timeframe: None,
                refs: &["1.2.4", "1.2.5", "1.2.6"],
                requires_serum_confirmation: false,
            },
            BreastfeedingSupport => RuleSpec {
                order: 390,
                priority: P::Routine,
                category: C::FeedingSupport,
                action: "Support the mother to continue breastfeeding: offer lactation support and encourage frequent feeding. Jaundice is not a reason to stop breastfeeding.",
                rationale: "Breastfed babies with jaundice need feeding support, not a change from breastfeeding.",
                timeframe: None,
                refs: &["1.2.2", "1.3.2", "1.3.3"],
                requires_serum_confirmation: false,
            },
            ParentCarerInformation => RuleSpec {
                order: 400,
                priority: P::Routine,
                category: C::Information,
                action: "Offer parents and carers information about neonatal jaundice tailored to their needs, including what to look for and who to contact if jaundice appears or worsens.",
                rationale: "NICE requires parent and carer information to accompany every jaundice assessment.",
                timeframe: None,
                refs: &["1.1.1", "1.3.1"],
                requires_serum_confirmation: false,
            },
            PhototherapyCareInformation => RuleSpec {
                order: 410,
                priority: P::Routine,
                category: C::Phototherapy,
                action: "Follow the CG98 phototherapy care checklist: position the baby supine unless contraindicated, protect the eyes, maximise exposed skin, maintain temperature and hydration, support short breaks for feeding, nappy changes and cuddles during standard phototherapy, and give parents information about the treatment.",
                rationale: "Phototherapy has been recommended or is in progress; the engine supplies the applicable care checklist but does not select equipment or irradiance.",
                timeframe: None,
                refs: &["1.4.11", "1.4.12", "1.4.13", "1.4.14", "1.4.15", "1.4.16", "1.4.17", "1.4.18", "1.4.19"],
                requires_serum_confirmation: false,
            },
            IvigInformation => RuleSpec {
                order: 420,
                priority: P::Routine,
                category: C::Information,
                action: "Offer parents and carers information about intravenous immunoglobulin, including why it is being considered and its possible adverse effects.",
                rationale: "NICE requires information for parents and carers when intravenous immunoglobulin is considered.",
                timeframe: None,
                refs: &["1.8.2"],
                requires_serum_confirmation: false,
            },
            ExchangeTransfusionInformation => RuleSpec {
                order: 430,
                priority: P::Routine,
                category: C::Information,
                action: "Offer parents and carers information about exchange transfusion, including why it is being considered, its possible adverse effects and when it will be possible to see and hold the baby.",
                rationale: "NICE requires information for parents and carers when exchange transfusion is considered.",
                timeframe: None,
                refs: &["1.9.1"],
                requires_serum_confirmation: false,
            },
            NoIcterometer => RuleSpec {
                order: 440,
                priority: P::Routine,
                category: C::Measurement,
                action: "Do not use an icterometer to measure the bilirubin level.",
                rationale: "NICE excludes icterometers from bilirubin measurement.",
                timeframe: None,
                refs: &["1.2.17"],
                requires_serum_confirmation: false,
            },
            DoNotUsePredictionTests => RuleSpec {
                order: 450,
                priority: P::Routine,
                category: C::Measurement,
                action: "Do not use umbilical cord blood bilirubin, end-tidal carbon monoxide or umbilical cord blood direct antiglobulin testing to predict significant hyperbilirubinaemia.",
                rationale: "NICE excludes these prediction tests.",
                timeframe: None,
                refs: &["1.2.8"],
                requires_serum_confirmation: false,
            },
        }
    }

    pub fn source_references(&self) -> Vec<SourceReference> {
        self.spec()
            .refs
            .iter()
            .map(|r| {
                if *r == "threshold-table" {
                    SourceReference {
                        product_id: NICE_PRODUCT_ID.into(),
                        reference: "treatment threshold graphs".into(),
                        url: NICE_THRESHOLD_RESOURCE_URL.into(),
                    }
                } else {
                    SourceReference {
                        product_id: NICE_PRODUCT_ID.into(),
                        reference: (*r).into(),
                        url: NICE_RECOMMENDATIONS_URL.into(),
                    }
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn catalogue_orders_are_unique_and_ascending() {
        let orders: Vec<u32> = RuleCode::all().iter().map(|c| c.spec().order).collect();
        let unique: HashSet<u32> = orders.iter().copied().collect();
        assert_eq!(unique.len(), orders.len(), "duplicate rule order");
        let mut sorted = orders.clone();
        sorted.sort_unstable();
        assert_eq!(
            sorted, orders,
            "RuleCode::all() must follow stable rule order"
        );
    }

    #[test]
    fn every_code_has_at_least_one_source_reference() {
        for code in RuleCode::all() {
            assert!(
                !code.source_references().is_empty(),
                "{} has no source ref",
                code.as_str()
            );
        }
    }

    #[test]
    fn codes_are_screaming_snake_case() {
        for code in RuleCode::all() {
            let s = code.as_str();
            assert!(
                s.chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'),
                "{s} is not SCREAMING_SNAKE_CASE"
            );
        }
    }
}
