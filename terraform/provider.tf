terraform {
  backend "s3" {}

  required_providers {
    archive = {
      source  = "registry.terraform.io/hashicorp/archive"
      version = "~> 2.4"
    }

    aws = {
      source  = "registry.terraform.io/hashicorp/aws"
      version = "~> 5.29"
    }

    random = {
      source  = "registry.terraform.io/hashicorp/random"
      version = "~> 3.6"
    }
  }
}

provider "aws" {
  region = var.region
  default_tags {
    tags = {
      Name      = "lambda-nop-testing"
      Project   = "lambda-nop-testing"
      Terraform = "https://github.com/dacut/lambda-nop/terraform"
    }
  }
}
