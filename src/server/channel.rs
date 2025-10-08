use once_cell::sync::OnceCell;
use std:: sync:: Arc;
use tokio:: sync:: mpsc:: Sender;

use crate::Message;

static TX: OnceCell<Arc<Sender<Message>>> = OnceCell::new();

pub fn get_sender() -> Arc<Sender<Message>> {
    TX.get().expect("TX already initialized").clone()
}

pub fn set_sender(sender: Sender<Message>) {
    TX.set(Arc::new(sender)).expect("TX already initialized");
}