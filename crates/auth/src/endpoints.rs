use crate::error::WebauthnError;
use crate::state::AuthState;
use axum::routing::get;
use tower_sessions::Session;
use tracing::{info,error};

use webauthn_rs::prelude::*;


pub use tower_sessions::Session as TowerSession;
pub use axum::extract::Extension;

pub struct User {
    id: Uuid,
    passkeys: Vec<Passkey>
}

impl User {
    fn credentials(&self) -> Vec<CredentialID>
    {
        self.passkeys.iter().map(|ps|ps.cred_id().clone()).collect()
    }

    fn passkeys(&self) -> &[Passkey]
    {
        &self.passkeys
    }
    
    async fn get_passkeys(id: &Uuid) -> Result<Vec<Passkey>, WebauthnError>
    {
        let keys = sjf_db::auth::get_keys(id).await?;
        let keys: Result<Vec<Passkey>, WebauthnError> = keys
        .iter()
        .map(|pk| {
            minicbor_serde::from_slice(&pk)
            .map_err(|e| e.into())
        } )
        .collect();
        keys
    }

    async fn lookup_name(username: &str) -> Result<Option<Self>, WebauthnError>
    {
        match sjf_db::auth::lookup_name(username).await?
        {
            Some(id) => {

                Ok(Some(User {
                    id,
                    passkeys: Self::get_passkeys(&id).await?
                }))
            },
            None => Ok(None)
        }
    } 

    pub(crate) async fn lookup_id(id: &Uuid) -> Result<Option<String>, WebauthnError>
    {
        sjf_db::auth::lookup_id(id).await.map_err(|e| e.into())
    }

    async fn add(id: Uuid, name: String, passkey: Passkey) -> Result<(), WebauthnError >
    {
        let keyid : Vec<u8> = passkey.cred_id().clone().into();
        let passkey : Vec<u8> = minicbor_serde::to_vec( passkey)?;

        sjf_db::auth::add(id,name,keyid,passkey).await.map_err(|e| e.into())

    }

    async fn update_credential(uuid: Uuid,auth_result: &AuthenticationResult ) -> Result<(), WebauthnError>
    {
        let keyid: Vec<u8> = auth_result.cred_id().clone().into();
        let (tx,rawkey) = sjf_db::auth::begin_passkey_update(uuid.clone(),keyid.clone()  ).await?;
        let mut passkey: Passkey=  minicbor_serde::from_slice(&rawkey)?;

        if let Some(true) = passkey.update_credential(auth_result)
        {
            let passkey : Vec<u8> = minicbor_serde::to_vec( passkey)?;
            sjf_db::auth::complete_passkey_update(tx, uuid, keyid, passkey).await?;
        }
        
        Ok(())
    }

}

pub async fn terminal_challenge(
    session: Session,
    username: String,
) -> Result<(), WebauthnError> {

    let name_len = username.len();

    if name_len == 0 || name_len > 100
    {
        return Err(WebauthnError::InvalidUsername);
    }

    let challenge = Uuid::new_v4();
    info!("Registration request '{}', code '{}'",username,challenge);
    Ok(session.insert("terminal_challenge", (username,challenge)).await?)
}

pub async fn start_register(
    Extension(app_state): Extension<AuthState>,
    session: Session,
    username: String,
    terminal_challenge: String
) -> Result<CreationChallengeResponse, WebauthnError> {

    info!("Got '{}'",terminal_challenge);

    let challenge = session.get::<(String,Uuid)>("terminal_challenge").await;
    let _ = session.remove::<(String,Uuid)>("terminal_challenge").await;


    match challenge?
    {
        Some((name,challenge)) => {
            let uuid = Uuid::try_parse(&terminal_challenge).map_err(|_| WebauthnError::InvalidChallengeUuid  )?;
            if challenge == uuid && username == name
            {
                Ok(())
            }
            else {
                Err(WebauthnError::InvalidTerminalChallenge)
            }

        }
        None => Err(WebauthnError::NoTerminalChallenge)
    }?;

    let user= User::lookup_name(&username).await?;
    let user_unique_id = user.as_ref().map(|f| f.id.clone()).unwrap_or( Uuid::new_v4() );


    // Remove any previous registrations that may have occured from the session.
    let _ = session.remove_value("reg_state").await;

    let exclude_credentials = user.map(|u| u.credentials() );

    let res = match app_state.webauthn.start_passkey_registration(
        user_unique_id,
        &username,
        &username,
        exclude_credentials,
    ) {
        Ok((ccr, reg_state)) => {
            session
                .insert("reg_state", (username, user_unique_id, reg_state))
                .await
                .expect("Failed to insert");
            info!("Registration Successful!");
            ccr
        }
        Err(e) => {
            info!("challenge_register -> {:?}", e);
            return Err(WebauthnError::Unknown);
        }
    };
    Ok(res)
}


pub async fn finish_register(
    Extension(app_state): Extension<AuthState>,
    session: Session,
    reg: RegisterPublicKeyCredential,
) -> Result<(), WebauthnError> {

    let (username, user_unique_id, reg_state) = match session.get("reg_state").await? {
        Some((username, user_unique_id, reg_state)) => (username, user_unique_id, reg_state),
        None => {
            error!("Failed to get session");
            return Err(WebauthnError::CorruptSession);
        }
    };

    let _ = session.remove_value("reg_state").await;

    let res = match app_state
        .webauthn
        .finish_passkey_registration(&reg, &reg_state)
    {
        Ok(passkey) => {
            User::add(user_unique_id, username, passkey).await
        }
        Err(e) => {
            Err(e.into())
        }
    };
    res

}


pub async fn start_authentication(
    Extension(app_state): Extension<AuthState>,
    session: Session,
    username: String,
) -> Result<RequestChallengeResponse, WebauthnError> {

    let _ = session.remove_value("auth_state").await;

    info!("Authentication attempt '{}'",username);
    let user  = User::lookup_name(&username).await?.ok_or(WebauthnError::UserNotFound)?;
    let allow_credentials = user.passkeys();
    info!("Lookup user '{}'={} with {}passkeys",username,user.id,allow_credentials.len());

    let res = match app_state
        .webauthn
        .start_passkey_authentication(allow_credentials)
    {
        Ok((rcr, auth_state)) => {
            session
                .insert("auth_state", (user.id, auth_state))
                .await
                .expect("Failed to insert");
            rcr
        }
        Err(e) => {
            info!("challenge_authenticate -> {:?}", e);
            return Err(WebauthnError::Unknown);
        }
    };
    Ok(res)
}


pub async fn finish_authentication(
    Extension(app_state): Extension<AuthState>,
    session: Session,
    auth: PublicKeyCredential,
) -> Result<Uuid, WebauthnError> {
    let (user_unique_id, auth_state): (Uuid, PasskeyAuthentication) = session
        .get("auth_state")
        .await?
        .ok_or(WebauthnError::CorruptSession)?;

    let _ = session.remove_value("auth_state").await;

    let res = match app_state
        .webauthn
        .finish_passkey_authentication(&auth, &auth_state)
    {
        Ok(auth_result) => {
            if auth_result.needs_update()
            {
                User::update_credential(user_unique_id.clone(),&auth_result).await?;
            }
            Ok(user_unique_id)
        }
        Err(e) => {
            info!("challenge_register -> {:?}", e);
            Err(e.into())
        }
    };
    info!("Authentication Successful!");
    res
}