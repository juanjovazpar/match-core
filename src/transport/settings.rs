//! Ingress server options owned by the transport layer (no dependency on `config`).

#[derive(Debug, Clone)]
pub struct GrpcTlsOptions {
    pub cert_path: String,
    pub key_path: String,
    pub client_ca_path: Option<String>,
    pub client_auth_optional: bool,
}

#[derive(Debug, Clone)]
pub struct GrpcServeOptions {
    pub request_timeout_secs: u64,
    pub concurrency_limit_per_connection: usize,
    pub max_decoding_message_bytes: usize,
    pub max_encoding_message_bytes: usize,
    pub tls: Option<GrpcTlsOptions>,
}

impl GrpcServeOptions {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.request_timeout_secs == 0 {
            anyhow::bail!("grpc.request_timeout_secs must be greater than 0");
        }
        if self.concurrency_limit_per_connection == 0 {
            anyhow::bail!("grpc.concurrency_limit_per_connection must be greater than 0");
        }
        if self.max_decoding_message_bytes == 0 || self.max_encoding_message_bytes == 0 {
            anyhow::bail!(
                "grpc max_decoding_message_bytes and max_encoding_message_bytes must be greater than 0"
            );
        }
        if let Some(tls) = &self.tls {
            if let Some(ca) = tls.client_ca_path.as_deref() {
                if ca.is_empty() {
                    anyhow::bail!("grpc TLS client_ca_path must not be empty when set");
                }
            }
        }
        Ok(())
    }
}
