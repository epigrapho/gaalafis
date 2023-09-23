use axum::{
    body::HttpBody,
    extract::Json,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Serialize)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn new(text: &str) -> Self {
        Self {
            message: text.to_string(),
        }
    }
}

struct ErrorBuilder<'a> {
    status: &'a StatusCode,
    message: &'a Option<String>,
}

impl<'a> ErrorBuilder<'a> {
    fn new(status: &'a StatusCode, message: &'a Option<String>) -> Self {
        Self {
            status,
            message,
        }
    }

    fn error(&self, message: &str) -> Result<Response, Response> {
        Err((*self.status, Json(Error::new(message))).into_response())
    }

    fn custom_status_error(&self, status: StatusCode, message: &str) -> Result<Response, Response> {
        Err((status, Json(Error::new(message))).into_response())
    }

    fn with_message(&self, default_message: &str) -> Result<Response, Response> {
        self.custom_status_with_message(*self.status, default_message)
    }

    fn custom_status_with_message(
        &self,
        status: StatusCode,
        default_message: &str,
    ) -> Result<Response, Response> {
        Err((
            status,
            Json(Error::new(match self.message {
                Some(message) => message,
                None => default_message,
            })),
        )
            .into_response())
    }
}

pub async fn handle_and_filter_error_details<B>(
    req: Request<B>,
    next: Next<B>,
) -> impl IntoResponse {
    let resp = next.run(req).await;
    let status = resp.status();

    if status.is_success() {
        return Ok(resp);
    }

    let inner_error_message = match resp.into_body().data().await {
        Some(Ok(data)) => match String::from_utf8(data.to_vec()) {
            Ok(s) => Some(s),
            Err(_) => None,
        },
        _ => None,
    };
    let error_message = match &inner_error_message {
        Some(message) => message,
        None => "Unknown error",
    };

    // print the error to the console
    if status.is_server_error() {
        tracing::error!(
            status = ?status,
            error = ?error_message,
            "Internal server error",
        );
    } else if status.is_client_error() {
        tracing::warn!(
            status = ?status,
            error = ?error_message,
            "Client error",
        );
    }

    let error_builder = ErrorBuilder::new(&status, &inner_error_message);

    match status {
        // Explicitly required by LFS documentation
        StatusCode::UNAUTHORIZED => error_builder.error("Unauthorized"),
        StatusCode::FORBIDDEN => error_builder.error("Missing write authorization"),
        StatusCode::NOT_FOUND => error_builder.error("Not found"),
        StatusCode::UNPROCESSABLE_ENTITY | StatusCode::BAD_REQUEST => error_builder
            .custom_status_with_message(StatusCode::UNPROCESSABLE_ENTITY, "Invalid payload"),
        // Explicitly allowed by LFS documentation
        StatusCode::NOT_ACCEPTABLE => {
            error_builder.error("Bad Accept header, should be application/vnd.git-lfs+json")
        }
        StatusCode::PAYLOAD_TOO_LARGE => {
            error_builder.error("Payload too large, try to send less files at the time")
        }
        StatusCode::TOO_MANY_REQUESTS => error_builder.error("Too many requests, try again later"),
        StatusCode::NOT_IMPLEMENTED => error_builder.with_message("Not implemented"),
        StatusCode::INSUFFICIENT_STORAGE => error_builder.error("Insufficient storage"),

        // Might happen, but not explicitly mentioned by LFS documentation, client might not support it
        StatusCode::METHOD_NOT_ALLOWED => {
            error_builder.error("Method not allowed, try GET or POST")
        }

        // Any other error is an internal server error
        _ => error_builder
            .custom_status_error(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
    }
}
