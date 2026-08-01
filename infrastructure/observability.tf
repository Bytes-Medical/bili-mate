# Metrics, allowlisted logs and the spec 10 alarm table. Application logs
# expire after 30 days (OPS-009); log content is allowlisted at the
# application layer and verified by the sentinel tests.

resource "aws_cloudwatch_log_group" "api" {
  name              = "/${var.name_prefix}/api"
  retention_in_days = 30
}

resource "aws_sns_topic" "critical" {
  name = "${var.name_prefix}-critical"
}

resource "aws_sns_topic" "operational" {
  name = "${var.name_prefix}-operational"
}

resource "aws_sns_topic_subscription" "critical_email" {
  topic_arn = aws_sns_topic.critical.arn
  protocol  = "email"
  endpoint  = var.alert_email
}

resource "aws_sns_topic_subscription" "operational_email" {
  topic_arn = aws_sns_topic.operational.arn
  protocol  = "email"
  endpoint  = var.alert_email
}

# Critical: no ready task — page the operator and the clinical-safety
# contact (spec 10 alarm table row 1).
resource "aws_cloudwatch_metric_alarm" "no_ready_tasks" {
  alarm_name          = "${var.name_prefix}-no-ready-tasks"
  alarm_description   = "CRITICAL: no ready API task. Clients fail closed; clinicians use local protocol."
  namespace           = "AWS/ApplicationELB"
  metric_name         = "HealthyHostCount"
  statistic           = "Minimum"
  period              = 60
  evaluation_periods  = 2
  threshold           = 1
  comparison_operator = "LessThanThreshold"
  treat_missing_data  = "breaching"

  dimensions = {
    TargetGroup  = aws_lb_target_group.api.arn_suffix
    LoadBalancer = aws_lb.api.arn_suffix
  }

  alarm_actions = [aws_sns_topic.critical.arn]
  ok_actions    = [aws_sns_topic.critical.arn]
}

# High: 5xx rate above 1% for 5 minutes (alarm table row 3).
resource "aws_cloudwatch_metric_alarm" "error_rate" {
  alarm_name          = "${var.name_prefix}-5xx-rate"
  alarm_description   = "HIGH: 5xx above 1% for 5 minutes. Investigate; consider rollback."
  evaluation_periods  = 5
  threshold           = 1
  comparison_operator = "GreaterThanThreshold"
  treat_missing_data  = "notBreaching"

  metric_query {
    id          = "error_rate"
    expression  = "100 * errors / MAX([requests, 1])"
    label       = "5xx percentage"
    return_data = true
  }

  metric_query {
    id = "errors"

    metric {
      namespace   = "AWS/ApplicationELB"
      metric_name = "HTTPCode_Target_5XX_Count"
      period      = 60
      stat        = "Sum"

      dimensions = {
        LoadBalancer = aws_lb.api.arn_suffix
      }
    }
  }

  metric_query {
    id = "requests"

    metric {
      namespace   = "AWS/ApplicationELB"
      metric_name = "RequestCount"
      period      = 60
      stat        = "Sum"

      dimensions = {
        LoadBalancer = aws_lb.api.arn_suffix
      }
    }
  }

  alarm_actions = [aws_sns_topic.critical.arn]
}

# Medium: p95 latency above 250 ms for 10 minutes (alarm table row 4).
resource "aws_cloudwatch_metric_alarm" "latency_p95" {
  alarm_name          = "${var.name_prefix}-latency-p95"
  alarm_description   = "MEDIUM: p95 above 250 ms for 10 minutes. Capacity investigation."
  namespace           = "AWS/ApplicationELB"
  metric_name         = "TargetResponseTime"
  extended_statistic  = "p95"
  period              = 60
  evaluation_periods  = 10
  threshold           = 0.25
  comparison_operator = "GreaterThanThreshold"
  treat_missing_data  = "notBreaching"

  dimensions = {
    LoadBalancer = aws_lb.api.arn_suffix
  }

  alarm_actions = [aws_sns_topic.operational.arn]
}

# Security: WAF block anomaly (alarm table row 6).
resource "aws_cloudwatch_metric_alarm" "waf_blocks" {
  alarm_name          = "${var.name_prefix}-waf-blocks"
  alarm_description   = "SECURITY: elevated WAF blocks on the API."
  namespace           = "AWS/WAFV2"
  metric_name         = "BlockedRequests"
  statistic           = "Sum"
  period              = 300
  evaluation_periods  = 1
  threshold           = 500
  comparison_operator = "GreaterThanThreshold"
  treat_missing_data  = "notBreaching"

  dimensions = {
    WebACL = aws_wafv2_web_acl.api.name
    Region = "eu-west-2"
    Rule   = "ALL"
  }

  alarm_actions = [aws_sns_topic.operational.arn]
}

# Certificate expiry warning (alarm table row 7): ACM emits DaysToExpiry.
resource "aws_cloudwatch_metric_alarm" "api_cert_expiry" {
  alarm_name          = "${var.name_prefix}-api-cert-expiry"
  alarm_description   = "OPERATIONAL: API certificate expires within 30 days."
  namespace           = "AWS/CertificateManager"
  metric_name         = "DaysToExpiry"
  statistic           = "Minimum"
  period              = 86400
  evaluation_periods  = 1
  threshold           = 30
  comparison_operator = "LessThanOrEqualToThreshold"
  treat_missing_data  = "notBreaching"

  dimensions = {
    CertificateArn = aws_acm_certificate.api.arn
  }

  alarm_actions = [aws_sns_topic.operational.arn]
}
