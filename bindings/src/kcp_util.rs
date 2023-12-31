use std::time;
use tokio_kcp::KcpConfig;

#[derive(uniffi::Record, Default)]
pub struct KcpConfigParams {
  /// Max Transmission Unit
  pub mtu: Option<i16>,
  /// Enable nodelay
  pub nodelay: Option<bool>,
  /// Internal update interval (ms)
  pub nodelay_interval: Option<i32>,
  /// ACK number to enable fast resend
  pub nodelay_resend: Option<i32>,
  /// Disable congetion control
  pub nodelay_nc: Option<bool>,
  /// Send window size
  pub window_size_send: Option<u16>,
  /// Recv window size
  pub window_size_recv: Option<u16>,
  /// Session expire duration, default is 90 seconds
  pub session_expire_milisec: Option<u32>,
  /// Flush KCP state immediately after write
  pub flush_write: Option<bool>,
  /// Flush ACKs immediately after input
  pub flush_acks_input: Option<bool>,
  /// Stream mode
  pub stream: Option<bool>,
}

impl Into<KcpConfig> for KcpConfigParams {
  fn into(self) -> KcpConfig {
    let mut config = KcpConfig::default();

    if self.mtu.is_some() {
      config.mtu = self.mtu.unwrap() as usize;
    }
    if self.nodelay.is_some() {
      config.nodelay.nodelay = self.nodelay.unwrap();
    }
    if self.nodelay_interval.is_some() {
      config.nodelay.interval = self.nodelay_interval.unwrap();
    }
    if self.nodelay_resend.is_some() {
      config.nodelay.resend = self.nodelay_resend.unwrap();
    }
    if self.nodelay_nc.is_some() {
      config.nodelay.nc = self.nodelay_nc.unwrap();
    }
    if self.window_size_send.is_some() && self.window_size_recv.is_some() {
      config.wnd_size = (
        self.window_size_send.unwrap(),
        self.window_size_recv.unwrap(),
      )
    }
    if self.session_expire_milisec.is_some() {
      let milisec = self.session_expire_milisec.unwrap();
      config.session_expire = time::Duration::from_millis(milisec as u64);
    }
    if self.flush_write.is_some() {
      config.flush_write = self.flush_write.unwrap();
    }
    if self.flush_acks_input.is_some() {
      config.flush_acks_input = self.flush_acks_input.unwrap();
    }
    if self.stream.is_some() {
      config.stream = self.stream.unwrap();
    }
    config
  }
}
