/*!
 * DIDComm Bridge Service
 *
 * This is a standalone bridge service that:
 * - Has its own persistent DID (can be configured/saved)
 * - Accepts requests via HTTP/WebSocket from MCP servers
 * - Forwards requests to password service via DIDComm
 * - Returns responses back to MCP servers
 *
 * Architecture:
 * MCP Server → HTTP/WS → DIDComm Bridge (trusted) → DIDComm → Password Service
 */

use affinidi_tdk::{
    common::TDKSharedState,
    didcomm::{MessageBuilder, PackEncryptedOptions},
    messaging::{ATM, config::ATMConfig, profiles::ATMProfile, transports::SendMessageResponse},
    secrets_resolver::{SecretsResolver, secrets::Secret},
};
use anyhow::{Context, Result, bail};
use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use clap::Parser;
use console::style;
use enterprise_pw_demo::{generate_did, PasswordRequest, PasswordResponse, CLI_BLUE, CLI_GREEN};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{info, error};
use tracing_subscriber::filter;
use uuid::Uuid;

/// Bridge request from MCP server
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum BridgeRequest {
    GetPassword { key: String },
    ListKeys,
}

/// Bridge response to MCP server
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum BridgeResponse {
    Success { data: String },
    NotFound { message: String },
    Error { error: String },
}

/// Bridge configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
struct BridgeConfig {
    /// DID of the bridge (persistent)
    pub bridge_did: Option<String>,
    /// Secrets for the bridge DID
    pub bridge_secrets: Option<Vec<Secret>>,
    /// DID of the password service
    pub service_did: String,
    /// DID of the mediator
    pub mediator_did: String,
    /// Port to listen on
    pub port: u16,
}

impl BridgeConfig {
    fn load(path: &str, service_did_override: Option<String>) -> Result<Self> {
        if !std::path::Path::new(path).exists() {
            // Create template if no override provided
            if service_did_override.is_none() {
                let template = BridgeConfig {
                    bridge_did: None,
                    bridge_secrets: None,
                    service_did: "REPLACE_WITH_SERVICE_DID".to_string(),
                    mediator_did: "did:web:mediator-nlb.storm.ws:mediator:v1:.well-known".to_string(),
                    port: 8080,
                };

                let contents = serde_json::to_string_pretty(&template)?;
                std::fs::write(path, contents)?;

                bail!(
                    "Configuration file '{}' created. Please:\n\
                    1. Start password service: cargo run --bin service\n\
                    2. Provide service DID via --service-did flag, or\n\
                    3. Edit {} and set service_did, then restart",
                    path, path
                );
            }

            // Create new config with override
            let new_config = BridgeConfig {
                bridge_did: None,
                bridge_secrets: None,
                service_did: service_did_override.unwrap(),
                mediator_did: "did:web:mediator-nlb.storm.ws:mediator:v1:.well-known".to_string(),
                port: 8080,
            };

            let contents = serde_json::to_string_pretty(&new_config)?;
            std::fs::write(path, contents)?;

            return Ok(new_config);
        }

        let contents = std::fs::read_to_string(path)?;
        let mut config: BridgeConfig = serde_json::from_str(&contents)?;

        // Apply override if provided
        if let Some(did) = service_did_override {
            config.service_did = did;
        }

        if config.service_did == "REPLACE_WITH_SERVICE_DID" {
            bail!(
                "Please provide service DID via --service-did flag or edit {} and set service_did",
                path
            );
        }

        Ok(config)
    }

    fn save(&self, path: &str) -> Result<()> {
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}

/// DIDComm Bridge Service
#[derive(Parser, Debug)]
#[command(version, about = "DIDComm Bridge Service - Trusted bridge between MCP and DIDComm", long_about = None)]
struct Args {
    /// Path to bridge configuration file
    #[arg(short, long, default_value = "bridge_config.json")]
    config: String,

    /// DID of the password service (overrides config)
    #[arg(short, long)]
    service_did: Option<String>,

    /// Port to listen on (overrides config)
    #[arg(short, long)]
    port: Option<u16>,
}

/// Bridge state
struct BridgeState {
    our_did: String,
    did_secrets: Vec<Secret>,
    mediator_did: String,
    service_did: String,
    atm: Arc<RwLock<Option<Arc<ATM>>>>,
    profile: Arc<RwLock<Option<Arc<ATMProfile>>>>,
}

impl BridgeState {
    fn new(
        our_did: String,
        did_secrets: Vec<Secret>,
        mediator_did: String,
        service_did: String,
    ) -> Self {
        Self {
            our_did,
            did_secrets,
            mediator_did,
            service_did,
            atm: Arc::new(RwLock::new(None)),
            profile: Arc::new(RwLock::new(None)),
        }
    }

    async fn initialize(&self) -> Result<()> {
        info!("Initializing DIDComm bridge");

        let tdk = TDKSharedState::default().await;
        tdk.secrets_resolver.insert_vec(&self.did_secrets).await;

        let atm = Arc::new(ATM::new(ATMConfig::builder().build()?, tdk).await?);

        let profile = ATMProfile::new(
            &atm,
            Some("DIDComm Bridge".to_string()),
            self.our_did.clone(),
            Some(self.mediator_did.clone()),
        )
        .await?;

        let profile = atm.profile_add(&profile, true).await?;

        *self.atm.write().await = Some(atm);
        *self.profile.write().await = Some(profile);

        info!("DIDComm bridge initialized successfully");
        Ok(())
    }

    async fn get_password(&self, key: String) -> Result<Option<String>> {
        info!("Getting password for key: {}", key);

        let atm = self.atm.read().await;
        let profile = self.profile.read().await;

        let atm = atm.as_ref().ok_or_else(|| anyhow::anyhow!("ATM not initialized"))?;
        let profile = profile.as_ref().ok_or_else(|| anyhow::anyhow!("Profile not initialized"))?;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let msg_id = Uuid::new_v4().to_string();

        let message = MessageBuilder::new(
            msg_id.clone(),
            "https://demo.example/password/request".to_string(),
            serde_json::to_value(PasswordRequest { key: key.clone() })?,
        )
        .from(self.our_did.clone())
        .to(self.service_did.clone())
        .created_time(now)
        .expires_time(now + 10)
        .finalize();

        let (packed_msg, _) = atm
            .pack_encrypted(
                &message,
                &self.service_did,
                Some(&self.our_did),
                Some(&self.our_did),
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
                &self.mediator_did,
                &self.service_did,
                None,
                None,
                true,
            )
            .await?;

        match response {
            SendMessageResponse::Message(msg) => {
                if msg.type_.starts_with("https://demo.example/password/response") {
                    match serde_json::from_value::<PasswordResponse>(msg.body)? {
                        PasswordResponse::NotFound => Ok(None),
                        PasswordResponse::Found { key: _, password } => Ok(Some(password)),
                    }
                } else {
                    bail!("Unexpected message type: {}", msg.type_);
                }
            }
            _ => bail!("Expected DIDComm message response"),
        }
    }

    async fn handle_request(&self, request: BridgeRequest) -> BridgeResponse {
        match request {
            BridgeRequest::GetPassword { key } => {
                match self.get_password(key.clone()).await {
                    Ok(Some(password)) => BridgeResponse::Success {
                        data: password,
                    },
                    Ok(None) => BridgeResponse::NotFound {
                        message: format!("Password not found for key: '{}'", key),
                    },
                    Err(e) => {
                        error!("Failed to get password: {}", e);
                        BridgeResponse::Error {
                            error: format!("Failed to get password: {}", e),
                        }
                    }
                }
            }
            BridgeRequest::ListKeys => BridgeResponse::Success {
                data: "Available password keys are managed by the DIDComm password service.".to_string(),
            },
        }
    }
}

// HTTP handlers
async fn handle_bridge_request(
    State(state): State<Arc<BridgeState>>,
    Json(request): Json<BridgeRequest>,
) -> Result<Json<BridgeResponse>, AppError> {
    println!("[Bridge <- MCP] Received request: {:?}", request);
    info!("Handling bridge request: {:?}", request);
    let response = state.handle_request(request).await;
    println!("[Bridge -> MCP] Sending response: {:?}", response);
    Ok(Json(response))
}

async fn health_check() -> &'static str {
    "DIDComm Bridge is healthy"
}

// Error handling
struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal error: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(filter::EnvFilter::from_default_env())
        .init();

    println!("{}", style("DIDComm Bridge Service").cyan().bold());
    println!("{}", style("=====================\n").cyan());

    // Load or create configuration (pass service_did override)
    let mut config = BridgeConfig::load(&args.config, args.service_did.clone())
        .context("Failed to load bridge configuration")?;

    // Override port if provided
    if let Some(port) = args.port {
        config.port = port;
    }

    // Generate or load bridge DID
    let (bridge_did, bridge_secrets) = if let (Some(did), Some(secrets)) =
        (&config.bridge_did, &config.bridge_secrets) {
        println!("{} {}",
            style("Using existing Bridge DID:").green().bold(),
            style(did).cyan()
        );
        (did.clone(), secrets.clone())
    } else {
        println!("{}", style("Generating new Bridge DID...").yellow());
        let (did, secrets) = generate_did(&config.mediator_did)?;

        // Save to config for persistence
        config.bridge_did = Some(did.clone());
        config.bridge_secrets = Some(secrets.clone());
        config.save(&args.config)?;

        println!("{} {}",
            style("✅ New Bridge DID created:").green().bold(),
            style(&did).cyan()
        );
        println!("{}",
            style("  (Saved to config for future use)").dim()
        );

        (did, secrets)
    };

    println!("{} {}",
        style("Password Service DID:").color256(CLI_BLUE),
        style(&config.service_did).color256(CLI_GREEN)
    );
    println!("{} {}",
        style("Listening on port:").yellow(),
        style(config.port).cyan()
    );
    println!();

    // Create bridge state
    let state = Arc::new(BridgeState::new(
        bridge_did.clone(),
        bridge_secrets,
        config.mediator_did.clone(),
        config.service_did.clone(),
    ));

    // Initialize DIDComm
    println!("{}", style("Initializing DIDComm connection...").yellow());
    state.initialize().await?;
    println!("{}", style("✅ DIDComm bridge ready").color256(CLI_GREEN));
    println!();

    // Build router
    let app = Router::new()
        .route("/bridge", post(handle_bridge_request))
        .route("/health", axum::routing::get(health_check))
        .with_state(state);

    // Start server
    let addr = format!("127.0.0.1:{}", config.port);
    println!("{}", style("═══════════════════════════════════════").cyan());
    println!("{} {}",
        style("✅ Bridge Service Ready").green().bold(),
        style(&addr).cyan()
    );
    println!("{}", style("═══════════════════════════════════════").cyan());
    println!();
    println!("{}", style("Bridge DID (share this with MCP servers):").yellow().bold());
    println!("  {}", style(&bridge_did).cyan().bold());
    println!();
    println!("{}", style("Endpoints:").yellow());
    println!("  POST   http://{}/bridge   - Process bridge requests", addr);
    println!("  GET    http://{}/health   - Health check", addr);
    println!();
    println!("{}", style("Waiting for requests from MCP servers...").dim());
    println!();

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
