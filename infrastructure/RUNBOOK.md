# Operator runbook (OPS-014)

Covers the required procedures: outage, rule-pack integrity, suspected
incorrect result, data leakage, certificate failure, WAF abuse, and
rollback. In every clinical-facing incident the safe state is the same:
**clients fail closed and clinicians use the locally approved procedure** —
never keep a doubtful instance serving.

Severity contacts: critical alarms page the operator **and** the
Clinical Safety Officer (spec 10 alarm table). Record every action in the
incident log with times.

## 1. Outage — no ready tasks

Alarm: `no-ready-tasks` (critical).

1. Confirm scope: `GET /health/live` vs `GET /health/ready` on a task.
   Live-but-not-ready means an integrity or authorisation self-check failed
   — treat as §2, not capacity.
2. Check ECS service events and the deployment circuit breaker; a bad
   deployment auto-rolls back.
3. Verify the service status page reflects the outage (OPS-010) and that
   the web client shows the fail-closed screen (it must never show a
   previous result).
4. Restore capacity (scale, redeploy last authorised release) or execute §7.
5. Post-incident: root cause, affected releases, prevention tests.

## 2. Rule-pack integrity failure

Signal: readiness false with startup log `rule pack integrity verification
failed`, or the release-manifest digest does not match `content_sha256`
from `/v1/guidelines/active` (SEC-014).

1. Do NOT restart-loop the task; capture the task's log stream first.
2. Treat as a clinical-integrity incident (spec 07 incident process):
   suspend clinical mode (§6 of spec 07) and notify the CSO immediately.
3. Compare the running image digest with the release manifest. Any mismatch
   is a supply-chain incident: preserve evidence, rotate deploy credentials.
4. Roll back (§7) to the last jointly authorised image + rule-pack pairing.
5. The pack and engine are validated as a pair — never mix versions without
   a fresh release authorisation.

## 3. Suspected incorrect clinical result

Report route: clinical-safety contact (heightened during pilot).

1. Within one working day, reproduce with the exact normalised inputs from
   the clinician's printed receipt (inputs are not logged — the receipt is
   the record; PRD-017).
2. Run the same inputs through `bili-eval` at the released commit; compare
   the receipt digest. A digest match with a clinically wrong answer is a
   rule/spec defect; a mismatch is an integrity incident (§2).
3. If the output may be unsafe: suspend clinical mode immediately
   (`operating_mode = "demonstration"` — OPS-011, no code deploy), then
   assess before any restore.
4. CSO dispositions the discrepancy; feed the case into the clinical
   scenario suite before the fix ships (OPS-017: return to the earliest
   affected validation stage).

## 4. Data leakage — clinical content in telemetry

Signal: sentinel test failure in CI, or any clinical value seen in logs,
metrics, traces or error reporting.

1. Treat as a health-data confidentiality incident. Stop the leaking
   pipeline first (disable the exporter/subscription), not the service.
2. Identify the leaked fields and window; purge affected log groups within
   retention policy and record the legal-hold decision (SEC-018).
3. Notify the security/privacy lead; assess UK GDPR reporting obligations.
4. Fix, then prove absence again with the sentinel suites
   (`apps/api/tests/privacy.rs`, `apps/api/tests/full_stack.rs`) plus a
   staging run before restoring the pipeline.

## 5. Certificate failure

Alarm: `api-cert-expiry` at 30 days (spec: escalate at 30/14/7).

1. ACM renews DNS-validated certificates automatically; a stuck renewal
   almost always means the validation CNAME was removed — restore it from
   the Terraform outputs.
2. If the API certificate has already expired: clients fail closed (this is
   the designed behaviour, PRD-013); fix the certificate rather than
   downgrading TLS (SEC-001). Publish the outage on the status page.

## 6. WAF abuse / rate anomaly

Alarm: `waf-blocks` (security).

1. Inspect sampled WAF metrics per rule: distinguish scraping (rate rule),
   out-of-UK access (geo rule) and exploit probing (managed rules).
2. For sustained abuse from specific sources, add a temporary IP-set block
   rule via Terraform change (OPS-007: reviewed, version-controlled).
3. Do not raise application rate limits in response to abuse.
4. Security log access is restricted and expires at 30 days (SEC-018).

## 7. Rollback

Trigger: clinical discrepancy or integrity concern, safety/security
incident, readiness or panic regression, sustained severe error/latency
regression, failed manifest verification.

Targets: begin within 15 minutes of the decision; restored within 30.

1. Identify the rollback target from the release log: the previous **jointly
   authorised image digest + rule pack** (they ship as one artifact — the
   pack is embedded in the binary).
2. `terraform apply -var api_image_digest=<previous digest>` (reviewed by a
   second operator; pre-approved change class for emergencies).
3. Watch readiness self-checks and the alarm table until stable; readiness
   proves the embedded pack verified.
4. If integrity is still uncertain after rollback: set
   `operating_mode = "demonstration"` (OPS-012 keeps the draft-pack
   labelling honest) or scale to zero and rely on the fail-closed client.
5. Record the rollback in the release log with cause and approvals.

## Canary deployment (spec 10 process, steps 6–10)

1. Deploy one canary task with no production traffic: a second ECS service
   with `desired_count = 1` on the same target group **weighted 0** via a
   separate target group and ALB weighted forward; run startup and smoke
   checks against its task IP.
2. Shift 5% of traffic (ALB listener rule weight) for at least 30 minutes;
   watch non-clinical metrics only (5xx, latency, readiness).
3. Increase to 50%, then 100%, with an explicit operator approval recorded
   at each stage.
4. Keep the previous image and rule pack as the immediate rollback target.
5. Record completion in the release log.
