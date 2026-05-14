use super::wrapper::AgentWrapper;
use bsky_sdk::agent::config::FileStore;
use std::fs::remove_file;
use std::error::Error;

// WARN: This comes before the bskyclient wrapper
pub async fn login(
    client: &AgentWrapper,
    identifier: &String,
    password: &String,
) -> Result<String, Box<dyn Error>> {
    let session = client.agent.login(identifier, password).await?;
    let retrieved_this_time = client.agent.to_config().await;
    retrieved_this_time
        .save(&FileStore::new(yet_again())).await?;

    // WARN: Confusing because the client can't do anything until this store has been created
    Ok(session.handle.to_string())
}

pub fn yet_again() -> std::path::PathBuf {
    dirs::config_dir()
        .expect("Couldn't retrive from your config directory")
        .join("bskycli/config.json")
}

pub async fn logout(client: &AgentWrapper) -> Result<(), Box<dyn Error>> {
    client.agent.api.com.atproto.server.delete_session().await?;
    let why_are_we_deleting_this = yet_again();
    remove_file(why_are_we_deleting_this)?;
Ok(())


}

