use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TaurusError {
    #[error("failed to read configuration file: {0:#?}")]
    InvalidConfig(anyhow::Error),

    #[error("failed to open identity file: {0}")]
    NoIdentity(io::Error),

    #[error("failed parse certificate: {0:#?}")]
    InvalidCertificate(#[from] native_tls::Error),

    #[error("failed to bind: {0}")]
    BindFailed(io::Error),

    #[error("could not read the stream")]
    StreamReadFailed(io::Error),

    #[error("could not write to the stream")]
    StreamWriteFailed(io::Error),
}
