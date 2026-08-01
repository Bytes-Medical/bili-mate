# Bili Mate specification

Status: **Draft for engineering and clinical review**  
Specification version: `0.1.0`  
Target clinical rule pack: `nice-cg98-2023-10-31.1`  
Last reviewed: 2026-08-01

This directory is the normative specification for Bili Mate: a UK-only neonatal jaundice clinical decision-support server, a reference web client, and future Android and iOS clients. The first implementation is a deterministic Rust service based on NICE guideline CG98.

The specification does not authorise clinical use. A build can be technically complete while remaining demonstration-only. Clinical use requires the release gates in [clinical safety and regulation](08-clinical-safety-and-regulation.md) and [delivery and rollout](11-delivery-and-rollout.md).

## Reading order

1. [Product requirements](01-product-requirements.md)
2. [Clinical rule engine](02-clinical-rule-engine.md)
3. [Domain model](03-domain-model.md)
4. [API contract](04-api-contract.md) and [OpenAPI definition](openapi.yaml)
5. [System architecture](05-system-architecture.md)
6. [Reference web application](06-reference-web.md)
7. [Security and privacy](07-security-and-privacy.md)
8. [Clinical safety and regulation](08-clinical-safety-and-regulation.md)
9. [Test and validation](09-test-and-validation.md)
10. [Deployment and operations](10-deployment-and-operations.md)
11. [Delivery and rollout](11-delivery-and-rollout.md)
12. [Traceability matrix](traceability.md)

Architectural decisions are recorded under [`decisions/`](decisions/README.md). Machine-readable clinical rules are specified in [`clinical/nice-cg98-2023-10-31.1.yaml`](clinical/nice-cg98-2023-10-31.1.yaml). Examples under [`examples/`](examples/) are informative instances of the normative OpenAPI contract.

Run `ruby spec/validate.rb` from the repository root to check YAML and JSON syntax, OpenAPI references, example/schema conformance, Markdown links, requirement-number continuity, and clinical-rule traceability. The validator uses only the Ruby standard library.

## Normative language

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, and **MAY** are normative:

- **MUST/MUST NOT**: required for conformance.
- **SHOULD/SHOULD NOT**: expected unless a documented, reviewed exception exists.
- **MAY**: optional and must not change clinical meaning.

Normative requirements have stable identifiers:

| Prefix | Area |
|---|---|
| `PRD` | Product and scope |
| `CLIN` | Clinical rules |
| `DATA` | Domain model and validation |
| `API` | HTTP interface |
| `WEB` | Reference web client |
| `SEC` | Security and privacy |
| `SAFE` | Clinical safety and regulatory controls |
| `OPS` | Deployment and operation |
| `TEST` | Verification and validation |

The Markdown documents are normative unless a section is labelled informative. The OpenAPI file is normative for HTTP wire shapes. The clinical YAML is normative for rule-pack data. If two normative sources conflict, implementation MUST stop and the conflict MUST be resolved through change control before release.

## Source hierarchy

Clinical rules use the following precedence:

1. Current published recommendations in [NICE CG98](https://www.nice.org.uk/guidance/cg98/chapter/recommendations).
2. The official [NICE treatment threshold graphs](https://www.nice.org.uk/guidance/cg98/resources/treatment-threshold-graphs-excel-544300525).
3. The published CG98 full guideline only to explain the construction of treatment curves; it does not override current recommendations.
4. Written interpretation approved by the Bili Mate clinical owner and Clinical Safety Officer when NICE is silent or ambiguous.

Clinical Knowledge Summaries are third-party content. Bili Mate MUST NOT copy or republish CKS wording unless a separate licence is obtained. The implementation is based on NICE CG98, not the linked CKS page.

The product MUST include the attribution and disclaimer required by the [NICE UK Open Content Licence](https://www.nice.org.uk/reusing-our-content/nice-uk-open-content-licence). Third-party rights in threshold resources MUST be confirmed before any clinical pilot.

## Fixed project decisions

- Intended users: registered UK healthcare professionals.
- Geography: United Kingdom only.
- Release objective: controlled clinical pilot readiness, not immediate market release.
- Clinical scope: the full supported CG98 assessment and management pathway.
- Patient data: no retained patient record and no patient identifier fields.
- Connectivity: server-authoritative; clinical calculation fails closed offline.
- Access: public API without clinician login, protected by rate limiting and professional-use labelling.
- First client: reference web application; native production clients follow through generated SDKs.
- Technology: stable Rust, deterministic exact arithmetic, Axum HTTP service.
- Clinical AI: none. No camera, image, skin-colour, prediction, statistical model, or generative model is used.

## Glossary

| Term | Meaning |
|---|---|
| Assessment age | Elapsed time since birth at the point being evaluated, in whole minutes |
| Completed gestational weeks | Whole weeks completed at birth; days beyond the completed week are not rounded upward |
| Decision receipt | A response object that records inputs in normalised non-identifying form, engine version, rule pack, thresholds, and recommendations |
| Rule pack | Immutable, versioned clinical facts and decision rules used by the engine |
| TSB | Total serum bilirubin |
| TcB | Transcutaneous bilirubin |
| Treatment line | Age- and gestation-specific bilirubin threshold for phototherapy or exchange transfusion |
| Local protocol | A deploying organisation's clinically approved rule outside or more specific than CG98 |

## Change control

Every change MUST:

1. identify affected requirement IDs;
2. update the traceability matrix;
3. state whether the API version, engine version, or rule-pack version changes;
4. receive engineering review;
5. receive clinical and safety review if clinical behaviour or presentation changes; and
6. retain previous signed rule packs for audit and rollback.

No document may contain unresolved `TBD`, `TODO`, or “decide later” markers when marked approved for implementation.
