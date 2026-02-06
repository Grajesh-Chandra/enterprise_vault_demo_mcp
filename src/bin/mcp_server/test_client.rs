/*!
 * Test Client for DIDComm Bridge via MCP Server
 *
 * This client:
 * 1. Spawns mcp-server-http (which connects to bridge via HTTP)
 * 2. Sends MCP JSON-RPC requests via stdio
 * 3. Bridge forwards to password service via DIDComm
 */

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use console::style;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

#[derive(Parser, Debug)]
#[command(version, about = "Test client for DIDComm Bridge", long_about = None)]
struct Args {
    /// URL of the DIDComm bridge
    #[arg(short, long, default_value = "http://127.0.0.1:8080")]
    bridge_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Get a password from the vault
    GetPassword {
        /// Password key to retrieve
        #[arg(short, long)]
        key: String,
    },
    /// List available password keys
    ListKeys,
    /// List all available tools
    ListTools,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

struct McpClient {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    request_id: u64,
}

impl McpClient {
    fn new(bridge_url: &str) -> Result<Self> {
        eprintln!("{}", style("Starting MCP server...").yellow());

        let mut process = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "mcp-server",
                "--",
                "--bridge-url",
                bridge_url,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to start MCP server")?;

        let stdin = process.stdin.take().context("Failed to get stdin")?;
        let stdout = BufReader::new(process.stdout.take().context("Failed to get stdout")?);

        let mut client = McpClient {
            process,
            stdin,
            stdout,
            request_id: 0,
        };

        // Initialize MCP connection
        client.initialize()?;

        Ok(client)
    }

    fn next_id(&mut self) -> u64 {
        self.request_id += 1;
        self.request_id
    }

    fn send_request(&mut self, method: &str, params: Option<Value>) -> Result<()> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.next_id(),
            method: method.to_string(),
            params,
        };

        let request_json = serde_json::to_string(&request)?;
        eprintln!("{} {}", style("→ Sending:").dim(), style(&request_json).cyan());

        writeln!(self.stdin, "{}", request_json)?;
        self.stdin.flush()?;

        Ok(())
    }

    fn read_response(&mut self) -> Result<JsonRpcResponse> {
        let mut line = String::new();
        self.stdout.read_line(&mut line)?;

        if line.trim().is_empty() {
            anyhow::bail!("Empty response from server");
        }

        eprintln!("{} {}", style("← Received:").dim(), style(&line.trim()).green());

        let response: JsonRpcResponse = serde_json::from_str(&line)
            .context("Failed to parse JSON-RPC response")?;

        if let Some(error) = &response.error {
            anyhow::bail!("RPC Error {}: {}", error.code, error.message);
        }

        Ok(response)
    }

    fn initialize(&mut self) -> Result<()> {
        eprintln!("\n{}", style("=== Initializing MCP Connection ===").yellow().bold());

        self.send_request(
            "initialize",
            Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "bridge-test-client",
                    "version": "0.1.0"
                }
            })),
        )?;

        let response = self.read_response()?;

        if response.result.is_some() {
            eprintln!("{}", style("✓ Connection initialized").green());

            // Send initialized notification (no ID, no response expected)
            let notification = json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized"
            });
            let notification_json = serde_json::to_string(&notification)?;
            eprintln!("{} {}", style("→ Sending:").dim(), style(&notification_json).cyan());
            writeln!(self.stdin, "{}", notification_json)?;
            self.stdin.flush()?;

            eprintln!("{}\n", style("✓ Sent initialized notification").green());
            Ok(())
        } else {
            anyhow::bail!("Failed to initialize MCP connection")
        }
    }

    fn list_tools(&mut self) -> Result<()> {
        eprintln!("{}", style("=== Listing Available Tools ===").yellow().bold());

        self.send_request("tools/list", None)?;
        let response = self.read_response()?;

        if let Some(result) = response.result {
            if let Some(tools) = result.get("tools").and_then(|t| t.as_array()) {
                eprintln!("{} {} tools:", style("Found").green(), tools.len());
                for tool in tools {
                    if let (Some(name), Some(desc)) = (
                        tool.get("name").and_then(|n| n.as_str()),
                        tool.get("description").and_then(|d| d.as_str()),
                    ) {
                        eprintln!("  {} - {}", style(name).cyan().bold(), desc);
                    }
                }
            }
        }

        Ok(())
    }

    fn call_tool(&mut self, name: &str, arguments: Value) -> Result<()> {
        eprintln!("{} {}", style("=== Calling Tool:").yellow().bold(), style(name).cyan());

        self.send_request(
            "tools/call",
            Some(json!({
                "name": name,
                "arguments": arguments
            })),
        )?;

        let response = self.read_response()?;

        if let Some(result) = response.result {
            if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
                eprintln!("{}", style("Result:").green().bold());
                for item in content {
                    if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                        println!("{}", text);
                    }
                }
            }
        }

        Ok(())
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("{}", style("Bridge Test Client").cyan().bold());
    println!("{}", style("==================\n").cyan());

    let mut client = McpClient::new(&args.bridge_url)?;

    match args.command {
        Commands::ListTools => {
            client.list_tools()?;
        }
        Commands::GetPassword { key } => {
            client.call_tool("get_password", json!({ "key": key }))?;
        }
        Commands::ListKeys => {
            client.call_tool("list_keys", json!({}))?;
        }
    }

    eprintln!("\n{}", style("✓ Test completed successfully").green().bold());

    Ok(())
}
