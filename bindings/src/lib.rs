uniffi::include_scaffolding!("bindings");

mod error;
mod manager;

use error::SwiftKcpError;
use lazy_static::lazy_static;
use manager::{StreamId, StreamManager};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;
use std::sync::Arc;
use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  runtime::Runtime,
  sync::Mutex,
};
use tokio_kcp::{KcpConfig, KcpStream};

type Result<T> = std::result::Result<T, error::SwiftKcpError>;

lazy_static! {
  static ref RUNTIME: Arc<Mutex<Option<Runtime>>> = Arc::new(Mutex::new(None));
  static ref MANAGER: Arc<StreamManager> = Arc::new(StreamManager::new());
}

#[uniffi::export]
async fn init_runtime() -> Result<()> {
  // TODO support parameters for runtimes
  let rt = Runtime::new()?;

  let mut runtime = RUNTIME.lock().await;

  let _ = runtime.insert(rt);

  Ok(())
}

#[uniffi::export]
async fn deinit_runtime() -> Result<()> {
  let mut runtime = RUNTIME.lock().await;

  runtime.take();

  Ok(())
}

#[uniffi::export]
async fn new_stream(addr_str: String) -> Result<StreamId> {
  let config = KcpConfig::default();
  let addr = SocketAddr::from_str(&addr_str)?;

  let join_handle = {
    let mut rt = RUNTIME.lock().await;
    if rt.is_none() {
      return Err(SwiftKcpError::Default {
        msg: "RUNTIME not inited".to_string(),
      });
    }
    let rt = rt.as_mut().unwrap();
    rt.spawn(async move {
      let stream = KcpStream::connect(&config, addr).await;
      // stream
      stream
    })
  };

  let stream = join_handle.await??;

  let id = MANAGER.insert_stream(stream);

  Ok(id)
}

#[uniffi::export]
async fn remove_stream(id: StreamId) -> Result<()> {
  let stream = MANAGER.remove_stream(id);

  if stream.is_none() {
    return Err(SwiftKcpError::Default {
      msg: format!("stream with id: {} doesn't exit", id),
    });
  }

  Ok(())
}

#[uniffi::export]
async fn write_stream(id: StreamId, data: Vec<u8>) -> Result<()> {
  let mut rt = RUNTIME.lock().await;

  if rt.is_none() {
    return Err(SwiftKcpError::Default {
      msg: "RUNTIME not inited".to_string(),
    });
  }

  let rt = rt.as_mut().unwrap();

  rt.spawn(async move {
    let stream = MANAGER.get_mut_stream(id);
    if stream.is_none() {
      return Err(SwiftKcpError::Default {
        msg: format!("no stream with id {}", id),
      });
    }
    let mut stream = stream.unwrap();
    stream.write_all(&data).await?;

    Ok(())
  })
  .await??;

  Ok(())
}

#[uniffi::export]
async fn read_stream(id: StreamId) -> Result<Vec<u8>> {
  Ok(vec![])
}
