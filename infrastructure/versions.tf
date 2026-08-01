# Reference pilot topology (spec 10). State must live in an encrypted,
# locked remote backend with restricted operator roles (OPS-007); the
# backend block is intentionally left to the deploying operator.

terraform {
  required_version = ">= 1.7.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.80"
    }
  }
}

provider "aws" {
  region = "eu-west-2" # London: UK-only hosting reference (ADR-008)
}

# CloudFront certificates and web ACLs live in us-east-1.
provider "aws" {
  alias  = "us_east_1"
  region = "us-east-1"
}
