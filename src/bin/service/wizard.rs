/*!
*    Runs a simple configuration wizard for the service
*/

use crate::{CLI_BLUE, config::Config};
use anyhow::Result;
use console::style;
use dialoguer::{Input, theme::ColorfulTheme};
use enterprise_pw_demo::{CLI_RED, DEFAULT_MEDIATOR, generate_did};
use std::collections::HashMap;

/// Runs the Setup Wizard
pub fn setup_wizard(config_path: &str) -> Result<Config> {
    // What mediator to use?
    let mediator_did = get_mediator();

    // Generate a DID for us to use
    let (our_did, did_secrets) = generate_did(&mediator_did)?;

    // Generate some test passwords
    let passwords = get_passwords();

    let config = Config {
        mediator_did,
        our_did,
        did_secrets,
        passwords,
    };

    config.save(config_path)?;

    Ok(config)
}

fn get_mediator() -> String {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Mediator URL")
        .default(DEFAULT_MEDIATOR.to_string())
        .interact_text()
        .unwrap()
}

fn get_passwords() -> HashMap<String, String> {
    let mut passwords = HashMap::new();

    println!(
        "{}",
        style(
            "Enter some test passwords (identifier=password), one per line. Enter an empty line to finish:"
        )
        .color256(CLI_BLUE)
    );

    loop {
        let pw: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Password key:value")
            .allow_empty(true)
            .interact_text()
            .unwrap();

        if pw.trim().is_empty() {
            break;
        } else if let Some((key, value)) = pw.split_once('=') {
            passwords.insert(key.trim().to_string(), value.trim().to_string());
        } else {
            println!(
                "{}",
                style("Invalid entry! Must be in format identifier=password").color256(CLI_RED)
            );
        }
    }

    passwords
}
