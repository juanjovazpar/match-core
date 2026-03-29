use std::fs::File;
use std::io::Read;

pub struct LoadedState {
    pub snapshot: Option<Vec<u8>>,
    pub wal_entries: Vec<String>,
}

pub struct Loader {
    pub snapshot_path: String,
    pub wal_path: String,
}
impl Loader {
    pub fn new(snapshot_path: &str, wal_path: &str) -> Self {
        Self {
            snapshot_path: snapshot_path.to_string(),
            wal_path: wal_path.to_string(),
        }
    }

    pub fn load(&self) -> LoadedState {
        let snapshot = self.load_snapshot();
        let wal_entries = self.load_wal();

        LoadedState {
            snapshot,
            wal_entries,
        }
    }

    fn load_snapshot(&self) -> Option<Vec<u8>> {
        if let Ok(mut file) = File::open(&self.snapshot_path) {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).ok()?;
            Some(buffer)
        } else {
            None
        }
    }

    fn load_wal(&self) -> Vec<String> {
        if let Ok(mut file) = File::open(&self.wal_path) {
            let mut content = String::new();
            file.read_to_string(&mut content).ok();

            content.lines().map(|l| l.to_string()).collect()
        } else {
            vec![]
        }
    }
}