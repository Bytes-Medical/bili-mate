# Clinical validation artefacts

`cg98-scenarios.yaml` is the clinical scenario set required by
[`spec/09-test-and-validation.md`](../spec/09-test-and-validation.md): at
least 60 scenarios across every distribution category, each recording
inputs, the expected primary and supporting actions and NICE references.

## Governance

- The harness (`cargo run -p bili-mate-scenarios`) runs every scenario
  against the engine and fails CI on any divergence, so the engine and the
  signed expectations cannot drift apart silently.
- `bili-scenarios --export review.md` produces the clinical review
  document with reviewer identity, outcome and discrepancy-disposition
  fields. The export embeds the scenario set's SHA-256, so an approval
  signs exactly one content state; any edit to this file changes the digest
  and invalidates prior signatures.
- Two clinical reviewers, independent of the rule transcription, must
  approve the set before Stage 3 exit (spec 08). Their signed export is
  attached to the release evidence bundle
  (`scripts/release-evidence.sh`).
- A scenario change after approval follows change control: new digest, new
  review, and a return to the earliest affected validation stage (OPS-017).

Engineering-authored expectations in this set are drawn from
`spec/02-clinical-rule-engine.md` and are not clinical approval; the
reviewer signatures are.
