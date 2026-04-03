use thiserror::Error;
use tracing::error;

use crate::shared::{channel::CommandSender, command::Command};

#[derive(Debug, Error)]
pub enum DispatchError {
    #[error("engine command channel closed")]
    ChannelClosed,
}

/// Delivers validated ingress commands into the engine command channel (async, bounded).
pub struct Dispatcher {
    tx: CommandSender,
}

impl Dispatcher {
    pub fn new(tx: CommandSender) -> Self {
        Self { tx }
    }

    pub async fn submit(&self, cmd: Command) -> Result<(), DispatchError> {
        self.tx.send(cmd).await.map_err(|e| {
            error!(error = %e, "failed to dispatch command to engine");
            DispatchError::ChannelClosed
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::channel::Channels;
    use crate::shared::command::CancelOrderCommand;

    #[tokio::test]
    async fn submit_ok_when_receiver_alive() {
        let ch = Channels::new(128);
        let d = Dispatcher::new(ch.command_tx.clone());
        let mut rx = ch.command_rx;
        let cmd = Command::CancelOrder(CancelOrderCommand {
            command_id: "c".into(),
            order_id: "o".into(),
            timestamp: 1,
        });
        assert!(d.submit(cmd).await.is_ok());
        assert!(rx.recv().await.is_some());
    }

    #[tokio::test]
    async fn submit_err_when_channel_disconnected() {
        let ch = Channels::new(128);
        let d = Dispatcher::new(ch.command_tx);
        drop(ch.command_rx);
        let cmd = Command::CancelOrder(CancelOrderCommand {
            command_id: "c".into(),
            order_id: "o".into(),
            timestamp: 1,
        });
        let err = d.submit(cmd).await.unwrap_err();
        assert!(matches!(err, DispatchError::ChannelClosed));
    }
}
