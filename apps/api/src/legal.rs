//! Legal and clinical-use notices (spec 04 `/v1/legal`, PRD-018, SAFE-001,
//! SAFE-013–SAFE-015). The intended-purpose statement is the single
//! controlled wording from spec 01 and must remain identical everywhere it
//! appears.

use std::collections::BTreeMap;

use serde::Serialize;

pub const INTENDED_PURPOSE: &str = "Bili Mate is software intended to support registered healthcare professionals in the United Kingdom with assessment and management decisions for jaundice in newborn babies from birth to less than 28 days of age. It calculates and presents bilirubin treatment thresholds up to and including 14 days of age and applies relevant recommendations from NICE guideline CG98. It accepts manually entered clinical observations and does not directly acquire data from a medical device. Its output is advisory and must be reviewed by a suitably trained healthcare professional before action.";

#[derive(Debug, Clone, Serialize)]
pub struct LegalNotices {
    pub intended_purpose: &'static str,
    pub intended_users: &'static str,
    pub uk_only: bool,
    pub professional_use_warning: &'static str,
    pub local_pathology_warning: &'static str,
    pub nice_attribution: &'static str,
    pub non_endorsement: &'static str,
    pub privacy_summary: &'static str,
    pub links: BTreeMap<&'static str, &'static str>,
}

pub fn legal_notices() -> LegalNotices {
    LegalNotices {
        intended_purpose: INTENDED_PURPOSE,
        intended_users: "Registered healthcare professionals in the United Kingdom.",
        uk_only: true,
        professional_use_warning: "This result requires review by a suitably trained healthcare professional and does not replace clinical judgement or local policy.",
        local_pathology_warning: "Consult the local pathology laboratory because bilirubin assay results may vary.",
        nice_attribution: "This product includes content from NICE guideline CG98 (Jaundice in newborn babies under 28 days), used under the NICE UK Open Content Licence. NICE guidance is prepared for the National Health Service in England and was accurate at the source update date shown; it may be updated or withdrawn.",
        non_endorsement: "NICE has not endorsed, and is not responsible for, Bili Mate.",
        privacy_summary: "The server does not retain assessment content after responding. No patient identifiers are accepted.",
        links: BTreeMap::from([
            ("guideline", "https://www.nice.org.uk/guidance/cg98"),
            ("licence", "https://www.nice.org.uk/reusing-our-content/nice-uk-open-content-licence"),
            ("privacy", "https://bili-mate.uk/privacy"),
            ("terms", "https://bili-mate.uk/terms"),
        ]),
    }
}
