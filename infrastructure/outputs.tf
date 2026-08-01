output "web_bucket" {
  description = "Upload target for the static export (web/out/)"
  value       = aws_s3_bucket.web.id
}

output "web_distribution_id" {
  description = "CloudFront distribution to invalidate after a web deploy"
  value       = aws_cloudfront_distribution.web.id
}

output "api_ecr_repository" {
  description = "Push target for signed API images (immutable tags)"
  value       = aws_ecr_repository.api.repository_url
}

output "api_endpoint" {
  value = "https://${var.api_domain}"
}

output "web_endpoint" {
  value = "https://${var.web_domain}"
}

output "api_certificate_validation_records" {
  description = "DNS validation records to create for the API certificate"
  value       = aws_acm_certificate.api.domain_validation_options
}

output "web_certificate_validation_records" {
  description = "DNS validation records to create for the web certificate"
  value       = aws_acm_certificate.web.domain_validation_options
}
