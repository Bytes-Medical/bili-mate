//! Shared application state: the verified embedded rule pack, configuration
//! and the evaluation rate limiter. The pack is loaded and self-tested once
//! at startup; a failure keeps the service alive but never ready (OPS-004).

use std::sync::Arc;

use guideline_data::{load_embedded_pack, LoadError, VerifiedPack};

use crate::config::Config;
use crate::rate_limit::RateLimiter;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub pack: Arc<Result<VerifiedPack, LoadError>>,
    pub limiter: Arc<RateLimiter>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        let limiter = RateLimiter::new(config.rate_limit_per_minute, config.rate_limit_burst);
        Self {
            config: Arc::new(config),
            pack: Arc::new(load_embedded_pack()),
            limiter: Arc::new(limiter),
        }
    }

    /// The verified pack, or `None` when integrity checks failed at startup.
    pub fn pack(&self) -> Option<&VerifiedPack> {
        self.pack.as_ref().as_ref().ok()
    }

    /// Readiness (spec 04 health): pack exists, integrity and self-tests
    /// passed, and the configured mode is authorised. Clinical mode requires
    /// an `active` pack plus a release-authorisation reference (CLIN-003,
    /// SAFE-024, OPS-004); demonstration mode accepts a draft pack but every
    /// response is labelled not-for-patient-care (CLIN-004).
    pub fn ready(&self) -> bool {
        match self.pack() {
            None => false,
            Some(pack) => match self.config.mode {
                clinical_core::types::Mode::Demonstration => true,
                clinical_core::types::Mode::Clinical => {
                    pack.clinical_mode_allowed() && self.config.release_authorisation_ref.is_some()
                }
            },
        }
    }
}
