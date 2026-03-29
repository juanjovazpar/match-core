use crate::shared::{
    channel::{CommandReceiver, EventSender},
    event::EventEnvelope,
    types::{CommandSeq, EventSeq},
};
use crate::core::{ 
    sequencer::Sequencer,
    log_writter::LogWriter,
    deduplicator::Deduplicator
};
use crate::utils::time::now;
use crate::domain::matching::MatchingEngine;

/// Core matching engine loop.
///
/// Orchestrate the processing commands sequentially, ensuring total order,
/// durability (via WAL), and deterministic state transitions.
///
/// This component is single-threaded by design to guarantee consistency
/// and replayability of the system state.
pub struct Engine {
    pub command_rx: CommandReceiver,
    pub event_tx: EventSender,
    pub sequencer: Sequencer,
    pub wal: LogWriter,
    pub dedup: Deduplicator,
    pub matching: MatchingEngine,
}
impl Engine {
    pub fn run(&mut self) {
        loop {
            let command = match self.command_rx.recv() {
                Ok(cmd) => cmd,
                Err(_) => break,
            };

            // Skip the command if it was already managed
            if self.dedup.exists(&command) {
                continue;
            }

            // Get the next command sequence from the sequencer
            let command_seq: CommandSeq = self.sequencer.next_command_seq();

            // Persist the incoming command into logs
            self.wal.write_command(command_seq, &command);

            // Process the incomming command
            let events = self.matching.process(command_seq, command);

            // Handle events for the processed command
            for event in events {
                let event_seq: EventSeq = self.sequencer.next_event_seq();
                let envelope = EventEnvelope {
                    event_seq,
                    command_seq,
                    timestamp: now(),
                    event,
                };

                // Persist event in log
                self.wal.write_event(&envelope);
                
                // Emit event for consumers
                self.event_tx.send(envelope).ok();
            }
        }
    }
}