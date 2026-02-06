/*!
*    Handles all Messaging duties
*/

use crate::{CLI_GREEN, CLI_ORANGE, config::Config};
use affinidi_tdk::{
    common::TDKSharedState,
    didcomm::{Message, MessageBuilder, PackEncryptedOptions},
    messaging::{ATM, config::ATMConfig, profiles::ATMProfile, protocols::Protocols},
    secrets_resolver::SecretsResolver,
};
use anyhow::{Context, Result};
use console::style;
use enterprise_pw_demo::{CLI_RED, PasswordRequest, PasswordResponse};
use std::{collections::HashMap, sync::Arc, time::SystemTime};
use uuid::Uuid;

pub async fn handle_messaging(config: Config) -> Result<()> {
    // Instantiate TDK
    let tdk = TDKSharedState::default().await;

    // Load in the secrets
    tdk.secrets_resolver.insert_vec(&config.did_secrets).await;

    // Setup ATM Environment
    let atm = ATM::new(ATMConfig::builder().build()?, tdk).await?;
    let protocols = Protocols::new();

    // Create the ATM Profile for this account
    let profile = ATMProfile::new(
        &atm,
        Some("Password Service".to_string()),
        config.our_did.clone(),
        Some(config.mediator_did.clone()),
    )
    .await?;

    let profile = atm.profile_add(&profile, true).await?;

    println!(
        "\n{}\n",
        style("Successfully connected to Affinidi Trusted Messaging...").color256(CLI_GREEN)
    );

    let mut error_counter: u32 = 0;
    loop {
        match protocols
            .message_pickup
            .live_stream_next(&atm, &profile, None, true)
            .await
            .context("Error receiving message")?
        {
            Some((message, _)) => {
                if let Some(password) = process_message(&config.passwords, &message) {
                    // Respond with the password
                    match send_message(&config, &atm, &profile, &message, password).await {
                        Ok(_) => {
                            error_counter = 0; // reset error counter on success

                            println!(
                                "{}",
                                style("Successfully sent a password response for key")
                                    .color256(CLI_GREEN)
                            );
                        }
                        Err(e) => {
                            error_counter += 1;
                            println!(
                                "{}",
                                style(format!("ERROR: Couldn't send response: {e}"))
                                    .color256(CLI_RED)
                            );
                        }
                    }
                }
            }
            None => {
                println!("{}", style("Empty message received: Typically means timeout or a reconnection occurred. Safe to ignore...").color256(CLI_ORANGE));
                error_counter += 1;

                if error_counter > 5 {
                    println!(
                        "{}",
                        style("Too many errors in a row... Exiting...").color256(CLI_RED)
                    );
                    break;
                }
            }
        }
    }

    Ok(())
}

/// Process an inbound message and return a password if found
fn process_message(
    passwords: &HashMap<String, String>,
    message: &Message,
) -> Option<PasswordResponse> {
    if message
        .type_
        .starts_with("https://didcomm.org/messagepickup")
    {
        // Safely ignore this Message Pickup Protocol message
        None
    } else if message
        .type_
        .starts_with("https://demo.example/password/request")
        && let Ok(request) = serde_json::from_value::<PasswordRequest>(message.body.clone())
    {
        if let Some(password) = passwords.get(&request.key) {
            println!(
                "{}{}{}",
                style("Password request for key (").color256(CLI_GREEN),
                style(&request.key).color256(CLI_ORANGE),
                style(") found",).color256(CLI_GREEN)
            );
            Some(PasswordResponse::Found {
                key: request.key,
                password: password.to_string(),
            })
        } else {
            println!(
                "{}{}{}",
                style("Password request for key (").color256(CLI_ORANGE),
                style(request.key).color256(CLI_RED),
                style(") NOT found.",).color256(CLI_ORANGE)
            );
            Some(PasswordResponse::NotFound)
        }
    } else {
        println!(
            "{}",
            style("Received message with invalid format. Ignoring...").color256(CLI_RED)
        );
        None
    }
}

/// Send the response back to the client
async fn send_message(
    config: &Config,
    atm: &ATM,
    profile: &Arc<ATMProfile>,
    message: &Message,
    response: PasswordResponse,
) -> Result<()> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let Some(to_address) = message.from.as_ref() else {
        return Err(anyhow::anyhow!("Message missing 'from' field"));
    };

    let response = MessageBuilder::new(
        Uuid::new_v4().to_string(),
        "https://demo.example/password/response".to_string(),
        serde_json::to_value(response)?,
    )
    .from(config.our_did.clone())
    .to(to_address.clone())
    .created_time(now)
    .expires_time(now + 10)
    .thid(message.id.clone())
    .finalize();

    let (packed_msg, _) = atm
        .pack_encrypted(
            &response,
            to_address,
            Some(&config.our_did),
            Some(&config.our_did),
            Some(&PackEncryptedOptions {
                forward: false,
                ..Default::default()
            }),
        )
        .await?;

    atm.forward_and_send_message(
        profile,
        false,
        &packed_msg,
        None,
        &config.mediator_did,
        to_address,
        None,
        None,
        false,
    )
    .await?;

    Ok(())
}
