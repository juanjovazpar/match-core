use std::sync::mpsc::{Receiver, Sender};

use tokio::sync::mpsc;

use super::command::Command;
use super::event::EventEnvelope;

pub type CommandSender = mpsc::Sender<Command>;
pub type CommandReceiver = mpsc::Receiver<Command>;
pub type EventSender = Sender<EventEnvelope>;
pub type EventReceiver = Receiver<EventEnvelope>;

/// Channels act as the nerve center of the system.
///
/// - **Commands**: async `tokio::sync::mpsc` (bounded) from ingress to the engine.
/// - **Events**: sync `std::sync::mpsc` from the engine to the publisher thread.
pub struct Channels {
    pub command_tx: CommandSender,
    pub command_rx: CommandReceiver,
    pub event_tx: EventSender,
    pub event_rx: EventReceiver,
}
impl Channels {
    pub fn new(command_channel_capacity: usize) -> Self {
        let (command_tx, command_rx) = tokio::sync::mpsc::channel(command_channel_capacity);
        let (event_tx, event_rx) = std::sync::mpsc::channel();

        Self {
            command_tx,
            command_rx,
            event_tx,
            event_rx,
        }
    }
}
