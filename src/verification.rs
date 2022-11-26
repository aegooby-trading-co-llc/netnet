use std::{sync::Arc, time::SystemTime};

use anyhow::Result;
use quinn::ServerConfig;
use rcgen::generate_simple_self_signed;
use rustls::{
    client::{ServerCertVerified, ServerCertVerifier},
    Certificate, Error, PrivateKey, ServerName,
};

struct SkipServerVerification;

impl SkipServerVerification {
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }
}

pub async fn get_server_config() -> Result<ServerConfig> {
    let names = vec!["0.0.0.0".into()];

    let cert = generate_simple_self_signed(names)?;
    let cert_der = cert.serialize_der()?;
    let priv_key = cert.serialize_private_key_der();

    let certificate = Certificate(cert_der);
    let private_key = PrivateKey(priv_key);

    let cert_chain = vec![certificate];

    let server_config = ServerConfig::with_single_cert(cert_chain, private_key)?;
    return Ok(server_config);
}
