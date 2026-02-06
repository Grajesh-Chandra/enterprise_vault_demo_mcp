/*!
*   Password Service
*
*   Connects to a DIDComm Mediator and listens for Password requests and responds
*
*   Trivial example, no secrecy implied on service side or complex Access Management Checks
*/

use crate::{config::Config, messaging::handle_messaging, wizard::setup_wizard};
use anyhow::Result;
use console::style;
use enterprise_pw_demo::{CLI_BLUE, CLI_GREEN, CLI_ORANGE};
use tracing_subscriber::filter;

mod config;
mod messaging;
mod wizard;

// Default path for the config file
pub const CONFIG_PATH: &str = "config.json";

#[tokio::main]
pub async fn main() -> Result<()> {
    // construct a subscriber that prints formatted traces to stdout
    let subscriber = tracing_subscriber::fmt()
        // Use a more compact, abbreviated log format
        .with_env_filter(filter::EnvFilter::from_default_env())
        .finish();
    // use that subscriber to process traces emitted after this point
    tracing::subscriber::set_global_default(subscriber).expect("Logging failed, exiting...");

    let config = if let Ok(config) = Config::load_config(CONFIG_PATH) {
        config
    } else {
        setup_wizard(CONFIG_PATH)?
    };

    // We have a working configuration! Yay!
    println!(
        "{}",
        style("Use the following DID in the client application as the service DID")
            .color256(CLI_BLUE)
    );
    println!(
        "{} {}",
        style("Our DID:").color256(CLI_BLUE),
        style(&config.our_did).color256(CLI_GREEN)
    );
    println!();
    println!(
        "{}",
        style("Password keys you can request:").color256(CLI_BLUE)
    );
    for (key, _) in config.passwords.iter() {
        println!("\t{}", style(key).color256(CLI_ORANGE));
    }

    // Connect to the Mediator and await requests (loops here)
    handle_messaging(config).await?;

    Ok(())
}
