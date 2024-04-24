resource "aws_lambda_function_url" "lambda_nop" {
  function_name      = aws_lambda_function.lambda_nop.function_name
  authorization_type = "NONE"
}

output "lambda_nop_function_url" {
  value = aws_lambda_function_url.lambda_nop.function_url
}
