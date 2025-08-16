use std::collections::HashMap;
use std::sync::Arc;
use webauthn_rs::prelude::*;

#[derive(Clone)]
pub struct AuthState {
    pub webauthn: Arc<Webauthn>,
}

impl AuthState {
    pub fn new() -> Self {
        let rp_id = dotenvy::var("AUTH_RP_ID").expect("AUTH_RP_ID must be set");
        let rp_origin = Url::parse(&dotenvy::var("AUTH_RP_URL").expect("AUTH_RP_URL must be set"))
            .expect("Invalid URL");
        let builder = WebauthnBuilder::new(&rp_id, &rp_origin).expect("Invalid configuration");

        let builder = builder.rp_name("Axum Webauthn-rs");
        let webauthn = Arc::new(builder.build().expect("Invalid passkey configuration"));

        AuthState { webauthn }
    }
}
