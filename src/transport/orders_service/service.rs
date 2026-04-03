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

#[derive(Clone, Copy)]
enum OrderCmdRpc {
    Submit,
    Cancel,
}

impl OrderCmdRpc {
    const fn trace_rpc(self) -> &'static str {
        match self {
            Self::Submit => "Submit",
            Self::Cancel => "Cancel",
        }
    }

    const fn metric_rpc(self) -> &'static str {
        match self {
            Self::Submit => "submit",
            Self::Cancel => "cancel",
        }
    }

    fn engine_unavailable_err(self) -> pb::CmdErr {
        match self {
            Self::Submit => pb::CmdErr::SubmitEngineUnavailable,
            Self::Cancel => pb::CmdErr::CancelEngineUnavailable,
        }
    }

    fn command_queue_timeout_err(self) -> pb::CmdErr {
        match self {
            Self::Submit => pb::CmdErr::SubmitCommandQueueTimeout,
            Self::Cancel => pb::CmdErr::CancelCommandQueueTimeout,
        }
    }
}

fn reject_validation(
    rpc: OrderCmdRpc,
    command_id: &str,
    order_id: &str,
    code: pb::CmdErr,
) -> Response<pb::OrderCmdReply> {
    warn!(
        rpc = rpc.trace_rpc(),
        command_id = %command_id,
        order_id = %order_id,
        reject_code = code as i32,
        "{} rejected: validation",
        rpc.metric_rpc(),
    );
    record_grpc_request(rpc.metric_rpc(), "rejected_validation");
    Response::new(order_cmd_reply(
        order_id.to_string(),
        pb::OrderCmdAck::Rejected as i32,
        Some(code as i32),
    ))
}

async fn dispatch_command(
    dispatcher: &Dispatcher,
    cmd: Command,
    rpc: OrderCmdRpc,
    command_id: &str,
    order_id: &str,
) -> Response<pb::OrderCmdReply> {
    match dispatcher.submit(cmd).await {
        Ok(()) => {
            debug!(
                rpc = rpc.trace_rpc(),
                command_id = %command_id,
                order_id = %order_id,
                "{} queued for engine",
                rpc.metric_rpc(),
            );
            record_grpc_request(rpc.metric_rpc(), "queued");
            Response::new(order_cmd_reply(
                order_id.to_string(),
                pb::OrderCmdAck::Queued as i32,
                None,
            ))
        }
        Err(e @ DispatchError::ChannelClosed) => {
            warn!(
                rpc = rpc.trace_rpc(),
                command_id = %command_id,
                order_id = %order_id,
                error = %e,
                "{} rejected: engine unavailable",
                rpc.metric_rpc(),
            );
            record_grpc_request(rpc.metric_rpc(), "rejected_engine_unavailable");
            Response::new(order_cmd_reply(
                order_id.to_string(),
                pb::OrderCmdAck::Rejected as i32,
                Some(rpc.engine_unavailable_err() as i32),
            ))
        }
        Err(e @ DispatchError::EnqueueTimeout) => {
            warn!(
                rpc = rpc.trace_rpc(),
                command_id = %command_id,
                order_id = %order_id,
                error = %e,
                "{} rejected: command queue timeout",
                rpc.metric_rpc(),
            );
            record_grpc_request(rpc.metric_rpc(), "rejected_command_queue_timeout");
            Response::new(order_cmd_reply(
                order_id.to_string(),
                pb::OrderCmdAck::Rejected as i32,
                Some(rpc.command_queue_timeout_err() as i32),
            ))
        }
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
            validator: Validator::new(symbol),
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
        let rpc = OrderCmdRpc::Submit;

        let (side, order_type, time_in_force) =
            match self.validator.validate_submit_order_command(&req) {
                Ok(v) => v,
                Err(code) => {
                    return Ok(reject_validation(
                        rpc,
                        &req.command_id,
                        &reply_order_id,
                        code,
                    ));
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

        Ok(dispatch_command(
            &self.dispatcher,
            cmd,
            rpc,
            &command_id_for_log,
            &reply_order_id,
        )
        .await)
    }

    async fn cancel(
        &self,
        request: Request<pb::CancelOrderCmdRequest>,
    ) -> Result<Response<pb::OrderCmdReply>, Status> {
        let req = request.into_inner();
        let reply_order_id = req.order_id.clone();
        let rpc = OrderCmdRpc::Cancel;

        if let Err(code) = self.validator.validate_cancel_order_command(&req) {
            return Ok(reject_validation(
                rpc,
                &req.command_id,
                &reply_order_id,
                code,
            ));
        }

        let command_id_for_log = req.command_id.clone();
        let cmd = Command::CancelOrder(CancelOrderCommand {
            command_id: req.command_id,
            order_id: req.order_id,
            timestamp: req.timestamp,
        });

        Ok(dispatch_command(
            &self.dispatcher,
            cmd,
            rpc,
            &command_id_for_log,
            &reply_order_id,
        )
        .await)
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
