use std::fs::{OpenOptions, File};
use std::io::Write;

pub struct WalStore {
    file: File,
}
impl WalStore {
    pub fn new(path: &str) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();

        Self { file }
    }

    pub fn append(&mut self, data: &[u8]) {
        self.file.write_all(data).unwrap();
        self.file.sync_all().unwrap();
    }
}