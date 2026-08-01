# Reference infrastructure (spec 10)

Terraform for the supported pilot topology in `eu-west-2` (London):

```
UK clinician browser ── CloudFront (UK geo restriction) ── private S3 web bucket
                    └── WAF ── public HTTPS ALB ── ECS Fargate ×2 AZs (private subnets)
                                                   └── CloudWatch metrics + allowlisted logs
Signed image in ECR (immutable tags, scan on push)
```

There is no database, cache, queue or clinical object store; tasks reach ECR,
S3 and CloudWatch through VPC endpoints, so the private subnets have **no
internet egress at all**.

## Usage

```sh
terraform init            # configure your own remote state backend first
terraform plan  -var-file=pilot.tfvars
terraform apply -var-file=pilot.tfvars   # reviewed by a second operator (OPS-007)
```

Deployments reference the API image **by digest** (OPS-008): set
`api_image_digest` from the release manifest. The web bucket receives the
static export from `web/out/`, and the CSP in `web/out/csp.txt` must be set
in the CloudFront response-headers policy (`web_csp` variable).

CI runs `terraform fmt -check` and `terraform validate`; `plan`/`apply` are
operator actions with reviewed state (this repository intentionally contains
no backend or credentials).

## Runtime hardening

The task definition sets a read-only root filesystem with a single writable
ephemeral volume at `/tmp`, a fixed unprivileged user (65532), no privileged
flags and dropped capabilities. The container image is
distroless and contains only the API binary, notices and CA certificates
(see the repository `Dockerfile`).

See [`RUNBOOK.md`](RUNBOOK.md) for operator procedures (OPS-014).
