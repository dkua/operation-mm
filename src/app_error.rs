use axum::http::StatusCode;
use axum::response::IntoResponse;

pub(crate) struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            #[cfg(deploy_env = "dev")]
            format!("Something went wrong: {:#?}", self.0),
            #[cfg(deploy_env = "prod")]
            "Something went wrong",
        )
            .into_response();
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self(value.into())
    }
}
