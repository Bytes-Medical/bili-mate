# WAF on the API path (SEC-005): UK allow policy, managed baseline rules,
# body-size limit and per-IP rate limit. The application enforces the same
# size and rate controls independently.

resource "aws_wafv2_web_acl" "api" {
  name  = "${var.name_prefix}-api"
  scope = "REGIONAL"

  default_action {
    allow {}
  }

  # UK-only access (SEC-003): block requests from outside GB at the edge.
  rule {
    name     = "uk-only"
    priority = 0

    action {
      block {}
    }

    statement {
      not_statement {
        statement {
          geo_match_statement {
            country_codes = ["GB"]
          }
        }
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "uk-only"
      sampled_requests_enabled   = false
    }
  }

  # Body size limit: 64 KiB (API-004), enforced before the application.
  rule {
    name     = "body-size"
    priority = 1

    action {
      block {}
    }

    statement {
      size_constraint_statement {
        comparison_operator = "GT"
        size                = 65536

        field_to_match {
          body {
            oversize_handling = "MATCH"
          }
        }

        text_transformation {
          priority = 0
          type     = "NONE"
        }
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "body-size"
      sampled_requests_enabled   = false
    }
  }

  # Per-IP rate limit (spec 07): evaluated over a 5-minute window.
  rule {
    name     = "rate-per-ip"
    priority = 2

    action {
      block {}
    }

    statement {
      rate_based_statement {
        limit              = 400
        aggregate_key_type = "IP"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "rate-per-ip"
      sampled_requests_enabled   = false
    }
  }

  rule {
    name     = "managed-common"
    priority = 3

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        vendor_name = "AWS"
        name        = "AWSManagedRulesCommonRuleSet"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "managed-common"
      sampled_requests_enabled   = false
    }
  }

  visibility_config {
    cloudwatch_metrics_enabled = true
    metric_name                = "${var.name_prefix}-api"
    sampled_requests_enabled   = false
  }
}

resource "aws_wafv2_web_acl_association" "api" {
  resource_arn = aws_lb.api.arn
  web_acl_arn  = aws_wafv2_web_acl.api.arn
}
