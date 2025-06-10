use axum::{extract::Request, middleware::Next, response::Response};
use chrono::Utc;

/// Middleware that logs non-2xx HTTP responses with timestamp and details
pub async fn log_errors(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();

    let response = next.run(request).await;
    let status = response.status();

    // Log all non-2xx responses for debugging
    if !status.is_success() {
        tracing::debug!(
            method = %method,
            uri = %uri,
            status_code = %status,
            status_text = %status.canonical_reason().unwrap_or("Unknown"),
            timestamp = %Utc::now().to_rfc3339(),
            "Non-2xx HTTP response"
        );
    }

    response
}
