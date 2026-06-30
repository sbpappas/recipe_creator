use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Message(String),
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Askama(#[from] askama::Error),
    #[error(transparent)]
    Llm(#[from] crate::llm::LlmError),
}

impl AppError {
    pub fn msg(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found").into_response(),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
            AppError::Message(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            AppError::Sqlx(e) => {
                tracing::error!("database error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong. Please try again.",
                )
                    .into_response()
            }
            AppError::Askama(e) => {
                tracing::error!("template error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong. Please try again.",
                )
                    .into_response()
            }
            AppError::Llm(e) => {
                tracing::warn!("llm error: {e}");
                (StatusCode::BAD_GATEWAY, e.to_string()).into_response()
            }
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    use askama::Template;
    use axum::http::header::CONTENT_TYPE;

    #[derive(Template)]
    #[template(source = "<h1>{{ name }}</h1>", ext = "html")]
    struct TestTemplate {
        name: &'static str,
    }

    #[test]
    fn render_sets_html_content_type() {
        let response = render(TestTemplate { name: "Recipe" }).unwrap();
        let content_type = response.headers().get(CONTENT_TYPE).unwrap().to_str().unwrap();

        assert_eq!(content_type, "text/html; charset=utf-8");
    }
}

pub fn render<T: Template>(template: T) -> AppResult<Response> {
    Ok(Html(template.render()?).into_response())
}
