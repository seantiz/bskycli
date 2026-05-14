#![allow(dead_code)]

mod action;
mod api;
mod app;
mod event;
mod models;
mod tui;
mod ui;
mod utils;

use std::sync::Arc;

use anyhow::Result;


#[tokio::main]
async fn main() -> Result<()> {


    let client = Arc::new(api::wrapper::AgentWrapper::spinupagain().await?);

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
