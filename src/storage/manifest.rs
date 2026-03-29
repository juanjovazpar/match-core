use std::fs::{File};
use std::io::{Write, Read};

pub struct Manifest {
    pub current_snapshot: Option<String>,
    pub wal_path: String,
}
impl Manifest {
    pub fn new(wal_path: &str) -> Self {
        Self {
            current_snapshot: None,
            wal_path: wal_path.to_string(),
        }
    }

    pub fn save(&self, path: &str) {
        let content = format!(
            "snapshot={}\nwal={}",
            self.current_snapshot.clone().unwrap_or_default(),
            self.wal_path
        );

        let mut file = File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.sync_all().unwrap();
    }

    pub fn load(path: &str) -> Self {
        let mut file = File::open(path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        let mut snapshot = None;
        let mut wal = String::new();

        for line in content.lines() {
            if line.starts_with("snapshot=") {
                let v = line.replace("snapshot=", "");
                if !v.is_empty() {
                    snapshot = Some(v);
                }
            } else if line.starts_with("wal=") {
                wal = line.replace("wal=", "");
            }
        }

        Self {
            current_snapshot: snapshot,
            wal_path: wal,
        }
    }
}