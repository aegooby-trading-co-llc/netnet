use std::{sync::Arc, time::SystemTime};

use anyhow::Result;
use quinn::{ClientConfig, ServerConfig};
use rcgen::generate_simple_self_signed;
use rustls::{
    client::{ServerCertVerified, ServerCertVerifier},
    Certificate, ClientConfig as TlsConfig, Error, PrivateKey, ServerName,
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

pub fn configure_client() -> ClientConfig {
    let crypto = TlsConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    ClientConfig::new(Arc::new(crypto))
}

pub async fn get_server_config() -> Result<ServerConfig> {
    let cert = generate_simple_self_signed(vec![
        "0.0.0.0".into(),
        "localhost".into(),
        "127.0.0.1".into(),
    ])?;

    let server_config = ServerConfig::with_single_cert(
        vec![Certificate(cert.serialize_der()?)],
        PrivateKey(cert.serialize_private_key_der()),
    )?;
    Ok(server_config)
}
