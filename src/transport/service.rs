use std::fs;
use std::net::SocketAddr;
use std::time::Duration;

use anyhow::Context;
use tracing::info;

use crate::transport::dispatcher::Dispatcher;
use crate::transport::orders_service;
use crate::transport::settings::GrpcServeOptions;

pub async fn serve(
    dispatcher: Dispatcher,
    symbol: String,
    addr: SocketAddr,
    options: GrpcServeOptions,
) -> anyhow::Result<()> {
    options.validate()?;

    let mut server = tonic::transport::Server::builder();

    if let Some(tls_cfg) = options.tls.as_ref() {
        let cert = fs::read(&tls_cfg.cert_path).with_context(|| {
            format!(
                "failed to read gRPC TLS certificate from {}",
                tls_cfg.cert_path
            )
        })?;
        let key = fs::read(&tls_cfg.key_path).with_context(|| {
            format!(
                "failed to read gRPC TLS private key from {}",
                tls_cfg.key_path
            )
        })?;
        let identity = tonic::transport::Identity::from_pem(cert, key);
        let mut tls_config = tonic::transport::ServerTlsConfig::new().identity(identity);

        if let Some(ca_path) = tls_cfg.client_ca_path.as_deref() {
            let ca_pem = fs::read(ca_path)
                .with_context(|| format!("failed to read gRPC mTLS client CA from {ca_path}"))?;
            let client_ca = tonic::transport::Certificate::from_pem(ca_pem);
            tls_config = tls_config
                .client_ca_root(client_ca)
                .client_auth_optional(tls_cfg.client_auth_optional);
        }

        server = server
            .tls_config(tls_config)
            .map_err(|e| anyhow::anyhow!("invalid gRPC TLS configuration: {e}"))?;

        let mtls = tls_cfg.client_ca_path.is_some();
        info!(
            port = addr.port(),
            %addr,
            tls = true,
            mtls,
            "gRPC server listening"
        );
    } else {
        info!(port = addr.port(), %addr, "gRPC server listening (plaintext)");
    }

    server
        .timeout(Duration::from_secs(options.request_timeout_secs))
        .concurrency_limit_per_connection(options.concurrency_limit_per_connection)
        .trace_fn(|req| {
            tracing::debug_span!(
                "grpc.request",
                method = %req.uri().path(),
            )
        })
        .add_service(orders_service::new(
            dispatcher,
            symbol,
            options.max_decoding_message_bytes,
            options.max_encoding_message_bytes,
        ))
        .serve(addr)
        .await?;

    Ok(())
}
