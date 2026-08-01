# ADR-007: Reference web before native apps

Status: Accepted  
Date: 2026-08-01

## Context

The API must eventually serve web, Android and iOS. Building three production UIs before validating the workflow would multiply safety and usability work.

## Decision

Deliver a production-quality reference web client with the Rust API. Generate and compile-test TypeScript, Swift and Kotlin clients, but defer production native apps until the advisory pilot is accepted.

## Consequences

- The complete contract and workflow are validated once first.
- Native work begins with a stable generated client and established safety patterns.
- Native offline calculations and device-specific persistence are not permitted in v1.
