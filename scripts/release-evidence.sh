#!/usr/bin/env bash
# Release-evidence bundle (spec 11 release checklist support): gathers the
# automated evidence for a release candidate into one hashed directory so
# the named approvers sign a specific, reproducible state.
#
# Usage: scripts/release-evidence.sh [output-dir]
set -euo pipefail

cd "$(dirname "$0")/.."
OUT="${1:-release-evidence/$(git rev-parse --short HEAD)}"
mkdir -p "$OUT"

echo "collecting release evidence into $OUT"

# Identity of the exact candidate.
{
  echo "commit:        $(git rev-parse HEAD)"
  echo "describe:      $(git describe --tags --always --dirty)"
  echo "engine:        $(grep -m1 '^version' Cargo.toml | cut -d'\"' -f2)"
  echo "api_version:   1.0.0-draft"
  echo "rule_pack:     nice-cg98-2023-10-31.1"
  echo "rule_pack_sha: $(shasum -a 256 spec/clinical/nice-cg98-2023-10-31.1.yaml | cut -d' ' -f1)"
  echo "generated_at:  $(date -u +%Y-%m-%dT%H:%M:%SZ)"
} > "$OUT/identity.txt"

# Automated verification evidence.
ruby spec/validate.rb            > "$OUT/spec-validation.txt"
ruby scripts/traceability-check.rb > "$OUT/traceability.txt"
cargo test --workspace 2>&1 | grep -E "^(running|test result)" > "$OUT/cargo-tests.txt"
cargo run -q -p guideline-data --bin pack_tool -- verify \
  spec/clinical/nice-cg98-2023-10-31.1.yaml > "$OUT/rule-pack-verify.txt"

# Clinical scenario evidence: the machine report plus the reviewable
# document that carries the scenario-set digest.
cargo run -q -p bili-mate-scenarios -- --export "$OUT/clinical-scenarios-review.md" \
  > "$OUT/clinical-scenarios-run.txt"

# Bundle manifest: hash of every artefact.
( cd "$OUT" && shasum -a 256 -- * > MANIFEST.sha256 )

echo
echo "bundle contents:"
ls -l "$OUT"
echo
echo "Attach this directory to the release record with the named approvals"
echo "(engineering, clinical, CSO, security, operations - spec 11 checklist)."
