use std::sync::Arc;
use std::{collections::HashMap, sync::atomic::AtomicU64};
use tokio::sync::Mutex;

pub type StreamId = u64;

pub struct Manager<T> {
  id: AtomicU64,
  item_by_id: HashMap<StreamId, Arc<Mutex<T>>>,
}

impl<T> Manager<T> {
  pub fn new() -> Self {
    Self {
      id: AtomicU64::new(0),
      item_by_id: HashMap::new(),
    }
  }

  pub fn len(&self) -> usize {
    self.item_by_id.len()
  }

  fn next_id(&self) -> StreamId {
    let id = self.id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    id
  }

  pub fn insert_stream(&mut self, stream: T) -> StreamId {
    let id = self.next_id();

    self.item_by_id.insert(id, Arc::new(Mutex::new(stream)));

    id
  }

  pub fn get_mut_stream(&self, id: StreamId) -> Option<Arc<Mutex<T>>> {
    let d = self.item_by_id.get(&id);

    if d.is_none() {
      return None;
    }

    let stream = d.unwrap();

    let e = stream.clone();

    Some(e)
  }

  pub fn remove_stream(&mut self, id: StreamId) -> Option<Arc<Mutex<T>>> {
    self.item_by_id.remove(&id).map(|stream| stream)
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
