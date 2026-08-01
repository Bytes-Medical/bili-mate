//! Health endpoints (spec 04). Liveness proves the process serves HTTP;
//! readiness verifies pack integrity, self-tests and mode authorisation and
//! discloses no dependency, host or source-path details.

use axum::extract::State;
use axum::http::HeaderValue;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::middleware::Ctx;
use crate::problem::Problem;
use crate::state::AppState;

#[derive(Serialize)]
struct HealthStatus {
    status: &'static str,
}

pub async fn live() -> Response {
    no_store(axum::Json(HealthStatus { status: "live" }).into_response())
}

pub async fn ready(State(state): State<AppState>, ctx: Ctx) -> Response {
    if state.ready() {
        no_store(axum::Json(HealthStatus { status: "ready" }).into_response())
    } else {
        Problem::engine_unavailable().into_response_with_request_id(&ctx.request_id)
    }
}

fn no_store(mut response: Response) -> Response {
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    );
    response
}
