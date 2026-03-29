use tracing::error;

use crate::shared::{channel::CommandSender, command::Command};

/// Dispatcher component to deliver incoming commands.
///
/// It manages the transmitor channel to send commands to the Engine.
pub struct Dispatcher {
    tx: CommandSender,
}
impl Dispatcher {
    pub fn new(tx: CommandSender) -> Self {
        Self { tx }
    }

    pub fn submit(&self, cmd: Command) {
        if let Err(e) = self.tx.send(cmd) {
            error!(?e, "Failed to send command to core");
        }
    }
}