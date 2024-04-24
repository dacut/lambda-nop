resource "aws_lb" "lambda_nop" {
  name               = local.alb_name
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.lambda_nop_alb.id]
  subnets            = [for subnet in aws_subnet.lambda_nop : subnet.id]
}

resource "aws_lb_listener" "lambda_nop" {
  load_balancer_arn = aws_lb.lambda_nop.arn
  port              = 80
  protocol          = "HTTP"

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.lambda_nop.arn
  }
}

resource "aws_lb_target_group" "lambda_nop" {
  name        = local.alb_name
  target_type = "lambda"
}

resource "aws_lambda_permission" "lambda_nop_alb" {
  function_name = aws_lambda_function.lambda_nop.function_name
  statement_id  = "AllowAlb"
  action        = "lambda:InvokeFunction"
  principal     = "elasticloadbalancing.amazonaws.com"
  source_arn    = aws_lb_target_group.lambda_nop.arn
}

resource "aws_lb_target_group_attachment" "lambda_nop" {
  target_group_arn = aws_lb_target_group.lambda_nop.arn
  target_id        = aws_lambda_function.lambda_nop.arn
  depends_on       = [aws_lambda_permission.lambda_nop_alb]
}

resource "aws_vpc" "lambda_nop" {
  cidr_block                       = var.alb_vpc_cidr_block
  assign_generated_ipv6_cidr_block = true
}

resource "aws_subnet" "lambda_nop" {
  count = length(local.availability_zones)

  assign_ipv6_address_on_creation = true
  availability_zone               = local.availability_zones[count.index]
  cidr_block                      = cidrsubnet(var.alb_vpc_cidr_block, 8, count.index)
  ipv6_cidr_block                 = cidrsubnet(aws_vpc.lambda_nop.ipv6_cidr_block, 8, count.index)
  map_public_ip_on_launch         = true
  vpc_id                          = aws_vpc.lambda_nop.id
}

resource "aws_internet_gateway" "lambda_nop" {
  vpc_id = aws_vpc.lambda_nop.id
}

resource "aws_default_route_table" "lambda_nop" {
  default_route_table_id = aws_vpc.lambda_nop.default_route_table_id
}

resource "aws_route" "lambda_nop_default_ipv4" {
  route_table_id         = aws_default_route_table.lambda_nop.id
  destination_cidr_block = "0.0.0.0/0"
  gateway_id             = aws_internet_gateway.lambda_nop.id
}

resource "aws_route" "lambda_nop_default_ipv6" {
  route_table_id              = aws_default_route_table.lambda_nop.id
  destination_ipv6_cidr_block = "::/0"
  gateway_id                  = aws_internet_gateway.lambda_nop.id
}

resource "aws_security_group" "lambda_nop_alb" {
  name        = local.alb_name
  description = "Lambda NOP ALB testing"
  vpc_id      = aws_vpc.lambda_nop.id
}

resource "aws_vpc_security_group_ingress_rule" "lambda_nop_alb_ipv4" {
  security_group_id = aws_security_group.lambda_nop_alb.id
  ip_protocol       = "tcp"
  cidr_ipv4         = "0.0.0.0/0"
  from_port         = 80
  to_port           = 80
}

resource "aws_vpc_security_group_ingress_rule" "lambda_nop_alb_ipv6" {
  security_group_id = aws_security_group.lambda_nop_alb.id
  ip_protocol       = "tcp"
  cidr_ipv6         = "::/0"
  from_port         = 80
  to_port           = 80
}

output "lambda_nop_alb" {
  value = "http://${aws_lb.lambda_nop.dns_name}"
}
