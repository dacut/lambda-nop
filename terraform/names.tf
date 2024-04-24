resource "random_string" "suffix" {
  length  = 12
  lower   = true
  upper   = false
  numeric = true
  special = false
}

locals {
  suffix = random_string.suffix.result

  availability_zones = slice(sort(tolist(data.aws_availability_zones.available.names)), 0, 2)
  alb_name           = "lambda-nop-${local.suffix}"
  function_name      = "lambda-nop-${local.suffix}"
  role_name          = "lambda-nop-${local.suffix}"
  log_group_name     = "/aws/lambda/${local.function_name}"
  account_id         = data.aws_caller_identity.current.account_id
}

data "aws_caller_identity" "current" {}
data "aws_availability_zones" "available" {
  state = "available"
  filter {
    name   = "opt-in-status"
    values = ["opt-in-not-required"]
  }
}
