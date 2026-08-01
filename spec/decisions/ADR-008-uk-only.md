# ADR-008: UK-only market and distribution

Status: Accepted  
Date: 2026-08-01

## Context

CG98 is UK guidance and NICE reuse terms distinguish UK and international distribution. Regulation, units, clinical workflows and local pathology expectations also vary by market.

## Decision

Limit v1 intended purpose, marketing, hosted access and NICE-derived content distribution to the United Kingdom. Use µmol/L and a reference deployment in AWS London with UK geo restriction.

## Consequences

- International app-store listing and hosted access are excluded.
- Additional jurisdictions require licensing, guideline profiles, regulatory review and separate validation.
- Geo restriction supports but cannot replace contractual and labelling controls.
