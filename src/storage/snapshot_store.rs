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

    pub fn save(&self, data: &[u8]) -> String {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let path = format!("{}/snapshot-{}.bin", self.base_path, ts);

        let mut file = File::create(&path).unwrap();
        file.write_all(data).unwrap();
        file.sync_all().unwrap();

        path
    }
}