# ADR-001: Rust/Axum with an exact clinical core

Status: Accepted  
Date: 2026-08-01

## Context

Clinical calculations must be deterministic, memory-safe, independently testable and protected from display rounding. The service also needs a compact portable binary and a typed HTTP boundary.

## Decision

Implement a pure Rust `clinical-core` using checked rational integer arithmetic. Use Axum/Tokio for HTTP around the core. The core has no I/O, clock, network, environment or web-framework dependency.

## Consequences

- Clinical decisions are reproducible across clients and deployments.
- Arithmetic failures become typed safety failures.
- Web and future native clients consume the API rather than share code.
- The team must maintain Rust and OpenAPI expertise.
