resource "aws_iam_role" "lambda_nop" {
  name                  = local.role_name
  description           = "Lambda NOP testing"
  force_detach_policies = true

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "lambda.amazonaws.com"
        }
      }
    ]
  })
}

resource "aws_cloudwatch_log_group" "lambda_nop" {
  name              = local.log_group_name
  retention_in_days = 1
}

resource "aws_iam_role_policy" "lambda_nop" {
  name = "Logging"
  role = aws_iam_role.lambda_nop.id
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = [
          "logs:CreateLogStream",
          "logs:PutLogEvents"
        ]
        Effect = "Allow"
        Resource = [
          "arn:aws:logs:us-west-2:us-west-2:${local.account_id}:log-group:${local.log_group_name}:*",
          "arn:aws:logs:us-west-2:us-west-2:${local.account_id}:log-group:${local.log_group_name}:log-stream:*",
        ]
      }
    ]
  })
}

data "archive_file" "lambda_nop_aarch64" {
  type        = "zip"
  output_path = "${path.module}/../target/aarch64-unknown-linux-gnu/release/lambda-nop.zip"
  source_file = "${path.module}/../target/aarch64-unknown-linux-gnu/release/bootstrap"
}

resource "aws_lambda_function" "lambda_nop" {
  architectures    = ["arm64"]
  description      = "Lambda NOP testing"
  filename         = data.archive_file.lambda_nop_aarch64.output_path
  function_name    = local.function_name
  handler          = "lambda-nop"
  package_type     = "Zip"
  role             = aws_iam_role.lambda_nop.arn
  runtime          = "provided.al2"
  source_code_hash = filebase64sha256(data.archive_file.lambda_nop_aarch64.output_path)
  timeout          = 10

  depends_on = [aws_cloudwatch_log_group.lambda_nop]
}
