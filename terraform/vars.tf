variable "alb_vpc_cidr_block" {
  type        = string
  description = "The CIDR block for the ALB VPC"
  default     = "172.16.0.0/16"
}

variable "region" {
  type        = string
  description = "The AWS region to deploy to"
  default     = "us-west-2"
}
