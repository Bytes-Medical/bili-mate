//! `GET /v1/legal` (spec 04, PRD-018): current intended-use, content and
//! privacy notices. The web client displays this server-provided content
//! rather than shipping a divergent copy.

use axum::http::{header, HeaderValue};
use axum::response::{IntoResponse, Response};

use crate::legal::legal_notices;

pub async fn legal() -> Response {
    (
        [(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=3600"),
        )],
        axum::Json(legal_notices()),
    )
        .into_response()
}
