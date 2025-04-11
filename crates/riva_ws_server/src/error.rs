#[derive(Debug, thiserror::Error)]
pub enum WsServerError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Axum(#[from] axum::Error),
    #[error(transparent)]
    Subscriber(#[from] tracing::subscriber::SetGlobalDefaultError),
}
