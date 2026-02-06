/*!
 * MCP Server with HTTP Bridge Client
 *
 * Accepts standard MCP requests (stdio/JSON-RPC) and forwards to DIDComm Bridge via HTTP
 */

use anyhow::Result;
use clap::Parser;
use console::style;
use rmcp::{
    ServerHandler, ServiceExt,
    model::{
        CallToolRequestParam, CallToolResult, Content, ListToolsResult,
        PaginatedRequestParam, ServerCapabilities, ServerInfo, Tool, Implementation,
    },
    service::{RequestContext, RoleServer},
    ErrorData as McpError,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::borrow::Cow;
use std::sync::Arc;
use tracing::{info, error};
use tracing_subscriber::filter;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum BridgeRequest {
    GetPassword { key: String },
    ListKeys,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum BridgeResponse {
    Success { data: String },
    NotFound { message: String },
    Error { error: String },
}

#[derive(Parser, Debug)]
#[command(version, about = "MCP Server with HTTP bridge", long_about = None)]
struct Args {
    #[arg(short, long, default_value = "http://127.0.0.1:8080")]
    bridge_url: String,
}

struct BridgeClient {
    bridge_url: String,
    client: reqwest::Client,
}

impl BridgeClient {
    fn new(bridge_url: String) -> Self {
        Self {
            bridge_url,
            client: reqwest::Client::new(),
        }
    }

    async fn get_password(&self, key: String) -> Result<Option<String>> {
        eprintln!("[MCP → Bridge] Requesting password for key: {}", key);
        info!("Requesting password for key: {} from bridge", key);

        let request = BridgeRequest::GetPassword { key };
        eprintln!("[MCP → Bridge] Sending POST to {}/bridge", self.bridge_url);
        let response = self
            .client
            .post(format!("{}/bridge", self.bridge_url))
            .json(&request)
            .send()
            .await?;
        eprintln!("[MCP ← Bridge] Received response: {}", response.status());

        if !response.status().is_success() {
            anyhow::bail!("Bridge returned error: {}", response.status());
        }

        let bridge_response: BridgeResponse = response.json().await?;

        match bridge_response {
            BridgeResponse::Success { data } => Ok(Some(data)),
            BridgeResponse::NotFound { .. } => Ok(None),
            BridgeResponse::Error { error } => anyhow::bail!("Bridge error: {}", error),
        }
    }

    async fn list_keys(&self) -> Result<String> {
        eprintln!("[MCP → Bridge] Requesting list of keys");
        info!("Requesting list of keys from bridge");

        let request = BridgeRequest::ListKeys;
        eprintln!("[MCP → Bridge] Sending POST to {}/bridge", self.bridge_url);
        let response = self
            .client
            .post(format!("{}/bridge", self.bridge_url))
            .json(&request)
            .send()
            .await?;
        eprintln!("[MCP ← Bridge] Received response: {}", response.status());

        if !response.status().is_success() {
            anyhow::bail!("Bridge returned error: {}", response.status());
        }

        let bridge_response: BridgeResponse = response.json().await?;

        match bridge_response {
            BridgeResponse::Success { data } => Ok(data),
            BridgeResponse::NotFound { message } => Ok(message),
            BridgeResponse::Error { error } => anyhow::bail!("Bridge error: {}", error),
        }
    }
}

struct PasswordVaultMCPServer {
    bridge: Arc<BridgeClient>,
}

impl PasswordVaultMCPServer {
    fn new(bridge: Arc<BridgeClient>) -> Self {
        Self { bridge }
    }
}

impl ServerHandler for PasswordVaultMCPServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: Default::default(),
            capabilities: ServerCapabilities {
                tools: Some(rmcp::model::ToolsCapability {
                    list_changed: None,
                }),
                ..Default::default()
            },
            server_info: Implementation {
                name: "password-vault-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
            instructions: Some("Password Vault MCP Server - HTTP to DIDComm Bridge".to_string()),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        info!("Listing available tools");

        Ok(ListToolsResult {
            tools: vec![
                Tool {
                    name: Cow::Borrowed("get_password"),
                    description: Some(Cow::Borrowed(
                        "Retrieves a password from the secure vault by key"
                    )),
                    input_schema: Arc::new({
                        let mut map = Map::new();
                        map.insert("type".to_string(), Value::String("object".to_string()));
                        let mut props = Map::new();
                        let mut key_prop = Map::new();
                        key_prop.insert("type".to_string(), Value::String("string".to_string()));
                        key_prop.insert("description".to_string(), Value::String("The password key to retrieve".to_string()));
                        props.insert("key".to_string(), Value::Object(key_prop));
                        map.insert("properties".to_string(), Value::Object(props));
                        map.insert("required".to_string(), Value::Array(vec![Value::String("key".to_string())]));
                        map
                    }),
                    annotations: None,
                    output_schema: None,
                },
                Tool {
                    name: Cow::Borrowed("list_keys"),
                    description: Some(Cow::Borrowed(
                        "Lists information about available password keys"
                    )),
                    input_schema: Arc::new({
                        let mut map = Map::new();
                        map.insert("type".to_string(), Value::String("object".to_string()));
                        map.insert("properties".to_string(), Value::Object(Map::new()));
                        map
                    }),
                    annotations: None,
                    output_schema: None,
                },
            ],
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        eprintln!("[MCP] Tool called: {}", request.name);
        info!("Tool called: {}", request.name);

        match request.name.as_ref() {
            "get_password" => {
                let key = request.arguments
                    .as_ref()
                    .and_then(|args| args.get("key"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| McpError::invalid_params("Missing 'key' parameter", None))?;

                match self.bridge.get_password(key.to_string()).await {
                    Ok(Some(password)) => Ok(CallToolResult {
                        content: Some(vec![Content::text(format!(
                            "Password for '{}': {}",
                            key, password
                        ))]),
                        is_error: None,
                        structured_content: None,
                    }),
                    Ok(None) => Ok(CallToolResult {
                        content: Some(vec![Content::text(format!(
                            "Password not found for key: '{}'",
                            key
                        ))]),
                        is_error: Some(true),
                        structured_content: None,
                    }),
                    Err(e) => {
                        error!("Failed to get password via bridge: {}", e);
                        Err(McpError::internal_error(format!("Failed to get password: {}", e), None))
                    }
                }
            }
            "list_keys" => {
                match self.bridge.list_keys().await {
                    Ok(data) => Ok(CallToolResult {
                        content: Some(vec![Content::text(data)]),
                        is_error: None,
                        structured_content: None,
                    }),
                    Err(e) => {
                        error!("Failed to list keys via bridge: {}", e);
                        Err(McpError::internal_error(format!("Failed to list keys: {}", e), None))
                    }
                }
            }
            _ => Err(McpError::invalid_request(format!("Unknown tool: {}", request.name), None)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(filter::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    eprintln!("{}", style("MCP Server (HTTP → Bridge)").cyan().bold());
    eprintln!("{}", style("Testing connection...").yellow());

    let bridge = Arc::new(BridgeClient::new(args.bridge_url.clone()));

    match bridge.client.get(format!("{}/health", args.bridge_url)).send().await {
        Ok(response) if response.status().is_success() => {
            eprintln!("{}", style("✅ Bridge is reachable\n").green());
        }
        Ok(response) => {
            eprintln!("{} {}", style("⚠️ Bridge responded:").yellow(), response.status());
        }
        Err(e) => {
            eprintln!("{} {}", style("❌ Cannot reach bridge:").red(), e);
            eprintln!("{}", style("\nPlease start the bridge:").yellow());
            eprintln!("  {}", style("cargo run --bin didcomm-bridge -- --service-did <DID>").cyan());
            return Err(e.into());
        }
    }

    let server = PasswordVaultMCPServer::new(bridge);
    eprintln!("{}", style("✅ MCP server ready\n").green());

    let transport = (tokio::io::stdin(), tokio::io::stdout());
    let service = server.serve(transport).await?;
    service.waiting().await?;

    Ok(())
}
