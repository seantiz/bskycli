use bskycli::*;

use std::sync::Arc;

// TODO: Have to keep anyhow at the moment to handle tokio-main traits
use anyhow::Result;

use apple_native_keyring_store::keychain;
use keyring_core::set_default_store;

#[tokio::main]
async fn main() -> Result<()> {
    set_default_store(keychain::Store::new()?);

    // WARN: Spin up a dumb client if we can't find a smart one

    let client = match api::wrapper::AgentWrapper::spinupagain().await {
    Ok(c) => Arc::new(c),
    Err(e) => {
        eprintln!("Warning: Could not restore session: {}", e);
        // Create unauthenticated agent instead of crashing
        let agent = bsky_sdk::BskyAgent::builder().build().await?;
        Arc::new(api::wrapper::AgentWrapper { agent })
    }
};

    let mut terminal = tui::init()?;

    // Install a panic hook that restores the terminal before printing the panic
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = tui::restore();
        default_panic(info);
    }));

    let result = app::App::new(client).run(&mut terminal).await;
    tui::restore()?;

    result
}
