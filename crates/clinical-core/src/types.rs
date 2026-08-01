//! Scalar domain types (spec 03). Constructors validate ranges so invalid
//! clinical scalars are unrepresentable inside the engine.

use serde::{Deserialize, Serialize};

/// Assessment ages run from birth through 27 days 23 h 59 min (CLIN-011).
pub const MAX_AGE_MINUTES: u32 = 40_319;
/// Treatment lines exist only through 336 hours inclusive (CLIN-012, PRD-002).
pub const TREATMENT_LINE_MAX_AGE_MINUTES: u32 = 20_160;
/// Exact age 1,440 minutes belongs to the conservative first-day pathway
/// (CLIN-051); the more-than-24-hours pathway starts at 1,441.
pub const FIRST_DAY_MAX_MINUTES: u32 = 1_440;
/// Preterm formulas plateau from 72 hours (CLIN-020).
pub const PRETERM_PLATEAU_MINUTES: u32 = 4_320;
/// 48 hours, the additional-visual-inspection window (NICE 1.2.9).
pub const FIRST_48_HOURS_MINUTES: u32 = 2_880;
/// Prolonged jaundice starts strictly beyond 14 days for gestation >= 37 weeks.
pub const PROLONGED_TERM_MINUTES: u32 = 20_160;
/// Prolonged jaundice starts strictly beyond 21 days for gestation < 37 weeks.
pub const PROLONGED_PRETERM_MINUTES: u32 = 30_240;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct GestationalWeeks(u8);

impl GestationalWeeks {
    pub const MIN: u8 = 23;
    pub const MAX: u8 = 42;

    pub fn new(weeks: u8) -> Result<Self, String> {
        if (Self::MIN..=Self::MAX).contains(&weeks) {
            Ok(Self(weeks))
        } else {
            Err(format!(
                "gestational age must be {} through {} completed weeks",
                Self::MIN,
                Self::MAX
            ))
        }
    }

    pub fn value(&self) -> u8 {
        self.0
    }

    /// The preterm treatment curves apply below 38 completed weeks.
    pub fn is_preterm(&self) -> bool {
        self.0 < 38
    }
}

impl TryFrom<u8> for GestationalWeeks {
    type Error = String;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        Self::new(v)
    }
}

impl From<GestationalWeeks> for u8 {
    fn from(v: GestationalWeeks) -> u8 {
        v.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub struct AgeMinutes(u32);

impl AgeMinutes {
    pub fn new(minutes: u32) -> Result<Self, String> {
        if minutes <= MAX_AGE_MINUTES {
            Ok(Self(minutes))
        } else {
            Err(format!("age must be 0 through {MAX_AGE_MINUTES} minutes"))
        }
    }

    pub fn value(&self) -> u32 {
        self.0
    }

    pub fn is_first_day(&self) -> bool {
        self.0 <= FIRST_DAY_MAX_MINUTES
    }

    pub fn within_treatment_line_range(&self) -> bool {
        self.0 <= TREATMENT_LINE_MAX_AGE_MINUTES
    }
}

impl TryFrom<u32> for AgeMinutes {
    type Error = String;
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        Self::new(v)
    }
}

impl From<AgeMinutes> for u32 {
    fn from(v: AgeMinutes) -> u32 {
        v.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u16", into = "u16")]
pub struct BilirubinUmolL(u16);

impl BilirubinUmolL {
    pub const MAX: u16 = 1_000;

    pub fn new(value: u16) -> Result<Self, String> {
        if value <= Self::MAX {
            Ok(Self(value))
        } else {
            Err(format!("bilirubin must be 0 through {} umol/L", Self::MAX))
        }
    }

    pub fn value(&self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for BilirubinUmolL {
    type Error = String;
    fn try_from(v: u16) -> Result<Self, Self::Error> {
        Self::new(v)
    }
}

impl From<BilirubinUmolL> for u16 {
    fn from(v: BilirubinUmolL) -> u16 {
        v.0
    }
}

/// Unknown is distinct from confirmed absence (PRD-008).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriState {
    Present,
    Absent,
    Unknown,
}

impl TriState {
    pub fn is_present(&self) -> bool {
        matches!(self, TriState::Present)
    }

    pub fn is_absent(&self) -> bool {
        matches!(self, TriState::Absent)
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, TriState::Unknown)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementMethod {
    Serum,
    Transcutaneous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TreatmentMode {
    None,
    Phototherapy,
    IntensifiedPhototherapy,
    PostPhototherapy,
    PostExchange,
}

/// Fixed clinical priority order (CLIN-047). Derived `Ord` ranks
/// `Emergency` first, so a smaller value is a higher clinical priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Emergency,
    Immediate,
    Urgent,
    Treatment,
    Timed,
    Routine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThresholdRelation {
    Below,
    At,
    Above,
    NotAvailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    Rising,
    Stable,
    Falling,
}

/// Operating mode (CLIN-003/CLIN-004). The engine labels demonstration
/// output; release gating is enforced outside the core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Demonstration,
    Clinical,
}
