use anyhow::Result;
use tracing::{info, warn};

use super::client::BlueskyClient;
use super::session::{self};

pub enum AuthResult {
    Success(String),
    NeedsLogin,
}

pub async fn try_restore_session(client: &BlueskyClient) -> AuthResult {
    match session::load_session() {
        Ok(Some(session_data)) => {
            info!("Found saved session for {}", session_data.handle);
            match client
                .restore_session(&session_data.access_jwt, &session_data.refresh_jwt, &session_data.did, &session_data.handle)
                .await
            {
                Ok(_) => {
                    info!("Session restored for {}", session_data.handle);
                    AuthResult::Success(session_data.handle)
                }
                Err(e) => {
                    warn!("Failed to restore session: {}", e);
                    AuthResult::NeedsLogin
                }
            }
        }
        Ok(None) => {
            info!("No saved session found");
            AuthResult::NeedsLogin
        }
        Err(e) => {
            warn!("Error loading session: {}", e);
            AuthResult::NeedsLogin
        }
    }
}

pub async fn login_with_app_password(
    client: &BlueskyClient,
    identifier: &str,
    password: &str,
) -> Result<String> {
    let session_data = client.login_app_password(identifier, password).await?;
    Ok(session_data.handle)
}

pub fn logout() -> Result<()> {
    session::clear_session()?;
    Ok(())
}
