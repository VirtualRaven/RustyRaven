use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use time::Duration;
use tower_sessions::{self, MemoryStore, SessionManagerLayer, cookie::Key};

use axum_login::{AuthManagerLayer, AuthManagerLayerBuilder};

pub async fn protect_authenticated_routes(
    auth_session: axum_login::AuthSession<crate::Backend>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    if path.contains("api/auth/") {
        if auth_session.user.is_none() {
            return StatusCode::FORBIDDEN.into_response();
        }
    }

    let response = next.run(request).await;
    response
}

pub fn create_auth_layer()
-> AuthManagerLayer<crate::Backend, MemoryStore, tower_sessions::service::SignedCookie> {
    // Session layer.
    let session_store = MemoryStore::default();

    // Generate a cryptographic key to sign the session cookie.
    let key = Key::generate();

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(tower_sessions::Expiry::OnInactivity(Duration::days(1)))
        .with_signed(key);

    // Auth service.
    let backend = crate::Backend::default();

    AuthManagerLayerBuilder::new(backend, session_layer).build()
}

