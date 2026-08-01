# ADR-010: Monochrome (black and white) design system

Status: Accepted  
Date: 2026-08-01

## Context

PRD-032 and WEB-012 prohibit communicating clinical state through colour alone, and TEST-025 requires emergency presentation to remain understandable in monochrome. Colour-based severity schemes need a parallel non-colour encoding anyway.

## Decision

The reference web client uses a strict monochrome design system: black, white and a small grayscale ramp only. Clinical priority is encoded through typography scale and weight, heavy banners with inverted text for emergency and immediate states, borders, iconography and explicit priority text from the server response. The threshold chart distinguishes lines by dash pattern (phototherapy solid, exchange dashed) and measurements by marker shape/fill (serum filled, transcutaneous hollow).

## Consequences

- The accessible non-colour encoding is the primary design, not a fallback; TEST-025 is satisfied by construction.
- Contrast requirements of WCAG 2.2 AA are trivially exceeded; print output matches screen semantics.
- Future native clients inherit the same encoding rules, since reference presentation is normative for them.
