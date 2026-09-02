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

impl From<&str> for AppError {
    fn from(message: &str) -> Self {
        AppError::Message(message.to_string())
    }
}

impl From<String> for AppError {
    fn from(message: String) -> Self {
        AppError::Message(message)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
