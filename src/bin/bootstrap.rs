//! An AWS lambda function that returns a blank response.
//! This is useful for bootstrapping Terraform/IaC systems and not much else.

#![deny(clippy::all)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(missing_docs)]

use {
    lambda_runtime::{run, service_fn, LambdaEvent},
    serde::Serialize,
    serde_json::{Map, Value},
};

const BLANK_HTML: &str = "<html><head></head><body></body></html>";
const BLANK_JSON: &str = "{}";
const BLANK_TEXT: &str = "";

const APPLICATION_JSON: &str = "application/json";
const TEXT_HTML: &str = "text/html";
const TEXT_PLAIN: &str = "text/plain";

const CONTENT_TYPE: &str = "content-type";

const BODY: &str = "body";
const HEADERS: &str = "headers";
const IS_BASE64_ENCODED: &str = "isBase64Encoded";
const STATUS_CODE: &str = "statusCode";

const FAILED: &str = "FAILED";
const NOT_IMPLEMENTED: &str = "Not implemented";

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Identifies whether we've gotten an LambdaFunctionUrlRequestContext
pub(crate) fn is_lambda_function_url_request_context(value: Option<&Value>) -> bool {
    let Some(value) = value else {
        return false;
    };

    let Some(value) = value.as_object() else {
        return false;
    };

    value.get("http").is_some()
}

/// Idenfities whether we've gotten valid HTTP headers
pub(crate) fn is_valid_http_headers(value: Option<&Value>) -> bool {
    let Some(value) = value else {
        return false;
    };

    value.is_object()
}

/// Identifies whether we've gotten a valid HTTP method
pub(crate) fn is_valid_http_method(value: Option<&Value>) -> bool {
    let Some(value) = value else {
        return false;
    };

    value.is_string()
}

/// The main handler function.
async fn function_handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let Value::Object(event) = event.payload else {
        // Can't decode this; just return success.
        return Ok(Value::Object(Map::new()));
    };

    // Is this an HTTP request, via ALB, API Gateway, or Lambda Function URL?

    // httpMethod is in AlbTargetGroupRequest, ApiGatewayProxyRequest, and ApiGatewayV2httpRequest
    let http_method = event.get("httpMethod");

    // headers is in AlbTargetGroupRequest, ApiGatewayProxyRequest, ApiGatewayV2httpRequest, and LambdaFunctionUrlRequest
    let headers = event.get("headers");

    // requestContext is in AlbTargetGroupRequest, ApiGatewayProxyRequest, ApiGatewayV2httpRequest, and LambdaFunctionUrlRequest
    // We only care about it for LambdaFunctionUrlRequest to positively identify that we're getting a LambdaFunctionUrlRequest
    // since it doesn't transmit httpMethod.
    let request_context = event.get("requestContext");

    if is_valid_http_headers(headers)
        && (is_valid_http_method(http_method) || is_lambda_function_url_request_context(request_context))
    {
        let headers = headers.unwrap().as_object().unwrap();
        return handle_http_request(headers);
    }

    // These are all expected in CloudFormation events
    let request_type = event.get("RequestType");
    let request_id = event.get("RequestId");
    let response_url = event.get("ResponseURL");
    let stack_id = event.get("StackId");
    let resource_type = event.get("ResourceType");
    let logical_resource_id = event.get("LogicalResourceId");
    let physical_resource_id = event.get("PhysicalResourceId");

    if request_type.is_some()
        && request_type.unwrap().is_string()
        && request_id.is_some()
        && request_id.unwrap().is_string()
        && response_url.is_some()
        && response_url.unwrap().is_string()
        && stack_id.is_some()
        && stack_id.unwrap().is_string()
        && resource_type.is_some()
        && resource_type.unwrap().is_string()
        && logical_resource_id.is_some()
        && logical_resource_id.unwrap().is_string()
    {
        return handle_cloudformation_request(
            request_id.unwrap().as_str().unwrap(),
            response_url.unwrap().as_str().unwrap(),
            stack_id.unwrap().as_str().unwrap(),
            logical_resource_id.unwrap().as_str().unwrap(),
            physical_resource_id,
        )
        .await;
    }

    // Unknown event type. Just return an empty result.
    Ok(Value::Object(Map::new()))
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(function_handler)).await
}

/// Common handling for all HTTP-style requests
fn handle_http_request(headers: &Map<String, Value>) -> Result<Value, Error> {
    let mut message = BLANK_HTML;
    let mut response_content_type = TEXT_HTML;

    // See what the client accepts.
    if let Some(accept) = headers.get("accept") {
        if let Some(accept) = accept.as_str() {
            for part in accept.split(',') {
                let part = part.trim();
                let content_type = part.split(';').next().unwrap_or("");
                if content_type == "application/json" {
                    message = BLANK_JSON;
                    response_content_type = APPLICATION_JSON;
                    break;
                }
                if content_type == "text/html" {
                    message = BLANK_HTML;
                    response_content_type = TEXT_HTML;
                    break;
                }
                if content_type == "text/plain" {
                    message = BLANK_TEXT;
                    response_content_type = TEXT_PLAIN;
                    break;
                }
            }
        }
    }

    let mut response = Map::new();
    let mut resp_headers = Map::new();
    resp_headers.insert(CONTENT_TYPE.to_string(), response_content_type.into());

    response.insert(STATUS_CODE.to_string(), 200.into());
    response.insert(BODY.to_string(), message.into());
    response.insert(IS_BASE64_ENCODED.to_string(), false.into());
    response.insert(HEADERS.to_string(), resp_headers.into());

    Ok(response.into())
}

/// Handles CloudFormation create, update, and delete requests.
async fn handle_cloudformation_request(
    request_id: &str,
    response_url: &str,
    stack_id: &str,
    logical_resource_id: &str,
    physical_resource_id: Option<&Value>,
) -> Result<Value, Error> {
    let physical_response_id = match physical_resource_id {
        Some(value) => value.as_str().map(|value| value.to_string()),
        None => None,
    };

    let response = CloudFormationResponse {
        status: FAILED.to_string(),
        reason: NOT_IMPLEMENTED.to_string(),
        physical_resource_id: physical_response_id,
        stack_id: stack_id.to_string(),
        request_id: request_id.to_string(),
        logical_resource_id: logical_resource_id.to_string(),
    };
    let body = serde_json::to_string(&response)?;

    let client = reqwest::Client::new();
    client.put(response_url).body(body).send().await?;
    Ok(Value::Object(Map::new()))
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct CloudFormationResponse {
    status: String,
    reason: String,
    physical_resource_id: Option<String>,
    stack_id: String,
    request_id: String,
    logical_resource_id: String,
}

#[cfg(test)]
mod test {
    use {
        super::*,
        aws_lambda_events::{
            alb::{AlbTargetGroupRequest, AlbTargetGroupRequestContext, AlbTargetGroupResponse, ElbContext},
            apigw::{
                ApiGatewayProxyRequest, ApiGatewayProxyRequestContext, ApiGatewayProxyResponse,
                ApiGatewayRequestIdentity,
            },
            encodings::Body,
            http::{HeaderMap, HeaderValue, Method},
            query_map::QueryMap,
        },
        lambda_runtime::Context,
        std::collections::HashMap,
    };

    #[tokio::test]
    async fn test_handle_alb_json_request() {
        let mut headers = HeaderMap::new();
        headers.insert("accept", HeaderValue::from_static("application/json"));

        let request_context = AlbTargetGroupRequestContext {
            elb: ElbContext {
                target_group_arn: Some(
                    "arn:aws:elasticloadbalancing:us-east-1:123456789012:targetgroup/lambda-123456789/1234567890123456"
                        .to_string(),
                ),
            },
        };

        let payload = AlbTargetGroupRequest {
            http_method: Method::GET,
            path: Some("/".to_string()),
            query_string_parameters: QueryMap::default(),
            multi_value_query_string_parameters: QueryMap::default(),
            headers: headers.clone(),
            multi_value_headers: headers,
            request_context,
            is_base64_encoded: false,
            body: None,
        };

        let context = Context::default();
        let payload_json = serde_json::to_value(&payload).unwrap();
        let event = LambdaEvent::new(payload_json, context);
        let result = function_handler(event).await.unwrap();
        let response: AlbTargetGroupResponse = serde_json::from_value(result).unwrap();
        let status_code = response.status_code;
        let body = response.body.unwrap();

        assert_eq!(status_code, 200);
        assert_eq!(body, Body::Text("{}".to_string()));
    }

    #[tokio::test]
    async fn test_handle_alb_html_request() {
        let mut headers = HeaderMap::new();
        headers.insert("accept", HeaderValue::from_static("text/html, application/xhtml+xml"));

        let request_context = AlbTargetGroupRequestContext {
            elb: ElbContext {
                target_group_arn: Some(
                    "arn:aws:elasticloadbalancing:us-east-1:123456789012:targetgroup/lambda-123456789/1234567890123456"
                        .to_string(),
                ),
            },
        };

        let payload = AlbTargetGroupRequest {
            http_method: Method::GET,
            path: Some("/".to_string()),
            query_string_parameters: QueryMap::default(),
            multi_value_query_string_parameters: QueryMap::default(),
            headers: headers.clone(),
            multi_value_headers: headers,
            request_context,
            is_base64_encoded: false,
            body: None,
        };

        let context = Context::default();
        let payload_json = serde_json::to_value(&payload).unwrap();
        let event = LambdaEvent::new(payload_json, context);
        let result = function_handler(event).await.unwrap();
        let response: AlbTargetGroupResponse = serde_json::from_value(result).unwrap();
        let status_code = response.status_code;
        let body = response.body.unwrap();

        assert_eq!(status_code, 200);
        assert_eq!(body, Body::Text("<html><head></head><body></body></html>".to_string()));
    }

    #[tokio::test]
    async fn test_handle_alb_text_request() {
        let mut headers = HeaderMap::new();
        headers.insert("accept", HeaderValue::from_static("text/plain"));

        let request_context = AlbTargetGroupRequestContext {
            elb: ElbContext {
                target_group_arn: Some(
                    "arn:aws:elasticloadbalancing:us-east-1:123456789012:targetgroup/lambda-123456789/1234567890123456"
                        .to_string(),
                ),
            },
        };

        let payload = AlbTargetGroupRequest {
            http_method: Method::GET,
            path: Some("/".to_string()),
            query_string_parameters: QueryMap::default(),
            multi_value_query_string_parameters: QueryMap::default(),
            headers: headers.clone(),
            multi_value_headers: headers,
            request_context,
            is_base64_encoded: false,
            body: None,
        };

        let context = Context::default();
        let payload_json = serde_json::to_value(&payload).unwrap();
        let event = LambdaEvent::new(payload_json, context);
        let result = function_handler(event).await.unwrap();
        let response: AlbTargetGroupResponse = serde_json::from_value(result).unwrap();
        let status_code = response.status_code;
        let body = response.body.unwrap();

        assert_eq!(status_code, 200);
        assert_eq!(body, Body::Text("".to_string()));
    }

    #[tokio::test]
    async fn test_handle_apigw_proxy_json_request() {
        let mut headers = HeaderMap::new();
        headers.insert("accept", HeaderValue::from_static("application/json"));

        let identity = ApiGatewayRequestIdentity::default();

        let request_context = ApiGatewayProxyRequestContext {
            account_id: Some("123456789012".to_string()),
            resource_id: Some("pqowifjqpwoeifj".to_string()),
            operation_name: None,
            stage: Some("prod".to_string()),
            domain_name: Some("api.example.com".to_string()),
            domain_prefix: Some("api".to_string()),
            request_id: Some("12345678-1234-1234-1234-123456789012".to_string()),
            protocol: Some("https".to_string()),
            identity,
            resource_path: Some("/".to_string()),
            path: Some("/".to_string()),
            authorizer: HashMap::<String, Value>::default(),
            http_method: Method::GET,
            request_time: None,
            request_time_epoch: 1,
            apiid: Some("1234567890".to_string()),
        };

        let payload = ApiGatewayProxyRequest {
            resource: Some("/".to_string()),
            path: Some("/".to_string()),
            http_method: Method::GET,
            headers: headers.clone(),
            multi_value_headers: headers,
            query_string_parameters: QueryMap::default(),
            multi_value_query_string_parameters: QueryMap::default(),
            path_parameters: HashMap::default(),
            stage_variables: HashMap::default(),
            request_context,
            is_base64_encoded: false,
            body: None,
        };

        let context = Context::default();
        let payload_json = serde_json::to_value(&payload).unwrap();
        let event = LambdaEvent::new(payload_json, context);
        let result = function_handler(event).await.unwrap();
        let response: ApiGatewayProxyResponse = serde_json::from_value(result).unwrap();
        let status_code = response.status_code;
        let body = response.body.unwrap();

        assert_eq!(status_code, 200);
        assert_eq!(body, Body::Text("{}".to_string()));
    }
}
