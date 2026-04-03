use tonic::{Request, Response, Status};
use tracing::{debug, warn};

use crate::shared::command::{CancelOrderCommand, Command, NewOrderCommand};
use crate::transport::dispatcher::{DispatchError, Dispatcher};
use crate::transport::metrics::record_grpc_request;
use crate::transport::proto::pb;

use super::validator::Validator;

fn order_cmd_reply(order_id: String, status: i32, error_code: Option<i32>) -> pb::OrderCmdReply {
    pb::OrderCmdReply {
        order_id,
        status,
        error_code,
    }
}

pub struct OrderCmdService {
    dispatcher: Dispatcher,
    validator: Validator,
}

impl OrderCmdService {
    pub fn new(dispatcher: Dispatcher, symbol: String) -> Self {
        Self {
            dispatcher,
            validator: Validator::new(symbol.clone()),
        }
    }
}

#[tonic::async_trait]
impl pb::order_cmd_service_server::OrderCmdService for OrderCmdService {
    async fn submit(
        &self,
        request: Request<pb::SubmitOrderCmdRequest>,
    ) -> Result<Response<pb::OrderCmdReply>, Status> {
        let req = request.into_inner();
        let reply_order_id = req.order_id.clone();
        let (side, order_type, time_in_force) =
            match self.validator.validate_submit_order_command(&req) {
                Ok(v) => v,
                Err(code) => {
                    warn!(
                        rpc = "Submit",
                        command_id = %req.command_id,
                        order_id = %reply_order_id,
                        reject_code = code as i32,
                        "submit rejected: validation"
                    );
                    record_grpc_request("submit", "rejected_validation");
                    return Ok(Response::new(order_cmd_reply(
                        reply_order_id,
                        pb::OrderCmdAck::Rejected as i32,
                        Some(code as i32),
                    )));
                }
            };
        let command_id_for_log = req.command_id.clone();
        let cmd = Command::NewOrder(NewOrderCommand {
            command_id: req.command_id,
            order_id: req.order_id,
            user_id: req.user_id,
            symbol: req.symbol,
            side,
            order_type,
            price: req.price,
            quantity: req.quantity,
            timestamp: req.timestamp,
            time_in_force,
        });

        match self.dispatcher.submit(cmd).await {
            Ok(()) => {
                debug!(
                    rpc = "Submit",
                    command_id = %command_id_for_log,
                    order_id = %reply_order_id,
                    "submit queued for engine"
                );
                record_grpc_request("submit", "queued");
                Ok(Response::new(order_cmd_reply(
                    reply_order_id,
                    pb::OrderCmdAck::Queued as i32,
                    None,
                )))
            }
            Err(e @ DispatchError::ChannelClosed) => {
                warn!(
                    rpc = "Submit",
                    command_id = %command_id_for_log,
                    order_id = %reply_order_id,
                    error = %e,
                    "submit rejected: engine unavailable"
                );
                record_grpc_request("submit", "rejected_engine_unavailable");
                Ok(Response::new(order_cmd_reply(
                    reply_order_id,
                    pb::OrderCmdAck::Rejected as i32,
                    Some(pb::CmdErr::SubmitEngineUnavailable as i32),
                )))
            }
        }
    }

    async fn cancel(
        &self,
        request: Request<pb::CancelOrderCmdRequest>,
    ) -> Result<Response<pb::OrderCmdReply>, Status> {
        let req = request.into_inner();
        let reply_order_id = req.order_id.clone();

        if let Err(code) = self.validator.validate_cancel_order_command(&req) {
            warn!(
                rpc = "Cancel",
                command_id = %req.command_id,
                order_id = %reply_order_id,
                reject_code = code as i32,
                "cancel rejected: validation"
            );
            record_grpc_request("cancel", "rejected_validation");
            return Ok(Response::new(order_cmd_reply(
                reply_order_id,
                pb::OrderCmdAck::Rejected as i32,
                Some(code as i32),
            )));
        }

        let command_id_for_log = req.command_id.clone();
        let cmd = Command::CancelOrder(CancelOrderCommand {
            command_id: req.command_id,
            order_id: req.order_id,
            timestamp: req.timestamp,
        });

        match self.dispatcher.submit(cmd).await {
            Ok(()) => {
                debug!(
                    rpc = "Cancel",
                    command_id = %command_id_for_log,
                    order_id = %reply_order_id,
                    "cancel queued for engine"
                );
                record_grpc_request("cancel", "queued");
                Ok(Response::new(order_cmd_reply(
                    reply_order_id,
                    pb::OrderCmdAck::Queued as i32,
                    None,
                )))
            }
            Err(e @ DispatchError::ChannelClosed) => {
                warn!(
                    rpc = "Cancel",
                    command_id = %command_id_for_log,
                    order_id = %reply_order_id,
                    error = %e,
                    "cancel rejected: engine unavailable"
                );
                record_grpc_request("cancel", "rejected_engine_unavailable");
                Ok(Response::new(order_cmd_reply(
                    reply_order_id,
                    pb::OrderCmdAck::Rejected as i32,
                    Some(pb::CmdErr::CancelEngineUnavailable as i32),
                )))
            }
        }
    }
}

pub fn new(
    dispatcher: Dispatcher,
    symbol: String,
    max_decoding_message_bytes: usize,
    max_encoding_message_bytes: usize,
) -> pb::order_cmd_service_server::OrderCmdServiceServer<OrderCmdService> {
    let svc = OrderCmdService::new(dispatcher, symbol);
    pb::order_cmd_service_server::OrderCmdServiceServer::new(svc)
        .max_decoding_message_size(max_decoding_message_bytes)
        .max_encoding_message_size(max_encoding_message_bytes)
}
