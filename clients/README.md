# Generated API clients

TypeScript, Swift and Kotlin clients are generated in CI from the committed
`spec/openapi.yaml` (PRD-010, TEST-017). Generated code is a build artifact
and is never committed; the committed contract is the single source.

- `typescript-fixture/` — a minimal consumer compiled in CI against the
  generated TypeScript client. It demonstrates metadata fetch, request
  construction, the stale-rule-pack refresh flow, fail-closed network
  handling and display of source/version fields (spec 05 native-client
  path).
- Swift and Kotlin clients are generated in CI to prove the contract is
  generatable; compiling consumer fixtures for both remains an M3 follow-up
  tracked in `IMPLEMENTATION_PLAN.md`.
