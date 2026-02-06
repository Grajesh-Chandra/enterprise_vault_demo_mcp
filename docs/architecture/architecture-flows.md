# Architecture Flows & Component Details

This document provides a comprehensive overview of how each component in the Enterprise Password Vault system works and communicates.



## 📊 System Overview

The system consists of **5 binaries** organized into 3 architectural layers:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           CLIENT LAYER                                  │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                   │
│  │   CLI Client │  │ AI Agents    │  │ Test Client  │                   │
│  │   (client)   │  │ (Claude, etc)│  │(bridge-test) │                   │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘                   │
│         │                  │                  │                         │
└─────────┼──────────────────┼──────────────────┼─────────────────────────┘
          │                  │                  │
          │                  │                  │ spawns & stdio
          │                  │ stdio/JSON-RPC   │
          │                  │                  ↓
          │                  │         ┌──────────────┐
          │                  └────────→│  MCP Server  │
          │                            │ (mcp-server) │
          │                            └──────┬───────┘
          │                                   │ HTTP/REST
          │ DIDComm                           │
┌─────────┼───────────────────────────────────┼──────────────────────────┐
│         │              BRIDGE LAYER         │                          │
├─────────┼───────────────────────────────────┼──────────────────────────┤
│         │                                   ↓                          │
│         │                          ┌──────────────┐                    │
│         │                          │   DIDComm    │                    │
│         │                          │    Bridge    │                    │
│         │                          │(didcomm-     │                    │
│         │                          │  bridge)     │                    │
│         │                          └──────┬───────┘                    │
│         │                                 │ DIDComm                    │
└─────────┼─────────────────────────────────┼────────────────────────────┘
          │                                 │
          │ DIDComm                         │ DIDComm
┌─────────┼─────────────────────────────────┼────────────────────────────┐
│         │           SERVICE LAYER         │                            │
├─────────┼─────────────────────────────────┼────────────────────────────┤
│         │                                 │                            │
│         └────────────────┬────────────────┘                            │
│                          ↓                                             │
│                 ┌──────────────┐                                       │
│                 │   Password   │                                       │
│                 │   Service    │                                       │
│                 │  (service)   │                                       │
│                 └──────────────┘                                       │
│                                                                        │
└────────────────────────────────────────────────────────────────────────┘
```



## 🔄 Data Flow Patterns

### Flow 1: Direct CLI Access (Simplest)
**Use Case:** Command-line password management

```
┌──────────┐         DIDComm          ┌──────────┐         DIDComm           ┌──────────┐
│   CLI    │ ────────────────────────>│ Mediator │──────────────────────────>│ Password │
│  Client  │  Encrypted Messages      │ (Cloud)  │  Encrypted Messages       │ Service  │
│          │ <────────────────────────│          │<──────────────────────────│          │
└──────────┘                          └──────────┘                           └──────────┘

Command:
  cargo run --bin client -- -s <DID> get-password --key myapp

Properties:
  ✅ Routes through mediator(s)
  ✅ Ephemeral DID (created per run)
  ✅ No MCP overhead
  ✅ Fastest for CLI use
  ✅ Client & Service can use same or different mediators
```



### Flow 2: AI Agent Integration
**Use Case:** Claude Desktop, MCP Inspector, custom AI agents

```
┌──────────┐   stdio     ┌──────────┐   HTTP    ┌──────────┐   DIDComm  ┌──────────┐   DIDComm  ┌──────────┐
│ AI Agent │ JSON-RPC    │   MCP    │  REST     │ DIDComm  │ Encrypted  │ Mediator │ Encrypted  │ Password │
│ (Claude) │ ──────────> │  Server  │ ────────> │  Bridge  │ ─────────> │ (Cloud)  │ ─────────> │ Service  │
│          │ <────────── │          │ <──────── │          │ <───────── │          │ <───────── │          │
└──────────┘  responses  └──────────┘  JSON     └──────────┘  messages  └──────────┘  messages  └──────────┘

Note: Bridge and Service each register with mediator(s). Can be same or different mediators.

Setup:
  Terminal 1: cargo run --bin service
  Terminal 2: cargo run --bin didcomm-bridge -- --service-did <DID>

Claude Config:
  {
    "mcpServers": {
      "password-vault": {
        "command": "cargo",
        "args": ["run", "--bin", "mcp-server", "--", "--bridge-url", "http://127.0.0.1:8080"]
      }
    }
  }

Properties:
  ✅ AI agent spawns MCP server
  ✅ MCP server is lightweight (HTTP client only)
  ✅ Bridge has persistent DID
  ✅ Scalable (multiple agents → one bridge)
  ✅ Bridge & Service use mediator(s) for DIDComm routing
  ✅ Mediators cannot decrypt messages (E2E encrypted)
```



### Flow 3: Testing & Validation
**Use Case:** Verify the complete MCP architecture works

```
┌──────────┐   spawns    ┌──────────┐   HTTP    ┌──────────┐   DIDComm  ┌──────────┐   DIDComm  ┌──────────┐
│  Test    │ subprocess  │   MCP    │  REST     │ DIDComm  │ Encrypted  │ Mediator │ Encrypted  │ Password │
│  Client  │ ──────────> │  Server  │ ────────> │  Bridge  │ ─────────> │ (Cloud)  │ ─────────> │ Service  │
│          │   stdio     │          │           │          │            │          │            │          │
│          │ JSON-RPC ─> │          │           │          │            │          │            │          │
│          │ <────────── │          │ <──────── │          │ <───────── │          │ <───────── │          │
└──────────┘  responses  └──────────┘  JSON     └──────────┘  messages  └──────────┘  messages  └──────────┘

Note: Bridge and Service each register with mediator(s). Can be same or different mediators.

Commands:
  Terminal 1: cargo run --bin service
  Terminal 2: cargo run --bin didcomm-bridge -- --service-did <DID>
  Terminal 3: cargo run --bin bridge-test-client -- get-password --key myapp

Properties:
  ✅ Test client spawns MCP server
  ✅ Full integration test
  ✅ Validates entire chain including mediator routing
  ✅ Good for debugging
```



## 🎯 Component Details

### 1️⃣ Password Service (`service`)
**Binary:** `service`
**Source:** `src/bin/service/`

**Role:** Backend vault that stores passwords

**Communication:**
- Receives: DIDComm encrypted messages (via mediator)
- Sends: DIDComm encrypted responses (via mediator)
- Registers with: Mediator for message routing

**Properties:**
- ✅ Persistent DID (saved on first run)
- ✅ DIDComm-only access (no HTTP)
- ✅ Passwords pre-configured in `config.json`
- ✅ Supports: **GetPassword**, **ListKeys** only
- ❌ Does NOT support storing passwords dynamically

**Command:**
```bash
cargo run --bin service
# Output: Service DID (copy this!)
```

**Configuration (`config.json`):**
```json
{
  "mediator_did": "did:web:mediator-nlb.storm.ws:mediator:v1:.well-known",
  "our_did": "did:peer:2.Ez6LS...",
  "did_secrets": [...],
  "passwords": {
    "myapp": "secret123",
    "database": "db_password",
    "api_key": "1234567890"
  }
}
```

**Mediator Configuration:**
- Service registers with mediator specified in `mediator_did`
- Mediator routes messages to/from Service's DID
- Multiple mediators can be configured for redundancy
```

**Note:** Passwords must be added to `config.json` before starting the service. The service will create this file on first run via setup wizard.



### 2️⃣ CLI Client (`client`)
**Binary:** `client`
**Source:** `src/bin/client/`

**Role:** Direct command-line password management

**Communication:**
- Sends: DIDComm encrypted requests (via mediator)
- Receives: DIDComm encrypted responses (via mediator)
- Registers with: Mediator for message routing

**Properties:**
- ✅ Ephemeral DID (new each run)
- ✅ Direct DIDComm client
- ✅ No MCP involvement
- ✅ Simple CLI tool

**Commands:**
```bash
# Get password (passwords must be pre-configured in config.json)
cargo run --bin client -- -s <SERVICE_DID> --password-key myapp
```

**Note:** The client only retrieves passwords. Passwords must be added to `config.json` before starting the service.

**Does NOT:**
- ❌ Spawn MCP server
- ❌ Use bridge
- ❌ Use HTTP



### 3️⃣ DIDComm Bridge (`didcomm-bridge`)
**Binary:** `didcomm-bridge`
**Source:** `src/bin/mcp_server/bridge_service.rs`

**Role:** Persistent bridge with HTTP API → DIDComm translation

**Communication:**
- Receives: HTTP POST requests (from MCP servers)
- Sends: DIDComm encrypted messages (via mediator to Password Service)
- Receives: DIDComm encrypted responses (via mediator from Password Service)
- Sends: HTTP JSON responses (to MCP servers)
- Registers with: Mediator for message routing

**Properties:**
- ✅ **Persistent DID** (saved to `bridge_config.json`)
- ✅ HTTP server (Axum) on port 8080
- ✅ DIDComm client to Password Service
- ✅ Stateful (reuses same DID across restarts)

**Endpoints:**
- `POST /bridge` - Forward requests to service
- `GET /health` - Health check

**Commands:**
```bash
# First run (creates config with service DID)
cargo run --bin didcomm-bridge -- --service-did <SERVICE_DID>

# Subsequent runs (reuses bridge DID)
cargo run --bin didcomm-bridge

# Custom port
cargo run --bin didcomm-bridge -- --port 9090

# Test health
curl http://127.0.0.1:8080/health
```

**Configuration:**
```json
// bridge_config.json (auto-created)
{
  "our_did": "did:peer:2.Ez6LS...",      // Persistent bridge DID
  "did_secrets": [...],                   // Private keys
  "service_did": "did:peer:2.Ez6LS...",  // Password service DID
  "mediator_did": "did:web:mediator-nlb.storm.ws:mediator:v1:.well-known" // Mediator
}
```

**Mediator Configuration:**
- Bridge registers with mediator specified in `mediator_did`
- Bridge and Service can use same or different mediators
- Mediator routes DIDComm messages between Bridge and Service
- All messages are end-to-end encrypted (mediator cannot read)



### 4️⃣ MCP Server (`mcp-server`)
**Binary:** `mcp-server`
**Source:** `src/bin/mcp_server/http_client.rs`

**Role:** Lightweight MCP protocol handler with HTTP client

**Communication:**
- Receives: stdio JSON-RPC (from AI agents)
- Sends: HTTP POST (to bridge)
- Receives: HTTP JSON responses (from bridge)
- Sends: stdio JSON-RPC responses (to AI agents)

**Properties:**
- ✅ **Stateless** (no persistent state)
- ✅ Implements MCP protocol (version 2024-11-05)
- ✅ HTTP client (reqwest) to bridge
- ✅ Lightweight (spawned per agent session)

**MCP Tools:**
- `get_password` - Retrieve password by key
- `list_keys` - List available password keys

**Commands:**
```bash
# Standalone (for AI agents)
cargo run --bin mcp-server -- --bridge-url http://127.0.0.1:8080

# With custom bridge URL
cargo run --bin mcp-server -- --bridge-url http://127.0.0.1:9090
```

**MCP Protocol Flow:**
```
1. Client → Server: initialize request
2. Server → Client: initialize response (capabilities)
3. Client → Server: initialized notification
4. Client → Server: tools/list request
5. Server → Client: tools/list response
6. Client → Server: tools/call request (get_password)
7. Server → Client: tools/call response
```



### 5️⃣ Bridge Test Client (`bridge-test-client`)
**Binary:** `bridge-test-client`
**Source:** `src/bin/mcp_server/test_client.rs`

**Role:** Test validation tool

**Communication:**
- Spawns: `mcp-server` as subprocess
- Sends: stdio JSON-RPC (to spawned MCP server)
- Receives: stdio JSON-RPC responses

**Properties:**
- ✅ Integration testing tool
- ✅ Spawns MCP server automatically
- ✅ Colored output for debugging
- ✅ Validates full chain

**Commands:**
```bash
# List available tools
cargo run --bin bridge-test-client -- list-tools

# Get password
cargo run --bin bridge-test-client -- get-password --key myapp

# List keys
cargo run --bin bridge-test-client -- list-keys

# Custom bridge URL
cargo run --bin bridge-test-client -- --bridge-url http://127.0.0.1:9090 list-tools
```



## 🔐 Security Model

### DID Management

| Component | DID Type | Persistence | Purpose |
|--|-|-||
| Password Service | Persistent | Saved to `config.json` | Stable service identity |
| DIDComm Bridge | Persistent | Saved to `bridge_config.json` | Trusted bridge identity |
| CLI Client | Ephemeral | Generated per run | Temporary access |

### Trust Chain

```
Service DID (trusted root)
    ↓
Bridge DID (configured with service DID)
    ↓
MCP Server (configured with bridge URL)
    ↓
AI Agent (configured with MCP server)
```

**Key Points:**
- Service DID is the root of trust
- Bridge must be configured with correct service DID
- MCP server trusts bridge URL
- All DIDComm messages are encrypted end-to-end
- Mediators route messages but cannot decrypt them
- Each component can register with multiple mediators for redundancy
- Bridge and Service can share mediators or use different ones



## 🚀 Complete Setup Guide

### Development Setup (All Components)

```bash
# Terminal 1: Password Service
cargo run --bin service
# → Copy the Service DID

# Terminal 2: DIDComm Bridge
cargo run --bin didcomm-bridge -- --service-did "did:peer:2.Ez6LS..."
# → Bridge starts on http://127.0.0.1:8080

# Terminal 3: Test via MCP (passwords must be pre-configured in config.json)
cargo run --bin bridge-test-client -- get-password --key myapp
# → Should output: Password for 'myapp': secret123
```



### Production Setup (AI Agent)

```bash
# Terminal 1: Password Service (daemon)
cargo build --release
./target/release/service

# Terminal 2: DIDComm Bridge (daemon)
./target/release/didcomm-bridge --service-did "did:peer:2.Ez6LS..."

# Claude Desktop Config (~/.config/Claude/claude_desktop_config.json)
{
  "mcpServers": {
    "password-vault": {
      "command": "/path/to/target/release/mcp-server",
      "args": ["--bridge-url", "http://127.0.0.1:8080"]
    }
  }
}

# Restart Claude Desktop
# → Ask Claude: "What tools do you have for password management?"
```


## 📊 Comparison Matrix

### When to Use Each Binary

| Use Case | Binary | Reason |
|-|--|--|
| Quick password management | `client` | Simplest, direct access |
| AI agent integration | `mcp-server` + `didcomm-bridge` | Standard MCP protocol |
| Testing MCP flow | `bridge-test-client` | Full integration test |
| Backend storage | `service` | Always required |
| HTTP → DIDComm bridge | `didcomm-bridge` | Required for MCP |

### Component Dependencies

```
service (independent - runs alone)
    ↑
    │ DIDComm
    │
client (independent - only needs service)

service (independent - runs alone)
    ↑
    │ DIDComm
    │
didcomm-bridge (needs service DID)
    ↑
    │ HTTP
    │
mcp-server (needs bridge URL)
    ↑
    │ stdio/JSON-RPC
    │
AI Agent (needs mcp-server config)
```



## 🔍 Troubleshooting Flow Diagram

```
Problem: Can't get password
    │
    ├─ Using CLI client?
    │   ├─ Yes → Check service is running
    │   │        Check service DID is correct
    │   │
    │   └─ No → Using MCP?
    │            │
    │            ├─ Check service is running
    │            ├─ Check bridge is running
    │            ├─ Check bridge has correct service DID
    │            ├─ Check MCP server has correct bridge URL
    │            └─ Check AI agent has correct MCP config
    │
    └─ Enable verbose logging:
        RUST_LOG=debug cargo run --bin <component>
```

## 🎓 Key Takeaways

1. **`client`** = Direct DIDComm access (simplest)
2. **`bridge-test-client`** = Testing tool (spawns MCP server)
3. **`mcp-server`** = For AI agents (spawned by them)
4. **`didcomm-bridge`** = Persistent service (always running)
5. **`service`** = Backend (always running)

**The bifurcated design means:**
- MCP servers are lightweight and stateless
- Bridge is persistent with stable DID
- Clean separation of concerns
- Scalable architecture (many agents → one bridge)
