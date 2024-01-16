uniffi::include_scaffolding!("bindings");

mod error;
mod kcp_util;
mod manager;

use error::SwiftKcpError;
pub use kcp_util::KcpConfigParams;
use lazy_static::lazy_static;
use manager::{Manager, StreamId};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  runtime::Runtime,
  sync::{Mutex, RwLock},
};
use tokio_kcp::{KcpConfig, KcpListener, KcpStream};

type Result<T> = std::result::Result<T, error::SwiftKcpError>;

const READ_BUF: usize = 65535;

lazy_static! {
  static ref RUNTIME: Arc<RwLock<Option<Runtime>>> = Arc::new(RwLock::new(None));
  static ref STREAM_MANAGER: Arc<Mutex<Manager<KcpStream>>> = Arc::new(Mutex::new(Manager::new()));
  static ref LISTENER_MANAGER: Arc<Mutex<Manager<KcpListener>>> =
    Arc::new(Mutex::new(Manager::new()));
}

#[uniffi::export]
async fn init_runtime() -> Result<()> {
  let rt = Runtime::new()?;
  let mut runtime = RUNTIME.write().await;
  let _ = runtime.insert(rt);

  Ok(())
}

#[uniffi::export]
async fn deinit_runtime() {
  let rt = {
    let mut runtime = RUNTIME.write().await;

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
    let rt = RUNTIME.read().await;
    if rt.is_none() {
      return Err(SwiftKcpError::RuntimeNotInited);
    }
    let rt = rt.as_ref().unwrap();
    rt.spawn(async move {
      let stream = KcpStream::connect(&config, addr).await;
      stream
    })
  };

  let stream = join_handle.await??;

  let id = {
    let mut manager = STREAM_MANAGER.lock().await;
    manager.insert_stream(stream)
  };

  Ok(id)
}

#[uniffi::export]
async fn remove_stream(id: StreamId) -> Result<()> {
  let stream = {
    let mut manager = STREAM_MANAGER.lock().await;
    manager.remove_stream(id)
  };

  if stream.is_none() {
    return Err(SwiftKcpError::NoStreamForId { id });
  }

  Ok(())
}

#[uniffi::export]
async fn write_stream(id: StreamId, data: Vec<u8>) -> Result<()> {
  let rt = RUNTIME.read().await;
  if rt.is_none() {
    return Err(SwiftKcpError::RuntimeNotInited);
  }
  let rt = rt.as_ref().unwrap();

  rt.spawn(async move {
    let stream = {
      let manager = STREAM_MANAGER.lock().await;
      manager.get_mut_stream(id)
    };
    if stream.is_none() {
      return Err(SwiftKcpError::NoStreamForId { id });
    }
    let stream = stream.unwrap();
    stream.lock().await.write_all(&data).await?;

    Ok(())
  })
  .await??;

  Ok(())
}

#[uniffi::export]
async fn read_stream(id: StreamId) -> Result<Vec<u8>> {
  let rt = RUNTIME.read().await;
  if rt.is_none() {
    return Err(SwiftKcpError::RuntimeNotInited);
  }
  let rt = rt.as_ref().unwrap();

  let (n, buf) = rt
    .spawn(async move {
      let stream = {
        let manager = STREAM_MANAGER.lock().await;
        manager.get_mut_stream(id)
      };
      if stream.is_none() {
        return Err(SwiftKcpError::NoStreamForId { id });
      }
      let stream = stream.unwrap();
      let mut buf: Vec<u8> = vec![0; READ_BUF];
      let n = stream.lock().await.read(&mut buf).await?;
      Ok((n, buf))
    })
    .await??;

  Ok(buf[0..n].to_vec())
}

#[uniffi::export]
async fn get_stream_count() -> u32 {
  STREAM_MANAGER.lock().await.len() as u32
}

// Shuts down the output stream, ensuring that the value can be dropped cleanly.
#[uniffi::export]
async fn shutdown_stream(id: StreamId) -> Result<()> {
  let rt = RUNTIME.read().await;
  if rt.is_none() {
    return Err(SwiftKcpError::RuntimeNotInited);
  }
  let rt = rt.as_ref().unwrap();

  rt.spawn(async move {
    let stream = {
      let manager = STREAM_MANAGER.lock().await;
      manager.get_mut_stream(id)
    };
    if stream.is_none() {
      return Err(SwiftKcpError::NoStreamForId { id });
    }

    let stream = stream.unwrap();
    stream.lock().await.shutdown().await?;

    Ok(())
  })
  .await??;

  Ok(())
}

// Call kcp flush behind. Note that this method won't guarantee data is transfered to
// the remove side.
#[uniffi::export]
async fn flush_stream(id: StreamId) -> Result<()> {
  let rt = RUNTIME.read().await;
  if rt.is_none() {
    return Err(SwiftKcpError::RuntimeNotInited);
  }
  let rt = rt.as_ref().unwrap();

  rt.spawn(async move {
    let stream = {
      let manager = STREAM_MANAGER.lock().await;
      manager.get_mut_stream(id)
    };
    if stream.is_none() {
      return Err(SwiftKcpError::NoStreamForId { id });
    }

    let stream = stream.unwrap();
    stream.lock().await.flush().await?;

    Ok(())
  })
  .await??;

  Ok(())
}

// NOTE: Empty operation.
#[uniffi::export]
async fn read_exact_stream(id: StreamId, len: u32) -> Result<Vec<u8>> {
  let rt = RUNTIME.read().await;
  if rt.is_none() {
    return Err(SwiftKcpError::RuntimeNotInited);
  }
  let rt = rt.as_ref().unwrap();

  let data = rt
    .spawn(async move {
      let stream = {
        let manager = STREAM_MANAGER.lock().await;
        manager.get_mut_stream(id)
      };
      if stream.is_none() {
        return Err(SwiftKcpError::NoStreamForId { id });
      }

      let stream = stream.unwrap();
      let mut data: Vec<u8> = vec![0; len as usize];

      stream.lock().await.read_exact(&mut data).await?;

      Ok(data)
    })
    .await??;

  Ok(data)
}

#[uniffi::export]
async fn new_listener(bind_addr_str: String, params: KcpConfigParams) -> Result<StreamId> {
  let config: KcpConfig = params.into();
  let addr = SocketAddr::from_str(&bind_addr_str)?;

  let join_handle = {
    let rt = RUNTIME.read().await;
    if rt.is_none() {
      return Err(SwiftKcpError::RuntimeNotInited);
    }
    let rt = rt.as_ref().unwrap();
    rt.spawn(async move { KcpListener::bind(config, addr).await })
  };

  let listener = join_handle.await??;

  let id = LISTENER_MANAGER.lock().await.insert_stream(listener);

  Ok(id)
}

#[uniffi::export]
async fn remove_listener(id: StreamId) -> Result<()> {
  let listener = LISTENER_MANAGER.lock().await.remove_stream(id);

  if listener.is_none() {
    return Err(SwiftKcpError::NoListenerForId { id });
  }

  Ok(())
}

#[derive(uniffi::Record)]
struct IDAddrPair {
  id: StreamId,
  addr: String,
}

#[uniffi::export]
async fn accepet(id: StreamId) -> Result<IDAddrPair> {
  let rt = RUNTIME.read().await;
  if rt.is_none() {
    return Err(SwiftKcpError::RuntimeNotInited);
  }
  let rt = rt.as_ref().unwrap();

  let (stream, addr) = rt
    .spawn(async move {
      let listener = {
        let manager = LISTENER_MANAGER.lock().await;
        manager.get_mut_stream(id)
      };
      if listener.is_none() {
        return Err(SwiftKcpError::NoListenerForId { id });
      }
      let listener = listener.unwrap();

      let ret = listener.lock().await.accept().await?;

      Ok(ret)
    })
    .await??;

  let id = STREAM_MANAGER.lock().await.insert_stream(stream);

  Ok(IDAddrPair {
    id,
    addr: addr.to_string(),
  })
}

#[uniffi::export]
async fn local_addr(id: StreamId) -> Result<String> {
  let listener = LISTENER_MANAGER.lock().await.get_mut_stream(id);

  if listener.is_none() {
    return Err(SwiftKcpError::NoListenerForId { id });
  }

  let addr = listener.unwrap().lock().await.local_addr()?;

  Ok(addr.to_string())
}
