# ADR-009: Next.js static export for the reference web client

Status: Accepted  
Date: 2026-08-01

## Context

The reference web specification requires a separately built static React/TypeScript application with no clinical logic, no server-side handling of clinical data, and a strict CSP without `unsafe-inline` scripts. The implementation team has standardised on Next.js.

## Decision

Build the reference web client with Next.js (App Router) configured for static export (`output: 'export'`). No Next.js API routes, server actions or server components may receive or render clinical data; all clinical state remains in client-side memory. Image optimisation is disabled. The build pipeline generates hashes for Next.js inline bootstrap scripts so the deployed CSP satisfies SEC-016 without `unsafe-inline`.

## Consequences

- The deployed artefact remains a static bundle served from the private bucket/CDN topology, as specified.
- Server-side leakage of clinical form state is structurally impossible.
- CSP hash generation is a required build step and an automated header test guards it.
- If hash-based CSP proves brittle with a Next.js upgrade, the fallback is a Vite/React static build; the workflow and components remain portable.
