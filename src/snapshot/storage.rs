use std::fs::{create_dir_all, File};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct SnapshotStore {
    base_path: String,
}
impl SnapshotStore {
    pub fn new(base_path: &str) -> Self {
        create_dir_all(base_path).ok();
        Self {
            base_path: base_path.to_string(),
        }
    }

    pub fn save(&self, data: Vec<u8>) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let file_path = format!("{}/snapshot-{}.bin", self.base_path, timestamp);

        let mut file = File::create(file_path).unwrap();
        file.write_all(&data).unwrap();
        file.sync_all().unwrap(); // fsync
    }
}