use std::thread;
use std::time::Duration;

use crate::snapshot::serializer::serialize;
use crate::snapshot::storage::SnapshotStore;

pub struct Snapshotter<F>
where
    F: Fn() -> Vec<u8> + Send + 'static,
{
    snapshot_fn: F,
    store: SnapshotStore,
    interval: Duration,
}
impl<F> Snapshotter<F>
where
    F: Fn() -> Vec<u8> + Send + 'static,
{
    pub fn new(snapshot_fn: F, store: SnapshotStore, interval: Duration) -> Self {
        Self {
            snapshot_fn,
            store,
            interval,
        }
    }

    pub fn run(self) {
        thread::spawn(move || loop {
            thread::sleep(self.interval);

            let state = (self.snapshot_fn)();
            let data = serialize(&state);

            self.store.save(data);
        });
    }
}