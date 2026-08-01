//! Shared builder for clinical test cases. The default baby is clinically
//! well with every danger sign confirmed absent, so tests state only what
//! differs from that baseline.

use clinical_core::evaluate::{evaluate, EvaluationContext};
use clinical_core::input::{
    Assessment, ClinicalFeatures, Measurement, RiskFactors, TreatmentState,
};
use clinical_core::output::EvaluationOutcome;
use clinical_core::types::{
    AgeMinutes, BilirubinUmolL, GestationalWeeks, MeasurementMethod, Mode, TreatmentMode, TriState,
};

pub struct Case {
    pub gestation: u8,
    pub age: u32,
    pub features: ClinicalFeatures,
    pub risks: RiskFactors,
    pub measurements: Vec<Measurement>,
    pub conjugated: Option<u16>,
    pub treatment: TreatmentState,
}

#[allow(dead_code)]
impl Case {
    pub fn new(gestation: u8, age: u32) -> Self {
        Self {
            gestation,
            age,
            features: ClinicalFeatures {
                suspected_or_obvious_jaundice: TriState::Absent,
                visible_jaundice: TriState::Absent,
                clinically_well: TriState::Present,
                acute_bilirubin_encephalopathy: TriState::Absent,
                pale_chalky_stools: TriState::Absent,
                dark_urine_stains_nappy: TriState::Absent,
                rhesus_haemolytic_disease: TriState::Absent,
                abo_haemolytic_disease: TriState::Absent,
                infection_suspected: TriState::Absent,
                urinary_tract_infection_suspected: TriState::Absent,
                routine_metabolic_screen_completed: TriState::Present,
            },
            risks: RiskFactors {
                previous_sibling_required_phototherapy: TriState::Absent,
                exclusive_breastfeeding_intended: TriState::Absent,
            },
            measurements: Vec::new(),
            conjugated: None,
            treatment: TreatmentState {
                mode: TreatmentMode::None,
                started_age_minutes: None,
                stopped_age_minutes: None,
                exchange_completed_age_minutes: None,
            },
        }
    }

    pub fn jaundice(mut self) -> Self {
        self.features.suspected_or_obvious_jaundice = TriState::Present;
        self.features.visible_jaundice = TriState::Present;
        self
    }

    pub fn feature(mut self, set: impl FnOnce(&mut ClinicalFeatures)) -> Self {
        set(&mut self.features);
        self
    }

    pub fn sibling_risk(mut self) -> Self {
        self.risks.previous_sibling_required_phototherapy = TriState::Present;
        self
    }

    pub fn breastfeeding(mut self) -> Self {
        self.risks.exclusive_breastfeeding_intended = TriState::Present;
        self
    }

    pub fn risk(mut self, set: impl FnOnce(&mut RiskFactors)) -> Self {
        set(&mut self.risks);
        self
    }

    pub fn serum(self, age: u32, value: u16) -> Self {
        self.measurement(age, value, MeasurementMethod::Serum)
    }

    pub fn tcb(self, age: u32, value: u16) -> Self {
        self.measurement(age, value, MeasurementMethod::Transcutaneous)
    }

    fn measurement(mut self, age: u32, value: u16, method: MeasurementMethod) -> Self {
        let id = format!("m{}", self.measurements.len() + 1);
        self.measurements.push(Measurement {
            id,
            age_minutes: AgeMinutes::new(age).unwrap(),
            total_bilirubin_umol_l: BilirubinUmolL::new(value).unwrap(),
            method,
        });
        self
    }

    pub fn phototherapy(mut self, started: u32) -> Self {
        self.treatment = TreatmentState {
            mode: TreatmentMode::Phototherapy,
            started_age_minutes: Some(AgeMinutes::new(started).unwrap()),
            stopped_age_minutes: None,
            exchange_completed_age_minutes: None,
        };
        self
    }

    pub fn intensified(mut self, started: u32) -> Self {
        self.treatment = TreatmentState {
            mode: TreatmentMode::IntensifiedPhototherapy,
            started_age_minutes: Some(AgeMinutes::new(started).unwrap()),
            stopped_age_minutes: None,
            exchange_completed_age_minutes: None,
        };
        self
    }

    pub fn post_phototherapy(mut self, started: u32, stopped: u32) -> Self {
        self.treatment = TreatmentState {
            mode: TreatmentMode::PostPhototherapy,
            started_age_minutes: Some(AgeMinutes::new(started).unwrap()),
            stopped_age_minutes: Some(AgeMinutes::new(stopped).unwrap()),
            exchange_completed_age_minutes: None,
        };
        self
    }

    pub fn conjugated(mut self, value: u16) -> Self {
        self.conjugated = Some(value);
        self
    }

    pub fn eval(self) -> EvaluationOutcome {
        let assessment = Assessment::new(
            GestationalWeeks::new(self.gestation).unwrap(),
            AgeMinutes::new(self.age).unwrap(),
            self.features,
            self.risks,
            self.measurements,
            self.conjugated.map(|v| BilirubinUmolL::new(v).unwrap()),
            self.treatment,
        )
        .expect("test case must be domain-valid");
        evaluate(
            &assessment,
            &EvaluationContext {
                mode: Mode::Demonstration,
            },
        )
        .expect("evaluation must not fail")
    }
}

#[allow(dead_code)]
pub fn activated(outcome: &EvaluationOutcome) -> Vec<&str> {
    outcome
        .decision_trace
        .activated_rules
        .iter()
        .map(String::as_str)
        .collect()
}

#[allow(dead_code)]
pub fn recommended(outcome: &EvaluationOutcome) -> Vec<&str> {
    outcome
        .recommendations
        .iter()
        .map(|r| r.code.as_str())
        .collect()
}

#[allow(dead_code)]
pub fn primary(outcome: &EvaluationOutcome) -> &str {
    &outcome.primary_action.code
}

#[allow(dead_code)]
pub fn assert_active(outcome: &EvaluationOutcome, code: &str) {
    assert!(
        activated(outcome).contains(&code),
        "{code} should be active; active rules: {:?}",
        activated(outcome)
    );
}

#[allow(dead_code)]
pub fn assert_inactive(outcome: &EvaluationOutcome, code: &str) {
    assert!(
        !activated(outcome).contains(&code),
        "{code} should NOT be active; active rules: {:?}",
        activated(outcome)
    );
}
