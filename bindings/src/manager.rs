use dashmap::{mapref::one::RefMut, DashMap};
use std::sync::atomic::AtomicU64;
use tokio_kcp::KcpStream;

pub type StreamId = u64;

pub struct StreamManager {
  id: AtomicU64,
  stream_by_id: DashMap<StreamId, KcpStream>,
}

impl StreamManager {
  pub fn new() -> Self {
    StreamManager {
      id: AtomicU64::new(0),
      stream_by_id: DashMap::new(),
    }
  }

  pub fn len(&self) -> usize {
    self.stream_by_id.len()
  }

  fn next_id(&self) -> StreamId {
    let id = self.id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    id
  }

  pub fn insert_stream(&self, stream: KcpStream) -> StreamId {
    let id = self.next_id();

    self.stream_by_id.insert(id, stream);

    id
  }

  pub fn get_mut_stream(&self, id: StreamId) -> Option<RefMut<StreamId, KcpStream>> {
    self.stream_by_id.get_mut(&id)
  }

  pub fn remove_stream(&self, id: StreamId) -> Option<KcpStream> {
    self.stream_by_id.remove(&id).map(|(_, stream)| stream)
  }
}

#[test]
fn test_manager() {
  let manager = StreamManager::new();

  assert_eq!(manager.next_id(), 0);
  assert_eq!(manager.next_id(), 1);
  assert_eq!(manager.next_id(), 2);
}
