# ADR-004: Public API without login

Status: Accepted  
Date: 2026-08-01

## Context

The selected v1 avoids clinician-account and identity-provider scope. The intended users remain healthcare professionals, but identity is not technically verified.

## Decision

Expose the UK-restricted API without authentication. Apply professional-use acknowledgement/labelling, WAF controls, per-IP rate limiting, strict CORS for browsers and explicit privacy terms.

## Consequences

- Integration and pilot onboarding are simpler.
- Intended-user controls are labelling and governance controls, not identity controls.
- Unintended parent use and automated abuse remain hazards requiring monitoring.
- Organisation OIDC is a preferred future enhancement, not part of v1.
