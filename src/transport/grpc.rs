use crate::shared::{channel::CommandSender, command::{CancelOrderCommand, Command, NewOrderCommand}};

pub struct GrpcServer {
    tx: CommandSender,
}
impl GrpcServer {
    pub fn new(tx: CommandSender) -> Self {
        Self { tx }
    }

    pub fn submit_order(&self, req: NewOrderCommand) {
        let cmd = Command::NewOrder(req);

        if let Err(e) = self.tx.send(cmd) {
            eprintln!("❌ Failed to send command to core: {:?}", e);
        }
    }

    pub fn cancel_order(&self, req: CancelOrderCommand) {
        let cmd = Command::CancelOrder(req);

        if let Err(e) = self.tx.send(cmd) {
            eprintln!("❌ Failed to send command to core: {:?}", e);
        }
    }
}