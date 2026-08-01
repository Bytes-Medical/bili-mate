# API contract

## Protocol

The API is HTTPS-only JSON over HTTP/1.1 or HTTP/2. The normative wire schema is [OpenAPI 3.1](openapi.yaml). This document defines behaviour not conveniently expressed in schema.

Base production URL: `https://api.bili-mate.uk`  
Media type: `application/json`  
API version prefix: `/v1`

The service has no login or API key in v1. Public access does not change the professional-only intended use.

## Common request behaviour

| ID | Requirement |
|---|---|
| API-001 | Clients MUST send `Accept: application/json`. POST requests MUST send `Content-Type: application/json`. |
| API-002 | Clients MAY send `X-Request-ID` containing 1–64 printable ASCII characters; otherwise the server generates one. |
| API-003 | Every response MUST return `X-Request-ID`. |
| API-004 | Evaluation requests larger than 64 KiB MUST receive `413 Payload Too Large`. |
| API-005 | The server MUST enforce a default limit of 60 requests per minute per source IP with a burst of 20. |
| API-006 | A rate-limited response MUST use `429`, include `Retry-After`, and contain no clinical result. |
| API-007 | Evaluation endpoints MUST return `Cache-Control: no-store`. Metadata MAY use a public cache lifetime no longer than one hour. |
| API-008 | All timestamps returned by the server MUST be RFC 3339 UTC with `Z`. |
| API-009 | JSON responses MUST be UTF-8 and MUST NOT contain `NaN`, infinity or implementation-specific numeric values. |

## `GET /v1/guidelines/active`

Returns the single active rule pack, its scope, sources, hashes, clinical approval status and required notices.

- `200`: active metadata.
- `503`: no clinically active rule pack is available in clinical mode.

Clients MUST retrieve this endpoint when starting an assessment and use the returned ID in the evaluation request. A client may cache metadata for one hour but MUST refresh after a stale-pack conflict.

## `POST /v1/evaluations`

Evaluates one transient assessment.

Processing order:

1. enforce transport and payload controls;
2. parse JSON with duplicate-key rejection;
3. validate the OpenAPI schema and domain invariants;
4. resolve the requested active rule pack;
5. normalise and sort measurements;
6. calculate exact thresholds and serum trend;
7. evaluate clinical rules;
8. resolve priorities and contradictions;
9. create the decision receipt and response;
10. discard request and response clinical content after sending.

| Status | Meaning |
|---:|---|
| `200` | Valid assessment evaluated, including incomplete-information or out-of-scope clinical warnings |
| `400` | Malformed JSON, duplicate key or invalid content type |
| `409` | Requested rule pack is unavailable, retired, draft in clinical mode, or no longer active |
| `413` | Body too large |
| `422` | Schema or domain validation failure |
| `429` | Rate limit exceeded |
| `500` | Unexpected internal error; no partial clinical result |
| `503` | Engine/rule pack unavailable or safety self-check failed |

| ID | Requirement |
|---|---|
| API-010 | A `200` response MUST represent a completed deterministic evaluation, not merely accepted work. |
| API-011 | The server MUST NOT return a partial evaluation after any arithmetic, rule-pack-integrity or internal safety failure. |
| API-012 | The API MUST return the requested rule-pack ID and the source update date in every successful response. |
| API-013 | The API MUST not silently substitute the active rule pack for a different requested ID. |
| API-014 | A stale-pack `409` MUST include the active pack ID and metadata URL but MUST NOT include a clinical result. |
| API-015 | The application MUST NOT log request bodies, response bodies, normalised inputs, digests, measurements or recommendation codes tied to a request ID. |

## `GET /v1/threshold-curves/{rule_pack_id}`

Query parameters:

- `gestational_age_completed_weeks`: required integer 23–42;
- `resolution_minutes`: optional enum `1`, `5`, `15`, `30`, `60`, default `60`.

The endpoint returns exact source metadata plus display points from birth through 336 hours. It contains no clinical action and accepts no patient data.

- `200`: curve points.
- `404`: unknown rule pack.
- `409`: pack exists but is not publishable in current mode.
- `422`: invalid query.

Curve values are display values. Clients MUST use `POST /v1/evaluations`, not locally compare measurements to curve points.

## `GET /v1/legal`

Returns:

- intended-purpose statement;
- intended-user and UK-only restrictions;
- NICE attribution and disclaimer;
- non-endorsement statement;
- local laboratory warning;
- privacy summary; and
- links to the current source, licence, privacy notice and terms.

The web client MUST display the current server-provided legal content rather than ship a divergent copy.

## Health endpoints

`GET /health/live` proves that the process can serve HTTP. It does not load or evaluate clinical rules.

`GET /health/ready` verifies:

- the configured rule pack exists;
- its manifest and content hash pass;
- required self-test vectors pass;
- the service is in an authorised mode; and
- no startup safety fault is active.

Readiness returns only `{"status":"ready"}` or an unavailable problem. It MUST NOT disclose dependency, host or source-path details publicly.

## Problem details

Errors use `application/problem+json` with:

- `type`: stable documentation URI;
- `title`: short category;
- `status`: HTTP status;
- `detail`: safe human explanation;
- `instance`: request URN containing request ID;
- `code`: stable machine code;
- `errors`: optional array of `{pointer, code, message}`; and
- `active_rule_pack_id`: only for rule-pack conflicts.

| ID | Requirement |
|---|---|
| API-016 | Error details MUST NOT include stack traces, source paths, internal hostnames or echoed clinical values. |
| API-017 | Validation pointers MUST identify fields but messages MUST NOT reproduce the submitted value. |
| API-018 | Stable error codes MUST be documented and MUST NOT change meaning within API v1. |

## CORS and browser security

- Allowed origins are an explicit deployment configuration; wildcard origin is prohibited in clinical mode.
- Allowed methods are `GET`, `POST`, and `OPTIONS`.
- Allowed request headers are `Accept`, `Content-Type`, and `X-Request-ID`.
- No credentials are permitted because v1 has no browser authentication.
- Preflight cache lifetime is at most one hour.

Native clients are not governed by browser CORS and receive no special API capability.

## Versioning and compatibility

API and clinical versions are independent:

- additive optional fields and new enum-independent endpoints may remain `/v1`;
- removal, changed field meaning, changed requiredness or new enum values that existing clients cannot safely handle require `/v2`;
- clinical changes always create a new rule pack;
- engine fixes that do not alter approved clinical outputs increment the engine patch version;
- any engine fix that changes clinical outputs requires a new rule pack or a documented clinical re-approval of the existing pack revision.

The OpenAPI definition MUST be committed and tagged with each API release. Release CI MUST generate TypeScript, Swift and Kotlin clients and compile minimal consumer fixtures before publishing.

## Example catalogue

The [example catalogue](examples/README.md) and following fixtures are part of specification review:

- [`normal-below-threshold-request.json`](examples/normal-below-threshold-request.json)
- [`normal-below-threshold-response.json`](examples/normal-below-threshold-response.json)
- [`early-jaundice-request.json`](examples/early-jaundice-request.json)
- [`phototherapy-request.json`](examples/phototherapy-request.json)
- [`intensified-phototherapy-request.json`](examples/intensified-phototherapy-request.json)
- [`exchange-escalation-request.json`](examples/exchange-escalation-request.json)
- [`prolonged-jaundice-request.json`](examples/prolonged-jaundice-request.json)
- [`stale-rule-pack-problem.json`](examples/stale-rule-pack-problem.json)
- [`validation-problem.json`](examples/validation-problem.json)
