#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("message: {0}")]
    Message(String),

    #[error("sqlx error: {0}")]
    Sqlx(sqlx::Error),

    #[error("anyhow error: {0}")]
    Any(anyhow::Error),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Sqlx(err)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Any(err)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
