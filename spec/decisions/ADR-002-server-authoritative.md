# ADR-002: Server-authoritative calculations

Status: Accepted  
Date: 2026-08-01

## Context

Duplicating rules across web, Android and iOS creates clinical drift and difficult update control. Offline capability would require a separately governed distribution and execution mechanism.

## Decision

All clinical decisions are produced by the versioned Rust API. Clients may cache static help and display curves but cannot compare measurements or produce recommendations offline.

## Consequences

- One rule implementation and release record serves every client.
- Clients fail closed when the API is unavailable.
- Connectivity and availability become explicit workflow dependencies.
- Offline rule execution requires a future ADR and renewed safety validation.
