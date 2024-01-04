use thiserror;

macro_rules! impl_from {
  ($from_type:ty) => {
    impl From<$from_type> for SwiftKcpError {
      fn from(value: $from_type) -> Self {
        SwiftKcpError::Default {
          msg: value.to_string(),
        }
      }
    }
  };
}

impl_from!(std::net::AddrParseError);
impl_from!(kcp::Error);
impl_from!(std::io::Error);
impl_from!(tokio::task::JoinError);

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum SwiftKcpError {
  #[error("{msg}")]
  Default { msg: String },

  #[error("RUNTIME not inited")]
  RuntimeNotInited,

  #[error("Stream not found for id {id}")]
  NoStreamForId { id: u64 },

  #[error("Listener not found for id {id}")]
  NoListenerForId { id: u64 },
}
