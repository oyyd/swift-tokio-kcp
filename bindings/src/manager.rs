use dashmap::{mapref::one::RefMut, DashMap};
use std::sync::atomic::AtomicU64;

pub type StreamId = u64;

pub struct Manager<T> {
  id: AtomicU64,
  item_by_id: DashMap<StreamId, T>,
}

impl<T> Manager<T> {
  pub fn new() -> Self {
    Self {
      id: AtomicU64::new(0),
      item_by_id: DashMap::new(),
    }
  }

  pub fn len(&self) -> usize {
    self.item_by_id.len()
  }

  fn next_id(&self) -> StreamId {
    let id = self.id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    id
  }

  pub fn insert_stream(&self, stream: T) -> StreamId {
    let id = self.next_id();

    self.item_by_id.insert(id, stream);

    id
  }

  pub fn get_mut_stream(&self, id: StreamId) -> Option<RefMut<StreamId, T>> {
    self.item_by_id.get_mut(&id)
  }

  pub fn remove_stream(&self, id: StreamId) -> Option<T> {
    self.item_by_id.remove(&id).map(|(_, stream)| stream)
  }
}

#[test]
fn test_manager() {
  use tokio_kcp::KcpStream;
  let manager = Manager::<KcpStream>::new();

  assert_eq!(manager.next_id(), 0);
  assert_eq!(manager.next_id(), 1);
  assert_eq!(manager.next_id(), 2);
}
