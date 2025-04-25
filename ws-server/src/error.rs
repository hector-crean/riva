use crate::file_storage::s3::S3Error;

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Axum(#[from] axum::Error),
    #[error(transparent)]
    Subscriber(#[from] tracing::subscriber::SetGlobalDefaultError),



    #[error(transparent)]
    S3Error(#[from] S3Error),
    #[error(transparent)]
    MultipartError(#[from] axum::extract::multipart::MultipartError),
    #[error(transparent)]
    SurrealdbError(#[from] surrealdb::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    UrlError(#[from] url::ParseError),
}
