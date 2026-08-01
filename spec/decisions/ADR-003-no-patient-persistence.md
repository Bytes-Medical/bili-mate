# ADR-003: No patient persistence

Status: Accepted  
Date: 2026-08-01

## Context

The first release needs serial values for one evaluation but does not need accounts or longitudinal records. Retention would materially expand privacy, security, support and integration scope.

## Decision

The API accepts no direct patient identifiers and stores no evaluation. Birth and collection timestamps remain in volatile client memory; only elapsed minutes are sent. Clinical request/response data is discarded after the response.

## Consequences

- The server has no clinical database or patient-recall feature.
- Users must re-enter an assessment after clearing or reloading.
- A non-identifying receipt can be printed but is not retained by Bili Mate.
- Health data is still sensitive while processed and requires DPIA/security controls.
