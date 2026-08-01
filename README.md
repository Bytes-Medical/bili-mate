# Bili Mate

Clinical decision support for registered UK healthcare professionals assessing jaundice in
newborn babies, based on NICE guideline CG98. A deterministic Rust API evaluates each
assessment against the embedded clinical rule pack; a Next.js reference web client provides
the professional workflow.

> **Demonstration only.** The embedded rule pack is `draft` status: the service runs in
> demonstration mode, labels every result "not for patient care", and refuses to start in
> clinical mode. Clinical use requires the release gates in
> [`spec/08-clinical-safety-and-regulation.md`](spec/08-clinical-safety-and-regulation.md).

## Repository layout

| Path | Contents |
|---|---|
| `spec/` | Normative specification (requirements, OpenAPI contract, clinical rule pack) |
| `crates/clinical-core` | Pure deterministic clinical engine (exact arithmetic, CG98 rules) |
| `crates/guideline-data` | Rule-pack loading, integrity self-tests, pack diff tool |
| `apps/api` | Axum HTTP service implementing `spec/openapi.yaml` |
| `apps/cli` | `bili-eval`, an engineering-only fixture evaluator |
| `web/` | Next.js reference web client (static export) |
| `clients/` | Generated API client fixtures (CI) |
| `IMPLEMENTATION_PLAN.md` | Milestones and current progress |

## Prerequisites

- **Rust** 1.90 (pinned in `rust-toolchain.toml`; `rustup` picks it up automatically)
- **Node.js** 24+ and npm (web client)
- **Ruby** (standard library only, for `spec/validate.rb`)

## Quick start

Run the API and the web client in two terminals.

**Terminal 1 — API** (defaults: demonstration mode on `127.0.0.1:8080`):

```sh
BILI_MATE_ALLOWED_ORIGINS=http://localhost:3000 cargo run -p bili-mate-api
```

`BILI_MATE_ALLOWED_ORIGINS` is the CORS allowlist; it must include the web client's origin
or the browser will refuse to call the API. Check it is up:

```sh
curl http://127.0.0.1:8080/health/ready
```

**Terminal 2 — web client:**

```sh
cd web
npm install
NEXT_PUBLIC_API_BASE_URL=http://127.0.0.1:8080 npm run dev
```

Open <http://localhost:3000>, confirm the professional-use acknowledgement, and start an
assessment.

### Production-style static export

The deployable artifact is a static export served from any static host:

```sh
cd web
NEXT_PUBLIC_API_BASE_URL=http://127.0.0.1:8080 npm run build   # writes out/ and out/csp.txt
npm start                                                      # serves out/ on :3000
```

The build also writes `out/csp.txt` — the Content-Security-Policy header (with hashes for
every inline script) that the static host must send.

### API configuration

| Variable | Default | Meaning |
|---|---|---|
| `BILI_MATE_BIND` | `127.0.0.1:8080` | Listen address |
| `BILI_MATE_MODE` | `demonstration` | `demonstration` or `clinical` (clinical refuses readiness without an `active` rule pack and a release authorisation) |
| `BILI_MATE_ALLOWED_ORIGINS` | *(empty)* | Comma-separated CORS origin allowlist; wildcard is rejected |
| `BILI_MATE_RELEASE_AUTHORISATION` | *(unset)* | Release-authorisation reference required for clinical mode |

### CLI evaluator (no server needed)

```sh
cargo run -p bili-mate-cli --bin bili-eval -- spec/examples/normal-below-threshold-request.json
```

## Tests

```sh
cargo test --workspace          # clinical core, rule pack, API contract tests
ruby spec/validate.rb           # specification self-validation
cd web
npx playwright install chromium firefox webkit   # first time only
NEXT_PUBLIC_API_BASE_URL=http://localhost:18099 npm run build
npx playwright test             # browser end-to-end (starts its own API and static server)
```

The Playwright suite builds nothing itself: build the export first (as above) so it tests the
real deployment artifact. It then starts the Rust API on `:18099` and serves `out/` on
`:3100` automatically.

## Documentation

Start with [`spec/README.md`](spec/README.md) for the normative specification and reading
order, and [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md) for milestone status.
