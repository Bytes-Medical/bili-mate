variable "name_prefix" {
  description = "Resource name prefix"
  type        = string
  default     = "bili-mate"
}

variable "api_domain" {
  description = "Public API domain (e.g. api.bili-mate.uk)"
  type        = string
}

variable "web_domain" {
  description = "Public web domain (e.g. bili-mate.uk)"
  type        = string
}

variable "hosted_zone_id" {
  description = "Route 53 hosted zone for the domains"
  type        = string
}

variable "api_image_digest" {
  description = "Fully qualified API image reference BY DIGEST (OPS-008), e.g. <ecr>/bili-mate-api@sha256:…"
  type        = string
}

variable "release_authorisation_ref" {
  description = "Release-authorisation reference validated by the service at startup (SAFE-024). Empty keeps clinical mode unavailable."
  type        = string
  default     = ""
}

variable "operating_mode" {
  description = "Service operating mode: demonstration or clinical (OPS-011: switching off clinical mode is a variable change, not a code deployment)"
  type        = string
  default     = "demonstration"

  validation {
    condition     = contains(["demonstration", "clinical"], var.operating_mode)
    error_message = "operating_mode must be demonstration or clinical."
  }
}

variable "web_csp" {
  description = "Content-Security-Policy for the static site, from web/out/csp.txt"
  type        = string
}

variable "vpc_cidr" {
  type    = string
  default = "10.40.0.0/16"
}

variable "alert_email" {
  description = "Operational alert subscription endpoint"
  type        = string
}

variable "task_cpu" {
  type    = number
  default = 512
}

variable "task_memory" {
  type    = number
  default = 1024
}
