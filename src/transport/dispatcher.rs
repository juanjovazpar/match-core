use std::time::Duration;

use thiserror::Error;
use tokio::time::timeout;
use tracing::{error, warn};

use crate::shared::{channel::CommandSender, command::Command};

#[derive(Debug, Error)]
pub enum DispatchError {
    #[error("engine command channel closed")]
    ChannelClosed,
    #[error("timed out waiting for space in the engine command queue")]
    EnqueueTimeout,
}

/// Delivers validated ingress commands into the engine command channel (async, bounded).
///
/// Uses a bounded wait ([`enqueue_timeout`](Self::new)): if the channel stays full longer than
/// that, [`DispatchError::EnqueueTimeout`] is returned so ingress can reject without unbounded
/// latency.
pub struct Dispatcher {
    tx: CommandSender,
    enqueue_timeout: Duration,
}

impl Dispatcher {
    pub fn new(tx: CommandSender, enqueue_timeout: Duration) -> Self {
        Self {
            tx,
            enqueue_timeout,
        }
    }

    pub async fn submit(&self, cmd: Command) -> Result<(), DispatchError> {
        match timeout(self.enqueue_timeout, self.tx.send(cmd)).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => {
                error!(error = %e, "failed to dispatch command to engine");
                Err(DispatchError::ChannelClosed)
            }
            Err(_) => {
                warn!(
                    timeout_ms = self.enqueue_timeout.as_millis(),
                    "command enqueue timed out (engine command queue backlog)"
                );
                Err(DispatchError::EnqueueTimeout)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::channel::Channels;
    use crate::shared::command::CancelOrderCommand;
    use std::time::Duration;

    fn sample_cancel() -> Command {
        Command::CancelOrder(CancelOrderCommand {
            command_id: "c".into(),
            order_id: "o".into(),
            timestamp: 1,
        })
    }

    #[tokio::test]
    async fn submit_ok_when_receiver_alive() {
        let ch = Channels::new(128);
        let d = Dispatcher::new(ch.command_tx.clone(), Duration::from_secs(5));
        let mut rx = ch.command_rx;
        let cmd = sample_cancel();
        assert!(d.submit(cmd).await.is_ok());
        assert!(rx.recv().await.is_some());
    }

    #[tokio::test]
    async fn submit_err_when_channel_disconnected() {
        let ch = Channels::new(128);
        let d = Dispatcher::new(ch.command_tx, Duration::from_secs(5));
        drop(ch.command_rx);
        let cmd = sample_cancel();
        let err = d.submit(cmd).await.unwrap_err();
        assert!(matches!(err, DispatchError::ChannelClosed));
    }

    #[tokio::test]
    async fn submit_enqueue_timeout_when_channel_stays_full() {
        let ch = Channels::new(2);
        let d = Dispatcher::new(ch.command_tx.clone(), Duration::from_millis(80));
        let _rx = ch.command_rx;

        d.submit(sample_cancel()).await.unwrap();
        d.submit(sample_cancel()).await.unwrap();

        let err = d.submit(sample_cancel()).await.unwrap_err();
        assert!(matches!(err, DispatchError::EnqueueTimeout));
    }
}
