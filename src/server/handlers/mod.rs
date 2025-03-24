use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

pub mod bring;
pub mod calendar;
pub mod food;
pub mod shopping;

// Error Wrapper for anyhow::Error.
pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

/// This is a simple error page that can be used to display an error message.
#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorPageTemplate {
    message: String,
}

/// Provides a fallback handler for all routes that are not found.
/// This displays a simple error page.
pub async fn fallback_handler() -> Result<impl IntoResponse, AppError> {
    let template = ErrorPageTemplate {
        message: "Seite konnte nicht gefunden werden".to_string(),
    };

    Ok(Html(template.render()?))
}
