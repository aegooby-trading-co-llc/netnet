use std::{sync::Arc, time::SystemTime};

use anyhow::Result;
use quinn::{ClientConfig, ServerConfig};
use rcgen::Certificate as SelfSignedCertificate;
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

pub fn quic_client_config(cert: &SelfSignedCertificate) -> Result<ClientConfig> {
    let crypto = TlsConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_single_cert(vec![], PrivateKey(cert.serialize_private_key_der()))?;

    Ok(ClientConfig::new(Arc::new(crypto)))
}

pub fn quic_server_config(cert: &SelfSignedCertificate) -> Result<ServerConfig> {
    let server_config = ServerConfig::with_single_cert(
        vec![Certificate(cert.serialize_der()?)],
        PrivateKey(cert.serialize_private_key_der()),
    )?;
    Ok(server_config)
}
