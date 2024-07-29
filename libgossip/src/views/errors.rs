#[derive(Debug, thiserror::Error, uniffi::Object)]
#[error("{err:?}")]
pub struct GossipError {
    err: anyhow::Error
}
impl From<anyhow::Error> for GossipError {
    fn from(err: anyhow::Error) -> Self {
        Self { err }
    }
}