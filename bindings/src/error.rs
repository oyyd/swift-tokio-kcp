use kcp;
use thiserror;

impl From<std::net::AddrParseError> for SwiftKcpError {
  fn from(value: std::net::AddrParseError) -> Self {
    SwiftKcpError::Default {
      msg: value.to_string(),
    }
  }
}

impl From<kcp::Error> for SwiftKcpError {
  fn from(value: kcp::Error) -> Self {
    SwiftKcpError::Default {
      msg: value.to_string(),
    }
  }
}

impl From<std::io::Error> for SwiftKcpError {
  fn from(value: std::io::Error) -> Self {
    SwiftKcpError::Default {
      msg: value.to_string(),
    }
  }
}

impl From<tokio::task::JoinError> for SwiftKcpError {
  fn from(value: tokio::task::JoinError) -> Self {
    SwiftKcpError::Default {
      msg: value.to_string(),
    }
  }
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum SwiftKcpError {
  #[error("{msg}")]
  Default { msg: String },
}
