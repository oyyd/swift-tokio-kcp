uniffi::include_scaffolding!("bindings");

mod error;
mod kcp_util;
mod manager;

use error::SwiftKcpError;
pub use kcp_util::KcpConfigParams;
use lazy_static::lazy_static;
use manager::{StreamId, StreamManager};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  runtime::Runtime,
  sync::Mutex,
};
use tokio_kcp::{KcpConfig, KcpStream};

type Result<T> = std::result::Result<T, error::SwiftKcpError>;

const READ_BUF: usize = 65535;

lazy_static! {
  static ref RUNTIME: Arc<Mutex<Option<Runtime>>> = Arc::new(Mutex::new(None));
  static ref MANAGER: Arc<StreamManager> = Arc::new(StreamManager::new());
}

#[uniffi::export]
async fn init_runtime() -> Result<()> {
  let rt = Runtime::new()?;
  let mut runtime = RUNTIME.lock().await;
  let _ = runtime.insert(rt);

  Ok(())
}

#[uniffi::export]
async fn deinit_runtime() {
  let rt = {
    let mut runtime = RUNTIME.lock().await;

    runtime.take()
  };

  if rt.is_some() {
    rt.unwrap()
      .shutdown_timeout(std::time::Duration::from_secs(1));
  }
}

#[uniffi::export]
fn default_kcp_config_params() -> KcpConfigParams {
  KcpConfigParams::default()
}

#[uniffi::export]
async fn new_stream(addr_str: String, params: KcpConfigParams) -> Result<StreamId> {
  let config: KcpConfig = params.into();
  let addr = SocketAddr::from_str(&addr_str)?;

  let join_handle = {
    let mut rt = RUNTIME.lock().await;
    if rt.is_none() {
      return Err(SwiftKcpError::RuntimeNotInited);
    }
    let rt = rt.as_mut().unwrap();
    rt.spawn(async move {
      let stream = KcpStream::connect(&config, addr).await;
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
    return Err(SwiftKcpError::NoStreamForId { id });
  }

  Ok(())
}

#[uniffi::export]
async fn write_stream(id: StreamId, data: Vec<u8>) -> Result<()> {
  let mut rt = RUNTIME.lock().await;
  if rt.is_none() {
    return Err(SwiftKcpError::RuntimeNotInited);
  }
  let rt = rt.as_mut().unwrap();

  rt.spawn(async move {
    let stream = MANAGER.get_mut_stream(id);
    if stream.is_none() {
      return Err(SwiftKcpError::NoStreamForId { id });
    }
    let mut stream: dashmap::mapref::one::RefMut<'_, u64, KcpStream> = stream.unwrap();
    stream.write_all(&data).await?;

    Ok(())
  })
  .await??;

  Ok(())
}

#[uniffi::export]
async fn read_stream(id: StreamId) -> Result<Vec<u8>> {
  let mut rt = RUNTIME.lock().await;
  if rt.is_none() {
    return Err(SwiftKcpError::RuntimeNotInited);
  }
  let rt = rt.as_mut().unwrap();

  let (n, buf) = rt
    .spawn(async move {
      let stream = MANAGER.get_mut_stream(id);
      if stream.is_none() {
        return Err(SwiftKcpError::NoStreamForId { id });
      }
      let mut stream = stream.unwrap();
      let mut buf: Vec<u8> = vec![0; READ_BUF];
      let n = stream.read(&mut buf).await?;
      Ok((n, buf))
    })
    .await??;

  Ok(buf[0..n].to_vec())
}
