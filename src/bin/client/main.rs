/*!
*   Password Manager Client
*/

use std::{sync::Arc, time::SystemTime};

use affinidi_tdk::{
    common::TDKSharedState,
    didcomm::{MessageBuilder, PackEncryptedOptions},
    messaging::{ATM, config::ATMConfig, profiles::ATMProfile, transports::SendMessageResponse},
    secrets_resolver::SecretsResolver,
};
use anyhow::{Result, bail};
use clap::Parser;
use console::style;
use enterprise_pw_demo::{
    CLI_BLUE, CLI_GREEN, CLI_ORANGE, CLI_RED, DEFAULT_MEDIATOR, PasswordRequest, PasswordResponse,
    generate_did,
};
use tracing_subscriber::filter;
use uuid::Uuid;

/// CLI Arguments
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Password Key
    #[arg(short, long)]
    password_key: String,

    /// Service DID to send request to
    #[arg(short, long)]
    service_did: String,

    /// Override Mediator?
    #[arg(short, long, default_value = DEFAULT_MEDIATOR)]
    mediator_did: String,
}

#[tokio::main]
pub async fn main() -> Result<()> {
    // construct a subscriber that prints formatted traces to stdout
    let subscriber = tracing_subscriber::fmt()
        // Use a more compact, abbreviated log format
        .with_env_filter(filter::EnvFilter::from_default_env())
        .finish();
    // use that subscriber to process traces emitted after this point
    tracing::subscriber::set_global_default(subscriber).expect("Logging failed, exiting...");

    let args: Args = Args::parse();

    // Generate a random DID for this request
    let (our_did, did_secrets) = generate_did(&args.mediator_did)?;
    println!(
        "{} {}",
        style("Our DID:").color256(CLI_BLUE),
        style(&our_did).color256(CLI_GREEN)
    );

    // Instantiate TDK
    let tdk = TDKSharedState::default().await;

    // Load in the secrets
    tdk.secrets_resolver.insert_vec(&did_secrets).await;

    // Setup ATM Environment
    let atm = ATM::new(ATMConfig::builder().build()?, tdk).await?;

    // Create the ATM Profile for this account
    let profile = ATMProfile::new(
        &atm,
        Some("Password Client ".to_string()),
        our_did.clone(),
        Some(args.mediator_did.clone()),
    )
    .await?;

    let profile = atm.profile_add(&profile, true).await?;

    println!(
        "\n{}\n",
        style("Successfully connected to Affinidi Trusted Messaging...").color256(CLI_GREEN)
    );

    // Send Message
    match get_password(&atm, &profile, &args).await? {
        None => {
            println!(
                "{} {}",
                style("Couldn't find password for key:").color256(CLI_RED),
                style(args.password_key).color256(CLI_ORANGE)
            );
        }
        Some(password) => {
            println!(
                "{}{}{} {}",
                style("Found password (").color256(CLI_GREEN),
                style(password).color256(CLI_ORANGE),
                style(") for key:").color256(CLI_GREEN),
                style(args.password_key).color256(CLI_ORANGE)
            );
        }
    }

    Ok(())
}

/// Send the request to the Service and wait for a response
/// Returns: msg_id thread to look for
async fn get_password(atm: &ATM, profile: &Arc<ATMProfile>, args: &Args) -> Result<Option<String>> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let our_did = &profile.inner.did;
    let msg_id = Uuid::new_v4().to_string();

    let response = MessageBuilder::new(
        msg_id.clone(),
        "https://demo.example/password/request".to_string(),
        serde_json::to_value(PasswordRequest {
            key: args.password_key.clone(),
        })?,
    )
    .from(our_did.clone())
    .to(args.service_did.clone())
    .created_time(now)
    .expires_time(now + 10)
    .finalize();

    let (packed_msg, _) = atm
        .pack_encrypted(
            &response,
            &args.service_did,
            Some(our_did),
            Some(our_did),
            Some(&PackEncryptedOptions {
                forward: false,
                ..Default::default()
            }),
        )
        .await?;

    let response = atm
        .forward_and_send_message(
            profile,
            false,
            &packed_msg,
            Some(&msg_id),
            &args.mediator_did,
            &args.service_did,
            None,
            None,
            true,
        )
        .await?;

    println!("{}", style("Message received").color256(CLI_GREEN));

    match response {
        SendMessageResponse::Message(msg) => {
            if msg
                .type_
                .starts_with("https://demo.example/password/response")
            {
                match serde_json::from_value::<PasswordResponse>(msg.body)? {
                    PasswordResponse::NotFound => Ok(None),
                    PasswordResponse::Found { key: _, password } => Ok(Some(password)),
                }
            } else {
                bail!(format!("Incorrect message type found: {}", msg.type_));
            }
        }
        _ => {
            bail!("expected DIDComm message respoonse, instead we got something else");
        }
    }
}
