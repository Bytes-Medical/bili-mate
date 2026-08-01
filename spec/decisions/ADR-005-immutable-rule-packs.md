# ADR-005: Immutable governed rule packs

Status: Accepted  
Date: 2026-08-01

## Context

Automatic scraping or silent rule changes would make decisions unreproducible and bypass clinical review.

## Decision

Embed immutable, versioned rule packs in a jointly authorised engine release. Monitor NICE sources for change, but update only through manual ingestion, clinical review, testing, safety approval and deployment.

## Consequences

- Every receipt identifies a reproducible clinical baseline.
- Rule changes require a release rather than a runtime content refresh.
- A source-monitor alert may force temporary suspension when current advice is uncertain.
- Previous authorised image/rule-pack pairs remain available for rollback and audit.
