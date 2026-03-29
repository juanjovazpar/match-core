use std::sync::mpsc::{Receiver, Sender, channel};

use crate::shared::event::EventEnvelope;

pub type EventSender = Sender<EventEnvelope>;

pub type EventReceiver = Receiver<EventEnvelope>;

pub fn create_event_channel() -> (EventSender, EventReceiver) {
    channel()
}