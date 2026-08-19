#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("message: {0}")]
    Message(String),

    #[error("sqlx error: {0}")]
    Sqlx(sqlx::Error),
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Error::Sqlx(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
