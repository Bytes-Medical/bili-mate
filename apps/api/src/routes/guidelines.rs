//! `GET /v1/guidelines/active` and `GET /v1/threshold-curves/{rule_pack_id}`
//! (spec 04). Metadata may be publicly cached for up to one hour; curve
//! points are display values only and carry no clinical action.

use std::collections::HashMap;

use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderValue};
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use clinical_core::thresholds::treatment_thresholds;
use clinical_core::types::{AgeMinutes, GestationalWeeks, TREATMENT_LINE_MAX_AGE_MINUTES};
use guideline_data::RulePackSummary;

use crate::legal::{legal_notices, LegalNotices};
use crate::middleware::Ctx;
use crate::problem::{FieldError, Problem};
use crate::state::AppState;

const PUBLIC_CACHE: HeaderValue = HeaderValue::from_static("public, max-age=3600");

#[derive(Serialize)]
struct WireSource {
    id: String,
    url: String,
    retrieved_on: String,
    sha256: String,
}

#[derive(Serialize)]
struct WireScope {
    gestation_weeks_minimum: i64,
    gestation_weeks_maximum: i64,
    assessment_age_minutes_maximum: i64,
    treatment_age_minutes_maximum: i64,
    bilirubin_unit: &'static str,
}

/// `RulePackMetadata` per the OpenAPI contract.
#[derive(Serialize)]
struct WireRulePackMetadata {
    id: String,
    guideline_id: String,
    source_updated_on: String,
    status: String,
    content_sha256: String,
    guideline_title: String,
    market: String,
    language: String,
    scope: WireScope,
    sources: Vec<WireSource>,
    legal: LegalNotices,
}

pub async fn active(State(state): State<AppState>, ctx: Ctx) -> Response {
    let Some(pack) = state.pack() else {
        return Problem::engine_unavailable().into_response_with_request_id(&ctx.request_id);
    };
    let rule_pack = &pack.file.rule_pack;
    let summary = pack.summary();
    let body = WireRulePackMetadata {
        id: summary.id,
        guideline_id: summary.guideline_id,
        source_updated_on: summary.source_updated_on,
        status: summary.status,
        content_sha256: summary.content_sha256,
        guideline_title: rule_pack.guideline_title.clone(),
        market: rule_pack.market.clone(),
        language: rule_pack.language.clone(),
        scope: WireScope {
            gestation_weeks_minimum: rule_pack.scope.gestational_age_completed_weeks.minimum,
            gestation_weeks_maximum: rule_pack.scope.gestational_age_completed_weeks.maximum,
            assessment_age_minutes_maximum: rule_pack.scope.assessment_age_minutes.maximum,
            treatment_age_minutes_maximum: rule_pack.scope.treatment_threshold_age_minutes.maximum,
            bilirubin_unit: "umol/L",
        },
        sources: rule_pack
            .sources
            .iter()
            .map(|s| WireSource {
                id: s.id.clone(),
                url: s.url.clone(),
                retrieved_on: s.retrieved_on.clone(),
                sha256: s.sha256.clone(),
            })
            .collect(),
        legal: legal_notices(),
    };
    ([(header::CACHE_CONTROL, PUBLIC_CACHE)], axum::Json(body)).into_response()
}

#[derive(Serialize)]
struct WireCurvePoint {
    age_minutes: u32,
    phototherapy_threshold_umol_l: f64,
    exchange_threshold_umol_l: f64,
}

#[derive(Serialize)]
struct WireThresholdCurve {
    rule_pack: RulePackSummary,
    gestational_age_completed_weeks: u8,
    resolution_minutes: u32,
    display_only: bool,
    points: Vec<WireCurvePoint>,
}

pub async fn threshold_curve(
    State(state): State<AppState>,
    ctx: Ctx,
    Path(rule_pack_id): Path<String>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let rid = ctx.request_id;
    let Some(pack) = state.pack() else {
        return Problem::engine_unavailable().into_response_with_request_id(&rid);
    };
    if rule_pack_id != pack.file.rule_pack.id {
        return Problem::not_found().into_response_with_request_id(&rid);
    }

    let mut errors: Vec<FieldError> = Vec::new();
    let gestation = query
        .get("gestational_age_completed_weeks")
        .and_then(|v| v.parse::<u8>().ok())
        .and_then(|v| GestationalWeeks::new(v).ok());
    if gestation.is_none() {
        errors.push(FieldError {
            pointer: "/gestational_age_completed_weeks".into(),
            code: "SCHEMA_INVALID".into(),
            message: "gestational_age_completed_weeks must be an integer from 23 through 42."
                .into(),
        });
    }
    let resolution = match query.get("resolution_minutes") {
        None => Some(60u32),
        Some(v) => match v.parse::<u32>() {
            Ok(r @ (1 | 5 | 15 | 30 | 60)) => Some(r),
            _ => None,
        },
    };
    if resolution.is_none() {
        errors.push(FieldError {
            pointer: "/resolution_minutes".into(),
            code: "SCHEMA_INVALID".into(),
            message: "resolution_minutes must be one of 1, 5, 15, 30 or 60.".into(),
        });
    }
    if !errors.is_empty() {
        return Problem::validation_failed(errors).into_response_with_request_id(&rid);
    }
    let (gestation, resolution) = (
        gestation.expect("validated"),
        resolution.expect("validated"),
    );

    // Display points from birth through 336 hours. Every permitted
    // resolution divides 20,160, so the final point lands exactly on the
    // scope limit; a safety failure here is a 503, never a partial curve.
    let mut points = Vec::with_capacity((TREATMENT_LINE_MAX_AGE_MINUTES / resolution + 1) as usize);
    let mut age = 0u32;
    while age <= TREATMENT_LINE_MAX_AGE_MINUTES {
        let minutes = AgeMinutes::new(age).expect("within range");
        let pair = match treatment_thresholds(gestation, minutes) {
            Ok(Some(pair)) => pair,
            _ => return Problem::engine_safety_check_failed().into_response_with_request_id(&rid),
        };
        let (photo, exchange) = match (
            pair.phototherapy.display_tenths(),
            pair.exchange.display_tenths(),
        ) {
            (Ok(p), Ok(e)) => (p, e),
            _ => return Problem::engine_safety_check_failed().into_response_with_request_id(&rid),
        };
        points.push(WireCurvePoint {
            age_minutes: age,
            phototherapy_threshold_umol_l: photo as f64 / 10.0,
            exchange_threshold_umol_l: exchange as f64 / 10.0,
        });
        age += resolution;
    }

    let body = WireThresholdCurve {
        rule_pack: pack.summary(),
        gestational_age_completed_weeks: gestation.value(),
        resolution_minutes: resolution,
        display_only: true,
        points,
    };
    ([(header::CACHE_CONTROL, PUBLIC_CACHE)], axum::Json(body)).into_response()
}
