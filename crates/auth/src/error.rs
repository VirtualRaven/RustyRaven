use std::convert::Infallible;

use thiserror::Error;
use webauthn_rs::prelude::Uuid;

#[derive(Error, Debug)]
pub enum WebauthnError {
    #[error("unknown webauthn error")]
    Unknown,
    #[error("Corrupt Session")]
    CorruptSession,
    #[error("Invalid terminal challenge")]
    InvalidTerminalChallenge,
    #[error("No terminal challenge presented")]
    NoTerminalChallenge,
    #[error("Invalid challenge Uuid")]
    InvalidChallengeUuid,
    #[error("User Not Found")]
    UserNotFound,
    #[error("Invalid username")]
    InvalidUsername,
    #[error("User Has No Credentials")]
    UserHasNoCredentials,
    #[error("Deserialising Session failed: {0}")]
    InvalidSessionState(#[from] tower_sessions::session::Error),
    #[error("Webauthn-rs error: {0}")]
    Webauthn(#[from] webauthn_rs::prelude::WebauthnError),
    #[error("Database error: {0}")]
    Db(#[from] sjf_db::Error),
    #[error("Encoding error")]
    Encoding(#[from] minicbor_serde::error::EncodeError<Infallible>),
    #[error("Decoding error: {0}")]
    Decoding(#[from] minicbor_serde::error::DecodeError),
}