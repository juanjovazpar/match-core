//! Validation of `OrderCmdService` gRPC payloads before dispatch.

use crate::shared::command::{TimeInForce, Type, Side};
use crate::transport::proto::pb;

pub struct Validator {
    symbol: String,
}

impl Validator {
    pub fn new(symbol: String) -> Self {
        Self { symbol }
    }

    fn validate_symbol(&self, symbol: &str) -> bool {
        symbol == self.symbol
    }

    pub fn validate_submit_order_command(
        &self,
        req: &pb::SubmitOrderCmdRequest,
    ) -> Result<(Side, Type, TimeInForce), pb::CmdErr> {
        if req.side == pb::Side::Unspecified as i32 {
            return Err(pb::CmdErr::SubmitSideUnspecified);
        }
        if req.order_type == pb::Type::Unspecified as i32 {
            return Err(pb::CmdErr::SubmitOrderTypeUnspecified);
        }
        if !req.price.is_finite() || req.price <= 0.0 {
            return Err(pb::CmdErr::SubmitPriceInvalid);
        }
        if !req.quantity.is_finite() || req.quantity <= 0.0 {
            return Err(pb::CmdErr::SubmitQtyInvalid);
        }
        if req.command_id.is_empty() {
            return Err(pb::CmdErr::SubmitCmdIdEmpty);
        }
        if req.order_id.is_empty() {
            return Err(pb::CmdErr::SubmitOrderIdEmpty);
        }
        if req.user_id.is_empty() {
            return Err(pb::CmdErr::SubmitUserIdEmpty);
        }
        if req.symbol.is_empty() {
            return Err(pb::CmdErr::SubmitSymbolEmpty);
        }
        if req.timestamp == 0 {
            return Err(pb::CmdErr::SubmitTimestampZero);
        }
        if !self.validate_symbol(&req.symbol) {
            return Err(pb::CmdErr::SubmitSymbolShardMismatch);
        }

        let side = match pb::Side::try_from(req.side) {
            Ok(pb::Side::Buy) => Side::Buy,
            Ok(pb::Side::Sell) => Side::Sell,
            Ok(pb::Side::Unspecified) | Err(_) => {
                return Err(pb::CmdErr::SubmitSideInvalid);
            }
        };
        let order_type = match pb::Type::try_from(req.order_type) {
            Ok(pb::Type::Limit) => Type::Limit,
            Ok(pb::Type::Market) => Type::Market,
            Ok(pb::Type::Unspecified) | Err(_) => {
                return Err(pb::CmdErr::SubmitOrderTypeInvalid);
            }
        };

        if req.time_in_force == pb::TimeInForce::Unspecified as i32 {
            return Err(pb::CmdErr::SubmitOrderTimeInForceUnspecified);
        }
        let time_in_force = match pb::TimeInForce::try_from(req.time_in_force) {
            Ok(pb::TimeInForce::Gtc) => TimeInForce::GTC,
            Ok(pb::TimeInForce::Ioc) => TimeInForce::IOC,
            Ok(pb::TimeInForce::Fok) => TimeInForce::FOK,
            _ => return Err(pb::CmdErr::SubmitOrderTimeInForceInvalid),
        };

        Ok((side, order_type, time_in_force))
    }

    pub fn validate_cancel_order_command(
        &self,
        req: &pb::CancelOrderCmdRequest,
    ) -> Result<(), pb::CmdErr> {
        if req.command_id.is_empty() {
            return Err(pb::CmdErr::CancelCmdIdEmpty);
        }
        if req.order_id.is_empty() {
            return Err(pb::CmdErr::CancelOrderIdEmpty);
        }
        if req.timestamp == 0 {
            return Err(pb::CmdErr::CancelTimestampZero);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::proto::pb;

    fn make_validator() -> Validator {
        Validator::new("BTC-USD".into())
    }

    fn valid_submit() -> pb::SubmitOrderCmdRequest {
        pb::SubmitOrderCmdRequest {
            command_id: "cmd-1".into(),
            order_id: "ord-1".into(),
            user_id: "user-1".into(),
            symbol: "BTC-USD".into(),
            side: pb::Side::Buy as i32,
            order_type: pb::Type::Limit as i32,
            price: 42_000.0,
            quantity: 0.25,
            timestamp: 1,
            time_in_force: pb::TimeInForce::Gtc as i32,
        }
    }

    #[test]
    fn submit_ok_for_valid_limit_order() {
        let v = make_validator();
        let req = valid_submit();
        let (side, ty, tif) = v.validate_submit_order_command(&req).unwrap();
        assert!(matches!(side, Side::Buy));
        assert!(matches!(ty, Type::Limit));
        assert!(matches!(tif, TimeInForce::GTC));
    }

    #[test]
    fn submit_rejects_symbol_mismatch() {
        let v = make_validator();
        let mut req = valid_submit();
        req.symbol = "ETH-USD".into();
        assert_eq!(
            v.validate_submit_order_command(&req).unwrap_err(),
            pb::CmdErr::SubmitSymbolShardMismatch
        );
    }

    #[test]
    fn submit_rejects_unspecified_side() {
        let v = make_validator();
        let mut req = valid_submit();
        req.side = pb::Side::Unspecified as i32;
        assert_eq!(
            v.validate_submit_order_command(&req).unwrap_err(),
            pb::CmdErr::SubmitSideUnspecified
        );
    }

    #[test]
    fn submit_rejects_zero_timestamp() {
        let v = make_validator();
        let mut req = valid_submit();
        req.timestamp = 0;
        assert_eq!(
            v.validate_submit_order_command(&req).unwrap_err(),
            pb::CmdErr::SubmitTimestampZero
        );
    }

    #[test]
    fn cancel_ok_when_ids_and_timestamp_present() {
        let v = make_validator();
        let req = pb::CancelOrderCmdRequest {
            command_id: "c1".into(),
            order_id: "o1".into(),
            timestamp: 1,
        };
        assert!(v.validate_cancel_order_command(&req).is_ok());
    }

    #[test]
    fn cancel_rejects_zero_timestamp() {
        let v = make_validator();
        let req = pb::CancelOrderCmdRequest {
            command_id: "c1".into(),
            order_id: "o1".into(),
            timestamp: 0,
        };
        assert_eq!(
            v.validate_cancel_order_command(&req).unwrap_err(),
            pb::CmdErr::CancelTimestampZero
        );
    }
}
