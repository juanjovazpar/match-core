use std::time::{SystemTime, UNIX_EPOCH};

use crate::shared::types::Timestamp;

pub fn now() -> Timestamp {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as Timestamp
}
