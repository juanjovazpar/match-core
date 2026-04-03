//! Prometheus counters for gRPC ingress (`metrics` crate).

pub(crate) fn record_grpc_request(rpc: &'static str, outcome: &'static str) {
    metrics::counter!(
        "match_core_grpc_requests_total",
        "rpc" => rpc,
        "outcome" => outcome
    )
    .increment(1);
}
