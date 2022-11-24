use std::sync::Arc;
use rcgen::generate_simple_self_signed;
use quinn::{ServerConfig};
use anyhow::Error; 

struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl rustls::client::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}

pub async fn get_server_config() -> Result<ServerConfig, Error> { 
    let names = vec!["0.0.0.0".into()];

    let cert = generate_simple_self_signed(names)?;
    let cert_der = cert.serialize_der()?;
    let priv_key = cert.serialize_private_key_der();

    tokio::fs::write("tmp.cert", &cert_der).await?;

    let certificate = rustls::Certificate(cert_der);
    let private_key = rustls::PrivateKey(priv_key);

    let cert_chain = vec![certificate];

    let server_config = ServerConfig::with_single_cert(cert_chain, private_key)?;
    return Ok(server_config); 

}
