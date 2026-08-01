# Security and privacy

## Security objectives

1. Prevent incorrect or tampered clinical output.
2. Prevent clinical input from being retained, leaked or attached to identities.
3. Keep the authoritative rule pack and release state trustworthy.
4. Remain available enough for the pilot while making outages unmistakable.
5. Limit abuse of the unauthenticated public API.

The service processes special-category health information even without direct identifiers. “No PII” in product scope means no patient identifiers and no retained patient record; it does not mean clinical values can be treated as non-sensitive.

## Data inventory

| Data | Location | Retention | Logging |
|---|---|---|---|
| Birth/assessment/collection timestamps | Browser memory only | Until clear, navigation or tab close | Never |
| Derived ages, gestation, clinical flags and bilirubin values | Browser memory and API process memory | Request/session only | Never |
| Evaluation response and receipt | Browser memory | Until clear, navigation or tab close | Never |
| Professional-use acknowledgement | Browser `sessionStorage` | Browser session | Never |
| Request ID, route, status, latency, release versions | API operational log | 30 days | Allowed |
| Source IP | WAF/load-balancer security controls | Maximum 30 days | Security log only, access-controlled |
| Aggregate availability and latency | Metrics | 13 months | No clinical labels |
| Rule packs and source manifests | Repository/artifact store | Product lifetime | Version controlled |

## Threat model

| Threat | Consequence | Required controls |
|---|---|---|
| Altered threshold data or binary | Incorrect treatment advice | Reviewed immutable pack, hashes, signed build provenance, startup self-tests, two-person release |
| Wrong age, gestation, unit or method | Wrong threshold or measurement advice | Restricted types/ranges, local time review, fixed units, explicit method, boundary tests |
| Replayed/cached result | Result applied to wrong assessment | POST/no-store, no service-worker storage, evaluation/version display, no “last result” fallback |
| Stale guidance | Obsolete recommendation | Weekly source monitor, explicit source date, immutable pack IDs, `409` refresh flow |
| Missing danger sign | False reassurance | Required tri-state fields, unknown blocks reassurance, emergency priority |
| Tampered client request | Incorrect result | Server-side validation and all clinical logic server-side |
| Denial of service | Clinical calculation unavailable | WAF, rate limits, autoscaling, multi-AZ, fail-closed UI, local-protocol message |
| Public API scraping/abuse | Cost or availability impact | UK geo restriction, per-IP throttling, body/method limits, anomaly alerting |
| Sensitive logs/telemetry | Health-data disclosure | Allowlisted logging fields, body capture disabled, automated log tests, short retention |
| Supply-chain compromise | Altered code or data | Locked dependencies, provenance, vulnerability scanning, protected branches, reproducible release metadata |
| Cross-site scripting | Clinical-data disclosure or altered display | React escaping, strict CSP, no unsafe HTML, dependency review, SRI where appropriate |
| Clickjacking | Misleading workflow | `frame-ancestors 'none'`, `X-Frame-Options: DENY` for compatibility |
| Unintended parent use | Unsafe self-treatment | Professional labelling, session acknowledgement, no parent workflow, no consumer marketing |

## Security requirements

| ID | Requirement |
|---|---|
| SEC-001 | Production endpoints MUST use TLS 1.2 or later with valid automated certificate rotation. |
| SEC-002 | HTTP requests MUST redirect to HTTPS only for safe GET routes; clinical POST requests over HTTP MUST be rejected rather than redirected. |
| SEC-003 | Clinical-mode deployments MUST restrict access to UK geolocation at the edge. |
| SEC-004 | Browser origins MUST be allowlisted; wildcard CORS is prohibited. |
| SEC-005 | The WAF MUST permit only documented methods, enforce body size, rate limits and baseline managed protections. |
| SEC-006 | The application MUST parse JSON with duplicate-key rejection and reject unknown schema properties. |
| SEC-007 | Application logs MUST use an explicit field allowlist and MUST never serialise domain/request/response objects. |
| SEC-008 | Metrics MUST NOT use gestation, age, bilirubin, recommendation, evaluation ID, source IP or clinical flags as labels. |
| SEC-009 | Error trackers MUST disable request bodies, response bodies, breadcrumbs containing rendered result text and automatic DOM capture. |
| SEC-010 | Secrets MUST be stored in the cloud secret manager or deployment secret mechanism, never the image, repository or client bundle. |
| SEC-011 | Containers MUST run non-root, drop Linux capabilities, use a read-only root filesystem and write only to a bounded temporary filesystem. |
| SEC-012 | Production images MUST include a software bill of materials and signed provenance tied to the source commit. |
| SEC-013 | Dependency, licence and vulnerability checks MUST block release for unresolved critical/high findings unless the security owner records a time-bounded exception. |
| SEC-014 | The rule-pack digest compiled into the binary MUST match the release manifest and readiness response. |
| SEC-015 | The API MUST set HSTS, `X-Content-Type-Options: nosniff`, a restrictive referrer policy and a restrictive permissions policy. |
| SEC-016 | The web application MUST use a nonce- or hash-based CSP with `object-src 'none'`, `base-uri 'none'` and `frame-ancestors 'none'`. |
| SEC-017 | No third-party advertising, session replay, social widget or third-party form analytics may execute on assessment or result pages. |
| SEC-018 | Security access logs MUST be access-controlled, encrypted and deleted after no more than 30 days unless an active incident requires a documented legal hold. |
| SEC-019 | A privacy notice MUST describe transient health-data processing and security-log IP processing before use. |
| SEC-020 | Penetration testing MUST be completed before advisory pilot and after material trust-boundary changes. |

## Headers

Minimum web response policy:

```text
Strict-Transport-Security: max-age=31536000; includeSubDomains
X-Content-Type-Options: nosniff
Referrer-Policy: no-referrer
Permissions-Policy: camera=(), microphone=(), geolocation=(), payment=(), usb=()
Content-Security-Policy: default-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'; connect-src 'self' https://api.bili-mate.uk
```

The final CSP may add hashes/nonces and required static asset hosts but MUST NOT add `unsafe-eval`. `unsafe-inline` is prohibited for scripts.

API JSON responses additionally use `Content-Security-Policy: default-src 'none'; frame-ancestors 'none'`.

## Rate and resource controls

- 60 requests/minute/IP sustained and burst 20 for the evaluation endpoint.
- Metadata and curve endpoints may allow 120 requests/minute/IP and be edge-cached.
- Maximum request body 64 KiB.
- Maximum decompressed body 64 KiB; request content encoding is rejected in v1 to avoid decompression ambiguity.
- Header section maximum 16 KiB.
- Request read timeout 5 seconds; evaluation timeout 2 seconds; total response timeout 5 seconds.
- Maximum 64 measurements.
- No automatic server retry of a clinical evaluation.

Rate-limit keys are processed at the edge; the application receives no persistent user identity.

## Privacy behaviour

The controller/processor roles for a pilot must be documented with the deploying organisation. A Data Protection Impact Assessment is required before the pilot even though the service retains no patient record, because health data is transmitted and processed.

The API contract prevents direct identifiers. If a client or proxy adds identifiers in headers, query strings or unsupported JSON properties, they must be rejected or stripped according to deployment policy and never logged.

No data is sold, used for advertising, used to train models, or combined to profile clinicians or patients.

## Incident response

Security or clinical-integrity incidents use one severity process:

1. disable clinical mode or roll back if output integrity may be affected;
2. preserve only necessary security evidence under legal hold;
3. notify the Clinical Safety Officer, security owner and service owner;
4. assess regulatory, data-protection and NHS reporting obligations;
5. publish an operational notice without disclosing clinical data;
6. correct through reviewed change control; and
7. document root cause, affected releases, residual risk and prevention tests.

The operator maintains a public vulnerability-reporting route and a private urgent clinical-safety reporting route.
