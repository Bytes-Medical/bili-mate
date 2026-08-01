# Architecture decision records

All records are accepted for the v1 specification. Reversal requires a new superseding ADR, impact analysis and updates to affected requirements and traceability.

| ADR | Decision |
|---|---|
| [ADR-001](ADR-001-rust-axum-exact-core.md) | Rust/Axum with a pure exact-arithmetic clinical core |
| [ADR-002](ADR-002-server-authoritative.md) | Server-authoritative calculations; no offline clinical engine |
| [ADR-003](ADR-003-no-patient-persistence.md) | No patient identifiers or server-side clinical persistence |
| [ADR-004](ADR-004-public-no-login.md) | Public professional-use API without login in v1 |
| [ADR-005](ADR-005-immutable-rule-packs.md) | Immutable governed rule packs and no runtime scraping |
| [ADR-006](ADR-006-cg98-not-cks.md) | NICE CG98 is the clinical/content baseline, not CKS |
| [ADR-007](ADR-007-reference-web-first.md) | Reference web client before native production apps |
| [ADR-008](ADR-008-uk-only.md) | UK-only market, content distribution and hosting reference |
| [ADR-009](ADR-009-nextjs-static-export.md) | Next.js static export for the reference web client |
| [ADR-010](ADR-010-monochrome-design.md) | Monochrome (black and white) design system |
