//! Typed immutable service configuration (spec 05: configuration is
//! environment-driven but validated into a typed structure at startup).
//! Runtime configuration can never change formulas, thresholds, comparators
//! or rule priorities (DATA-030) — none of those are configurable here.

use clinical_core::types::Mode;

#[derive(Debug, Clone)]
pub struct Config {
    /// Operating mode; defaults to demonstration. Clinical mode additionally
    /// requires an active pack and a release-authorisation reference before
    /// the service reports ready (OPS-004, OPS-011, CLIN-003).
    pub mode: Mode,
    pub bind_address: String,
    /// Browser origins allowed by CORS. Wildcard is prohibited (SEC-004);
    /// an empty list means no browser origin is allowed.
    pub allowed_origins: Vec<String>,
    /// Release-authorisation reference required for clinical mode (SAFE-024).
    pub release_authorisation_ref: Option<String>,
    /// Evaluation rate limit (API-005): sustained requests/minute per IP.
    pub rate_limit_per_minute: u32,
    /// Evaluation burst capacity per IP.
    pub rate_limit_burst: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let mode = match std::env::var("BILI_MATE_MODE").as_deref() {
            Err(_) | Ok("demonstration") => Mode::Demonstration,
            Ok("clinical") => Mode::Clinical,
            Ok(other) => {
                return Err(format!(
                    "BILI_MATE_MODE must be demonstration or clinical, got {other}"
                ))
            }
        };
        let allowed_origins: Vec<String> = std::env::var("BILI_MATE_ALLOWED_ORIGINS")
            .map(|v| {
                v.split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();
        if allowed_origins.iter().any(|o| o == "*") {
            return Err("wildcard CORS origin is prohibited (SEC-004)".into());
        }
        Ok(Self {
            mode,
            bind_address: std::env::var("BILI_MATE_BIND")
                .unwrap_or_else(|_| "127.0.0.1:8080".into()),
            allowed_origins,
            release_authorisation_ref: std::env::var("BILI_MATE_RELEASE_AUTHORISATION").ok(),
            rate_limit_per_minute: 60,
            rate_limit_burst: 20,
        })
    }

    /// Demonstration-mode defaults used by tests.
    pub fn for_tests() -> Self {
        Self {
            mode: Mode::Demonstration,
            bind_address: "127.0.0.1:0".into(),
            allowed_origins: vec!["https://bili-mate.uk".into()],
            release_authorisation_ref: None,
            rate_limit_per_minute: 60,
            rate_limit_burst: 20,
        }
    }
}
