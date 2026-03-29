use std::sync::mpsc::{channel, Receiver, Sender};

use super::command::Command;
use super::event::EventEnvelope;

pub type CommandSender = Sender<Command>;
pub type CommandReceiver = Receiver<Command>;
pub type EventSender = Sender<EventEnvelope>;
pub type EventReceiver = Receiver<EventEnvelope>;

/// Channels acts as the nerve center of the system.
///
/// It manages the two message directions in the system:
/// - Command channel sends commands to the command consumer.
/// - Event channel sends events to the event consumer.
pub struct Channels {
    pub command_tx: CommandSender,
    pub command_rx: CommandReceiver,
    pub event_tx: EventSender,
    pub event_rx: EventReceiver,
}
impl Channels {
    pub fn new() -> Self {
        let (command_tx, command_rx) = channel();
        let (event_tx, event_rx) = channel();

        Self {
            command_tx,
            command_rx,
            event_tx,
            event_rx,
        }
    }
}