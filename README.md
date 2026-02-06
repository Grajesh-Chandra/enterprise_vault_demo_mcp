# Enterprise Password Vault Demo

This prototype enables **AI agents** to securely retrieve passwords using standard **MCP protocol**, while decoupling transport and encryption via **DIDComm** for scalable, and decentralised credential access.

This demonstration showcases how autonomous agents (non-human identities) can securely access enterprise services using DIDComm for encrypted, decentralised communication with MCP as one integration path among many. By meeting agentic platforms where they are, it reduces the cost and complexity of adopting DIDComm in Agent-to-Human workflows without requiring deep protocol changes.

Autonomous agents can securely access enterprise services using MCP for agent-side simplicity and DIDComm for dynamic, end-to-end encrypted communication. DIDComm enables mutual authentication, decentralised identity resolution, and payload-level encryption establishing digital trust across boundaries. The architecture pattern avoids static client registration and user-in-the-flow dependencies enabling dynamic, auditable, and zero-trust Agent-to-Enterprise access.
It offers a credible starting point for exploring scalable, trust-minimized architectures built on agent identity, digital trust, and adaptive interoperability across heterogeneous systems

## 🎯 What Can I Do With This?

| I Want To... | Use This | Command |
|--------------|----------|---------|
| **Retrieve passwords** | Client | `cargo run --bin client -- -s <DID> --password-key myapp` |
| **Integrate with AI agents** | MCP Server + Bridge | See [Quick Start](#quick-start) |
| **Test the architecture** | Test Client | `cargo run --bin bridge-test-client -- list-tools` |

## 🏗️ Architecture

### For AI Agent Developers

**If you're familiar with MCP (Model Context Protocol)**, this is how it works:

```
┌──────────────────┐
│   Your AI Agent  │ ──── You're here! Your agent needs secure password access
│   (Claude, etc)  │
└────────┬─────────┘
         │ stdio (MCP JSON-RPC)
         │ Standard MCP protocol - nothing new to learn!
         ↓
┌─────────────────┐
│   MCP Server    │ ──── Lightweight process, speaks standard MCP
│   (this repo)   │      Exposes tools: get_password, list_keys
└────────┬────────┘
         │ HTTP/REST (localhost) ⚠️ CURRENT DEMO - NOT PRODUCTION READY
         │
         │ ❌ Security Issue: Passwords sent in plaintext on localhost
         │ ✅ Solution: See security-hardening doc for options:
         │    • Option 1: In-Process MCP (zero network)
         │    • Option 2: MCP/DIDComm Transport (E2E encrypted)
         │    • Option 3: HTTPS + mTLS (traditional security)
         ↓
┌──────────────────────────────────────────────────────────────────────────────┐
│                         🔐 DIDComm Security Layer                            │
│                                                                              │
│  ┌──────────────┐                                          ┌──────────────┐  │
│  │   Bridge     │                                          │  Password    │  │
│  │   (Relay)    │                                          │  Service     │  │
│  │              │                                          │  (Vault)     │  │
│  │ DID: did:A   │                                          │ DID: did:B   │  │
│  └──────┬───────┘                                          └──────┬───────┘  │
│         │                                                         │          │
│         │ DIDComm Messages                         DIDComm Messages          │
│         │ (E2E Encrypted)                          (E2E Encrypted)           │
│         ↓                                                         ↓          │
│  ┌──────────────┐                                      ┌──────────────┐      │
│  │  Mediator 1  │                                      │  Mediator 2  │      │
│  │ (Cloud/P2P)  │ ◄──────────────────────────────────► │ (Cloud/P2P)  │      │
│  └──────────────┘      DIDComm Message Routing         └──────────────┘      │
│       Bridge's         (E2E Encrypted)                     Service's         │
│       mediator(s)                                          mediator(s)       │
│                                                                              │
│  How it works:                                                               │
│  • Bridge registers with Mediator 1 (or multiple mediators for redundancy)   │
│  • Service registers with Mediator 2 (or multiple mediators)                 │
│  • Mediators can be the same server or different servers                     │
│  • Messages flow: Bridge → Mediator 1 → Mediator 2 → Service                 │
│  • ALL messages are end-to-end encrypted (mediators can't read them)         │
│  • Each component can use multiple mediators for high availability           │
│  • Default mediator: did:web:mediator-nlb.storm.ws:mediator:v1:.well-known   │
└──────────────────────────────────────────────────────────────────────────────┘
```
### Key Concepts for AI Agent Developers

**What is DIDComm?**
- Think of it like **encrypted email between services**
- Each service has a DID (Decentralized Identifier) - like an email address
- Messages are **end-to-end encrypted** - mediator can't read them
- Works through firewalls and NATs

**What is a Mediator?**
- Like an **email server** or **message router**
- Routes encrypted messages between DIDs
- **Cannot decrypt messages** - it's just a postman delivering sealed envelopes
- Each component (Bridge, Service) can register with **multiple mediators** for redundancy
- Components can share the same mediator or use different ones
- Default: `did:web:mediator-nlb.storm.ws:mediator:v1:.well-known`
- Can be self-hosted, use cloud services, or mix both

**Mediator Usage in This Architecture**:
1. **Bridge's Mediators**: Bridge registers with one or more mediators (e.g., Mediator 1)
2. **Service's Mediators**: Password Service registers with one or more mediators (e.g., Mediator 2)
3. **Can Use Same Mediator**: Both components can use the same mediator server, or different ones
4. **Message Flow**: Bridge → Mediator 1 → Mediator 2 → Service (all encrypted end-to-end)
5. **High Availability**: Each component can register with multiple mediators for redundancy

**Why This Architecture?**

From an AI agent perspective:
1. **Standard MCP**: Your agent uses normal MCP - no DIDComm knowledge needed
2. **⚠️ Current Demo Limitation**: MCP Server → Bridge uses HTTP (passwords in plaintext on localhost)
3. **DIDComm Backend**: Bridge → Service uses DIDComm (encrypted, decentralized)
4. **Mediator Network**: Each component can use multiple mediators for resilience
5. **Zero Trust**: Only the Password Service can decrypt passwords - mediators are untrusted routers

**🔒 Production Security**:
This demo shows the architecture separation, but the HTTP connection is **not production-ready**.
For production deployments, see [security-hardening.md](docs/security/security-hardening.md) which provides **3 secure alternatives**:
- **Option 1**: Eliminate network entirely (in-process MCP)
- **Option 2**: Replace HTTP with DIDComm E2E encryption
- **Option 3**: Secure HTTP with mTLS certificates

**Benefits**:
- ✅ **Standard MCP** - Works with Claude, Cline, any MCP client
- ✅ **Secure Backend** - DIDComm encryption for sensitive data
- ✅ **Scalable** - Multiple agents → one bridge → one vault
- ✅ **Decentralized** - No central authority, works through firewalls
- ✅ **High Availability** - Multiple mediators per component, automatic failover
- ✅ **Flexible Deployment** - Use shared or separate mediators per component

📖 **Full Details**: [architecture-flows.md](docs/architecture/architecture-flows.md)
🔒 **Security Improvements**: See [security-hardening.md](docs/security/security-hardening.md) for fixing the HTTP issue

## Components

### Core Services

1. **🔐 Password Service** (`service`)
   - Backend vault storing passwords
   - DIDComm-only access
   - [Source: `src/bin/service/`](src/bin/service/)

2. **💻 CLI Client** (`client`)
   - Direct password management
   - DIDComm client
   - [Source: `src/bin/client/`](src/bin/client/)

### MCP Architecture

3. **🌉 DIDComm Bridge** (`didcomm-bridge`)
   - Persistent service with fixed DID
   - HTTP API for MCP servers
   - DIDComm backend to password service
   - [Source: `src/bin/mcp_server/bridge_service.rs`](src/bin/mcp_server/bridge_service.rs)

4. **🤖 MCP Server** (`mcp-server`)
   - Standard MCP protocol (stdio/JSON-RPC)
   - HTTP client to bridge
   - Spawned by AI agents
   - [Source: `src/bin/mcp_server/http_client.rs`](src/bin/mcp_server/http_client.rs)

5. **🧪 Test Client** (`bridge-test-client`)
   - Validates complete flow
   - Tests MCP → Bridge → Service
   - [Source: `src/bin/mcp_server/test_client.rs`](src/bin/mcp_server/test_client.rs)

## Quick Start

### 3-Step Setup

#### Step 1: Start Password Service
```bash
# Terminal 1
cargo run --bin service
```
**→ Copy the Service DID** (e.g., `did:peer:2.Ez6LSghw...`)

**Note:** On first run, the setup wizard will help you configure passwords in `config.json`. These passwords are pre-configured and cannot be changed via the API.

---

#### Step 2: Start DIDComm Bridge
```bash
# Terminal 2
cargo run --bin didcomm-bridge -- --service-did "did:peer:2.Ez6LSghw..."
```
**→ Bridge DID is saved** and reused on restart

---

#### Step 3: Test It!
```bash
# Terminal 3 - Test via MCP (passwords are pre-configured in config.json)
cargo run --bin bridge-test-client -- \
  --bridge-url http://127.0.0.1:8080 \
  get-password --key myapp
```

**Expected Output**: `Password for 'myapp': secret123`

---

### Integration with AI Agents

#### Claude Desktop
Edit `claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "password-vault": {
      "command": "cargo",
      "args": ["run", "--bin", "mcp-server", "--", "--bridge-url", "http://127.0.0.1:8080"],
      "cwd": "/path/to/enterprise_pw_vault_demo"
    }
  }
}
```

#### MCP Inspector
```bash
npx @modelcontextprotocol/inspector \
  cargo run --bin mcp-server -- --bridge-url http://127.0.0.1:8080
```

---

## Available Commands

### Core Operations
```bash
# Start services
cargo run --bin service                              # Password vault
cargo run --bin didcomm-bridge -- --service-did <DID> # Bridge

# Password retrieval (passwords must be in config.json)
cargo run --bin client -- -s <DID> --password-key <KEY>

# Testing
cargo run --bin bridge-test-client -- list-tools
cargo run --bin bridge-test-client -- get-password --key <KEY>
```

### Bridge Management
```bash
# Custom port
cargo run --bin didcomm-bridge -- --service-did <DID> --port 9090

# Check health
curl http://127.0.0.1:8080/health

# Test bridge directly
curl -X POST http://127.0.0.1:8080/bridge \
  -H "Content-Type: application/json" \
  -d '{"type":"GetPassword","key":"myapp"}'
```

---

## Key Features

- ✅ **Architecture**: Clean MCP/DIDComm separation
- ✅ **Persistent Bridge DID**: Trusted, verifiable identity
- ✅ **End-to-End Encryption**: All passwords via DIDComm (Bridge → Service)
- ✅ **Decentralized**: No central authority
- ✅ **AI-Agent Ready**: Claude, MCP Inspector, custom agents
- ✅ **Scalable**: Multiple agents → one bridge
- ⚠️ **Demo Status**: Current HTTP connection needs hardening for production

**🔒 For Production**: Implement one of the [security hardening options](docs/security/security-hardening.md) to eliminate plaintext password transmission

---

## Documentation

### Architecture
- 📖 [Architecture Flows](docs/architecture/architecture-flows.md) - Component interactions and data flows
- 📖 [MCP/DIDComm Architecture](docs/architecture/mcp-didcomm-architecture.md) - Visual architecture guide

### Security
- 🔒 [Security Hardening](docs/security/security-hardening.md) - Complete security analysis
- 🔒 [Option 1: In-Process MCP Server](docs/options/option1-InProcess.md) - Maximum security
- 🔒 [Option 2: MCP over DIDComm Transport](docs/options/option2-McpDidcomm.md) - New approach ()
- 🔒 [Option 3: HTTPS + mTLS](docs/options/option3-HttpsMtls.md) - Traditional security
- 🔒 [Options Comparison](docs/options/optionsComparison.md) - Decision guide

---

## What's New

### Architecture (Current)
- ✅ **Separated concerns**: MCP protocol vs DIDComm security
- ✅ **Persistent bridge DID**: Saved and reused across restarts
- ✅ **Lightweight MCP servers**: No DIDComm overhead
- ✅ **Better scalability**: One bridge, many MCP servers
- ✅ **Centralized security**: All credentials in bridge

### Why This Matters
Traditional approaches bundled everything together. The bifurcated architecture:
1. **MCP Server** handles AI agent communication (stdio/JSON-RPC)
2. **Bridge** handles security and DIDComm (persistent, trusted)
3. **Service** stores passwords (backend)

This separation enables better security, monitoring, and scalability.

---

## License

MIT License - See LICENSE file for details

---

## Support

- 📖 [Architecture Flows](docs/architecture/architecture-flows.md)
- 📖 [MCP/DIDComm Architecture](docs/architecture/mcp-didcomm-architecture.md)
- 🔒 [Security Options](docs/options/optionsComparison.md)
- 💬 Open an issue for questions

**Built with**:
- [Affinidi TDK](https://github.com/affinidi/affinidi-tdk) - DIDComm implementation
- [rmcp](https://github.com/modelcontextprotocol/rust-sdk) - Official MCP SDK
