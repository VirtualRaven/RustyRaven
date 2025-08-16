pub mod axum;
pub mod state;
pub mod endpoints;
mod error;

pub use error::WebauthnError as Error;
use ::axum::async_trait;
use axum_login::{AuthUser, AuthnBackend, UserId};
use webauthn_rs::prelude::{ Uuid};


#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
}

impl AuthUser for User {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.name.as_bytes()
    }
}

#[derive(Clone, Default)]
pub struct Backend {
}

#[derive(Clone)]
pub struct Credentials {
    pub uuid: Uuid,
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = crate::Error;

    async fn authenticate(
        &self,
        Credentials { uuid }: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        self.get_user(&uuid).await
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        match crate::endpoints::User::lookup_id(&user_id).await?
        {
            Some(name) => {
                Ok(Some(
                    User {
                        id: user_id.clone(),
                        name
                    }
                ))
            },
            None => Ok(None)
        }
        
    }
}
