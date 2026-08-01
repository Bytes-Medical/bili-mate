# ADR-006: CG98 rather than CKS

Status: Accepted  
Date: 2026-08-01

## Context

The initial product idea referenced the NICE-hosted Clinical Knowledge Summary. CKS is third-party content with separate rights, while NICE CG98 contains the neonatal jaundice recommendations and treatment resources needed for the engine.

## Decision

Base v1 clinical rules and licensed text on current NICE CG98 and its official treatment graphs. Do not copy or republish CKS wording without a separate licence.

## Consequences

- Clinical provenance maps directly to numbered CG98 recommendations.
- NICE UK Open Content Licence conditions and third-party rights checks remain release gates.
- CKS may be linked as external context but is not a rule or content source.
