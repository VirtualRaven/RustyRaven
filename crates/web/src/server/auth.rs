use dioxus::logger::tracing::{info, warn};
use dioxus::prelude::server_fn::error::NoCustomError;
use dioxus::prelude::server_fn::ServerFn;
use dioxus::prelude::*;
use dioxus::prelude::{server, ServerFnError};
use webauthn_rs_proto::{AuthenticatorAssertionResponseRaw, RegisterPublicKeyCredential};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum AuthApiError {
    #[error("Failed to get WebAuthn state")]
    Extension,
    #[error("Auth failure: {0}")]
    Auth(String),
    #[error("ExtractionFailure")]
    ExtractionFailure,
    #[error("Login failure")]
    LoginBackend,
}

use std::str::FromStr;

impl FromStr for AuthApiError {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Failed to get WebAuthn state" => Ok(AuthApiError::Extension),
            "ExtractionFailure" => Ok(AuthApiError::ExtractionFailure),
            "Login failure" => Ok(AuthApiError::LoginBackend),
            s if s.starts_with("Auth failure:") => Ok(AuthApiError::Auth(s.into())),
            _ => Err(()),
        }
    }
}

#[cfg(feature = "server")]
impl From<axum::extract::rejection::ExtensionRejection> for AuthApiError {
    fn from(_: axum::extract::rejection::ExtensionRejection) -> Self {
        Self::Extension
    }
}

#[cfg(feature = "server")]
impl From<sjf_auth::Error> for AuthApiError {
    fn from(e: sjf_auth::Error) -> Self {
        let error_text = e.to_string();
        Self::Auth(error_text)
    }
}

#[cfg(feature = "server")]
impl From<(axum::http::StatusCode, &str)> for AuthApiError {
    fn from(_: (axum::http::StatusCode, &str)) -> Self {
        Self::ExtractionFailure
    }
}

#[cfg(feature = "server")]
impl From<axum_login::Error<sjf_auth::Backend>> for AuthApiError {
    fn from(_: axum_login::Error<sjf_auth::Backend>) -> Self {
        Self::LoginBackend
    }
}

#[cfg(feature = "server")]
pub mod def {

    use super::*;
    use axum_login::AuthSession;
    use sjf_auth::Backend;
    use web_sys::PublicKeyCredential;

    pub async fn get_session() -> Result<axum_login::AuthSession<sjf_auth::Backend>, AuthApiError> {
        Ok(extract().await?)
    }

    pub async fn get_state() -> Result<axum::Extension<sjf_auth::state::AuthState>, AuthApiError> {
        Ok(extract().await?)
    }

    pub async fn get_tower_session() -> Result<sjf_auth::endpoints::TowerSession, AuthApiError> {
        Ok(extract().await?)
    }
}

fn to_err<T, E1>(r: Result<T, E1>) -> Result<T, ServerFnError<AuthApiError>>
where
    E1: Into<AuthApiError>,
{
    r.map_err(|e| {
        let api_error: AuthApiError = e.into();
        api_error.into()
    })
}

#[server(endpoint="passkey/authentication/terminal",input=dioxus::prelude::server_fn::codec::PostUrl)]
pub async fn terminal_challenge(username: String) -> Result<(), ServerFnError<AuthApiError>> {
    let r =
        sjf_auth::endpoints::terminal_challenge(def::get_tower_session().await?, username).await;
    to_err(r)
}

#[server(endpoint="passkey/registration/start",input=dioxus::prelude::server_fn::codec::PostUrl)]
pub async fn start_registration(
    username: String,
    terminal_challange: String,
) -> Result<webauthn_rs_proto::CreationChallengeResponse, ServerFnError<AuthApiError>> {
    let r = sjf_auth::endpoints::start_register(
        def::get_state().await?,
        def::get_tower_session().await?,
        username,
        terminal_challange,
    )
    .await;
    to_err(r)
}
#[server(endpoint="passkey/registration/finish",input=dioxus::prelude::server_fn::codec::PostUrl)]
pub async fn finish_registration(
    reg: webauthn_rs_proto::RegisterPublicKeyCredential,
) -> Result<(), ServerFnError<AuthApiError>> {
    let r = sjf_auth::endpoints::finish_register(
        def::get_state().await?,
        def::get_tower_session().await?,
        reg,
    )
    .await;
    to_err(r)
}
#[server(endpoint="passkey/authentication/start",input=dioxus::prelude::server_fn::codec::PostUrl)]
pub async fn start_authentication(
    username: String,
) -> Result<webauthn_rs_proto::RequestChallengeResponse, ServerFnError<AuthApiError>> {
    let r = sjf_auth::endpoints::start_authentication(
        def::get_state().await?,
        def::get_tower_session().await?,
        username,
    )
    .await;
    to_err(r)
}
#[server(endpoint="passkey/authentication/finish",input=dioxus::prelude::server_fn::codec::PostUrl)]
pub async fn finish_authentication(
    key: webauthn_rs_proto::PublicKeyCredential,
) -> Result<(), ServerFnError<AuthApiError>> {
    let uuid = to_err(
        sjf_auth::endpoints::finish_authentication(
            def::get_state().await?,
            def::get_tower_session().await?,
            key,
        )
        .await,
    )?;
    let mut session = def::get_session().await?;
    let user = match to_err(session.authenticate(sjf_auth::Credentials { uuid }).await)? {
        Some(user) => user,
        None => return Err(ServerFnError::ServerError("invalid".into())),
    };
    to_err(session.login(&user).await)
}

#[server(endpoint="is_authenticated",input=dioxus::prelude::server_fn::codec::GetUrl)]
pub async fn is_authenticated() -> Result<bool, ServerFnError> {
    let session = def::get_session().await?;
    Ok(session.user.is_some())
}
