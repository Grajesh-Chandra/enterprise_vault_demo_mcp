# MCP over DIDComm: A New Standard for Secure AI Agent Communication

**A repeatable pattern for enterprise-grade AI agent integration with end-to-end encryption and decentralized identity**

---

## 🎯 The Problem

AI agents (like Claude Desktop, custom agents) need to access enterprise backends securely. Current approaches have limitations:

| Approach | Issue |
|----------|-------|
| **Direct API calls** | Credentials exposed in agent config |
| **HTTP/REST** | Network vulnerabilities, MITM attacks |
| **MCP over HTTP** | Passwords in plaintext on localhost |
| **Custom protocols** | No standardization, hard to audit |

---

## 💡 The Solution: MCP over DIDComm

Combine two powerful standards:
- **MCP (Model Context Protocol)**: Standard for AI agent ↔ tool communication
- **DIDComm**: Secure, encrypted messaging with decentralized identity

**Result**: Secure, standardized, verifiable AI agent communication

---

## 🏗️ Architecture Overview

### High-Level Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     AI AGENT LAYER                                      │
│                                                                         │
│  ┌──────────────┐         ┌──────────────────────────────┐              │
│  │   AI Agent   │         │  Agent Identity Manager      │              │
│  │   (Claude,   │────────>│  - DID (did:peer:2.Ez6LS...) │              │
│  │   Custom)    │         │  - Keys (OS Keychain)        │              │
│  └──────┬───────┘         └──────────────────────────────┘              │
│         │                                                               │
│         │ stdio/JSON-RPC                                                │
│         │                                                               │
└─────────┼───────────────────────────────────────────────────────────────┘
          │
          ↓
┌─────────────────────────────────────────────────────────────────────────┐
│                   MCP CLIENT LAYER                                      │
│                                                                         │
│  ┌──────────────────────────────────────┐                               │
│  │  MCP Client (uses DIDComm transport) │                               │
│  │  - Reads agent DID from keychain     │                               │
│  │  - Wraps MCP in DIDComm envelopes    │                               │
│  │  - Registers with Mediator 1         │                               │
│  │  - Sends to tool server DID          │                               │
│  └──────────────┬───────────────────────┘                               │
│                 │                                                       │
│                 │ DIDComm (encrypted)                                   │
│                 │ Message type: https://mcp.didcomm.org/jsonrpc/1.0     │
│                 │                                                       │
└─────────────────┼───────────────────────────────────────────────────────┘
                  │
                  ↓
┌─────────────────────────────────────────────────────────────────────────┐
│                      MEDIATOR LAYER                                     │
│                                                                         │
│  ┌─────────────────┐                          ┌─────────────────┐       │
│  │   Mediator 1    │                          │   Mediator 2    │       │
│  │  (MCP Client's) │ ◄─── Routes E2E ──────► │ (Tool Server's) │        │
│  │                 │   Encrypted Messages     │                 │       │
│  └─────────────────┘   (Cannot decrypt)      └─────────────────┘        │
│                                                                         │
│  • Each component registers with mediator(s)                            │
│  • Can be same or different mediator servers                            │
│  • Messages remain end-to-end encrypted                                 │
│  • Mediators only route, never decrypt                                  │
│                                                                         │
└─────────────────┬───────────────────────────────────────────────────────┘
                  │
                  ↓
┌─────────────────┼────────────────────────────────────────────────────────┐
│                 │          MCP TOOL SERVER LAYER                         │
│                 │                                                        │
│  ┌──────────────┴──────────────────────────────┐                         │
│  │  MCP Tool Server (listens via DIDComm)     │                          │
│  │  - Has own DID (did:peer:2.Ez6LS...)       │                          │
│  │  - Registers with Mediator 2                │                         │
│  │  - Receives MCP wrapped in DIDComm         │                          │
│  │  - Unwraps, processes with ServerHandler   │                          │
│  │  - Returns MCP response via DIDComm        │                          │
│  └──────────────┬──────────────────────────────┘                         │
│                 │                                                        │
│                 │ Internal DIDComm (to backend via mediators)            │
│                 │                                                        │
└─────────────────┼────────────────────────────────────────────────────────┘
                  │
                  ↓
┌─────────────────┼─────────────────────────────────────────────────────────┐
│                 │         ENTERPRISE BACKEND                              │
│                 │                                                         │
│  ┌──────────────┴──────────────────────┐                                  │
│  │  Password Service / Business Logic  │                                  │
│  │  - Has own DID                       │                                 │
│  │  - Registers with mediator(s)        │                                 │
│  │  - DIDComm only access               │                                 │
│  │  - Pre-configured data               │                                 │
│  └──────────────────────────────────────┘                                 │
│                                                                           │
└───────────────────────────────────────────────────────────────────────────┘
```

---

## 🔄 Message Flow Diagram

### Step-by-Step Communication

```
Agent         MCP Client    Mediator 1    Mediator 2    Tool Server       Backend
  │               │               │             │             │                │
  │               │               │             │             │                │
  ├─ 1. Init ────>│               │             │             │                │
  │  (stdio)      │               │             │             │                │
  │               │               │             │             │                │
  │               ├─ 2. DIDComm ─>│             │             │                │
  │               │  (encrypted)  │             │             │                │
  │               │               │             │             │                │
  │               │               ├─ 3. Route ─>│             │                │
  │               │               │  (E2E enc)  │             │                │
  │               │               │             │             │                │
  │               │               │             ├─ 4. Route ─>│                │
  │               │               │             │             │                │
  │               │               │             │<─ 5. Resp ──┤                │
  │               │               │             │  (E2E enc)  │                │
  │               │               │             │             │                │
  │               │               │<─ 6. Route ─┤             │                │
  │               │               │             │             │                │
  │               │<─ 7. DIDComm  ┤             │             │                │
  │<─ 8. Result ──┤               │             │             │                │
  │  (stdio)      │               │             │             │                │
  │               │               │             │             │                │
  ├─ 9. Call ────>│               │             │             │                │
  │  get_password │               │             │             │                │
  │               │               │             │             │                │
  │               ├─10. DIDComm ─>│             │             │                │
  │               │  tools/call   │             │             │                │
  │               │               │             │             │                │
  │               │               ├─11. Route ─>│             │                │
  │               │               │             │             │                │
  │               │               │             ├─12. Route ─>│                │
  │               │               │             │             │                │
  │               │               │             │             ├─13. Query ────>│
  │               │               │             │             │   (DIDComm)    │
  │               │               │             │             │                │
  │               │               │             │             │<─14. Data ─────┤
  │               │               │             │             │   (encrypted)  │
  │               │               │             │             │                │
  │               │               │             │<─15. Resp ──┤                │
  │               │               │             │             │                │
  │               │               │<─16. Route ─┤             │                │
  │               │               │             │             │                │
  │               │<─17. DIDComm  ┤             │             │                │
  │<─18. Result ──┤               │             │             │                │
  │  Password!    │               │             │             │                │
  │               │               │             │             │                │
```

### Legend
- **Solid lines** (───): Standard communication
- **DIDComm**: End-to-end encrypted, authenticated
- **stdio/JSON-RPC**: Local process communication
- **Mediator 1**: Routes messages for MCP Client (cannot decrypt)
- **Mediator 2**: Routes messages for Tool Server (cannot decrypt)
- **E2E enc**: End-to-end encrypted from MCP Client to Tool Server

### Key Points
- MCP Client registers with Mediator 1
- Tool Server registers with Mediator 2
- Can be same or different mediator servers
- Messages remain encrypted through entire routing path
- Backend also uses mediator(s) for DIDComm communication

---

## 📦 Component Diagram

### Three-Layer Architecture

```
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ LAYER 1: AI AGENT                                                    ┃
┃ ┌──────────────────────────────────────────────────────────────────┐ ┃
┃ │ Claude Desktop / Custom Agent                                    │ ┃
┃ │                                                                  │ ┃
┃ │ Responsibilities:                                                │ ┃
┃ │ • User interaction                                               │ ┃
┃ │ • Generate MCP requests                                          │ ┃
┃ │ • Manage agent DID (create, store in OS keychain)                │ ┃
┃ │ • Configure MCP servers                                          │ ┃
┃ │                                                                  │ ┃
┃ │ Config:                                                          │ ┃
┃ │ {                                                                │ ┃
┃ │   "mcpServers": {                                                │ ┃
┃ │     "password-vault": {                                          │ ┃
┃ │       "transport": "didcomm",                                    │ ┃
┃ │       "toolServerDid": "did:peer:2.Ez6LS...",                    │ ┃
┃ │       "mediator": "did:web:mediator-nlb.storm.ws:mediator:v1"    │ ┃
┃ │       // Can specify multiple mediators for redundancy           │ ┃
┃ │     }                                                            │ ┃
┃ │   }                                                              │ ┃
┃ │ }                                                                │ ┃
┃ └──────────────────────────────────────────────────────────────────┘ ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
                                    │
                                    │ stdio/JSON-RPC
                                    ↓
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ LAYER 2: MCP CLIENT (Agent-Side)                                     ┃
┃ ┌──────────────────────────────────────────────────────────────────┐ ┃
┃ │ MCP Client with DIDComm Transport                                │ ┃
┃ │                                                                  │ ┃
┃ │ Responsibilities:                                                │ ┃
┃ │ • Implement MCP protocol (initialize, tools/list, tools/call)    │ ┃
┃ │ • Load agent DID from keychain                                   │ ┃
┃ │ • Register with mediator(s) for message routing                  │ ┃
┃ │ • Wrap MCP JSON-RPC in DIDComm envelopes                         │ ┃
┃ │ • Encrypt using agent's keys                                     │ ┃
┃ │ • Send to tool server DID via mediator                           │ ┃
┃ │ • Receive and unwrap DIDComm responses via mediator              │ ┃
┃ │                                                                  │ ┃
┃ │ Code:                                                            │ ┃
┃ │ let transport = DIDCommTransport::new(                           │ ┃
┃ │     agent_did,                                                   │ ┃
┃ │     agent_secrets,                                               │ ┃
┃ │     tool_server_did,                                             │ ┃
┃ │     mediator                                                     │ ┃
┃ │ );                                                               │ ┃
┃ │ let service = mcp_client.serve(transport).await?;                │ ┃
┃ └──────────────────────────────────────────────────────────────────┘ ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
                                    │
                                    │ DIDComm (E2E encrypted)
                                    │ Type: https://mcp.didcomm.org/jsonrpc/1.0
                                    ↓
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ LAYER 3: MCP TOOL SERVER (Enterprise-Side)                           ┃
┃ ┌──────────────────────────────────────────────────────────────────┐ ┃
┃ │ MCP Tool Server with DIDComm Listener                            │ ┃
┃ │                                                                  │ ┃
┃ │ Responsibilities:                                                │ ┃
┃ │ • Register with mediator(s) for incoming messages                │ ┃
┃ │ • Listen for DIDComm messages via mediator                       │ ┃
┃ │ • Verify sender DID (authentication)                             │ ┃
┃ │ • Decrypt DIDComm envelope                                       │ ┃
┃ │ • Extract MCP JSON-RPC payload                                   │ ┃
┃ │ • Process with standard ServerHandler                            │ ┃
┃ │ • Call backend services via DIDComm (through mediators)          │ ┃
┃ │ • Wrap response in DIDComm                                       │ ┃
┃ │ • Send back to requesting agent DID via mediator                 │ ┃
┃ │                                                                  │ ┃
┃ │ Tools Exposed:                                                   │ ┃
┃ │ • get_password(key: string)                                      │ ┃
┃ │ • list_keys()                                                    │ ┃
┃ │ • [custom enterprise tools]                                      │ ┃
┃ └──────────────────────────────────────────────────────────────────┘ ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
                                    │
                                    │ DIDComm (business protocol)
                                    ↓
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ LAYER 4: ENTERPRISE BACKEND                                          ┃
┃ ┌──────────────────────────────────────────────────────────────────┐ ┃
┃ │ Password Service / Database / Business Logic                     │ ┃
┃ │                                                                  │ ┃
┃ │ • Registers with mediator(s) for message routing                 │ ┃
┃ │ • DIDComm-only access (via mediators)                            │ ┃
┃ │ • No HTTP endpoints                                              │ ┃
┃ │ • Zero trust - all requests authenticated via DID                │ ┃
┃ │ • Can use same or different mediators as Tool Server             │ ┃
┃ └──────────────────────────────────────────────────────────────────┘ ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
```

---

## 🔐 Security Stack Visualization

### Defense in Depth

```
┌─────────────────────────────────────────────────────────────────────┐
│                    SECURITY LAYERS                                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Layer 1: Identity (DID)                                            │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ • Decentralized Identifiers (did:peer)                     │     │
│  │ • No central authority                                     │     │
│  │ • Agent controls private keys                              │     │
│  │ • Cryptographically verifiable                             │     │
│  └────────────────────────────────────────────────────────────┘     │
│                            ↓                                        │
│  Layer 2: Authentication                                            │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ • DID-based authentication (no passwords)                  │     │
│  │ • Every message cryptographically signed                   │     │
│  │ • Sender verification automatic                            │     │
│  │ • No shared secrets to compromise                          │     │
│  └────────────────────────────────────────────────────────────┘     │
│                            ↓                                        │
│  Layer 3: Encryption (DIDComm)                                      │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ • End-to-end encryption                                    │     │
│  │ • Forward secrecy                                          │     │
│  │ • Mediators route but cannot decrypt messages              │     │
│  │ • Multiple mediators for high availability                 │     │
│  │ • Multiple encryption algorithms supported                 │     │
│  └────────────────────────────────────────────────────────────┘     │
│                            ↓                                        │
│  Layer 4: Protocol (MCP)                                            │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ • Standard JSON-RPC 2.0                                    │     │
│  │ • Well-defined capabilities                                │     │
│  │ • Tool-based access control                                │     │
│  │ • Auditable requests/responses                             │     │
│  └────────────────────────────────────────────────────────────┘     │
│                            ↓                                        │
│  Layer 5: Key Management                                            │
│  ┌────────────────────────────────────────────────────────────┐     │
│  │ • OS-level key storage (Keychain/DPAPI/Secret Service)     │     │
│  │ • Never in config files                                    │     │
│  │ • Encrypted at rest                                        │     │
│  │ • Hardware security module support                         │     │
│  └────────────────────────────────────────────────────────────┘     │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 📊 Comparison: Traditional vs MCP over DIDComm

### Traditional HTTP/REST Approach

```
Agent ────[HTTP/API Key]───> Enterprise API
              │
              │ Issues:
              │ • API key in config file
              │ • No encryption (or TLS only)
              │ • Centralized authentication
              │ • IP-based access control
              │ • Credentials can be copied
```

### MCP over HTTP (Current)

```
Agent ──[stdio/JSON-RPC]──> MCP Server ──[HTTP plaintext]──> Bridge ──[DIDComm]──> Backend
                                              │
                                              │ Issues:
                                              │ • Passwords in plaintext
                                              │ • Local network exposure
                                              │ • No authentication
                                              │ • MITM possible
```

### MCP over DIDComm (Proposed)

```
Agent ──[stdio]──> MCP Client ──[DIDComm]──> Mediator 1 ──> Mediator 2 ──[DIDComm]──> Tool Server ──> Backend
                                      │                                           │
                                      │ Benefits:                                 │
                                      │ ✅ End-to-end encryption                   │
                                      │ ✅ DID-based auth                          │
                                      │ ✅ No plaintext                            │
                                      │ ✅ Decentralized                           │
                                      │ ✅ Verifiable                              │
                                      │ ✅ Firewall friendly (via mediators)      │
                                      │ ✅ High availability (multiple mediators) │
```

---

## 🎯 Key Benefits Matrix

| Feature | Traditional API | MCP/HTTP | MCP/DIDComm |
|---------|----------------|----------|-------------|
| **Standard Protocol** | ❌ Custom | ✅ MCP | ✅ MCP |
| **Encryption** | ⚠️ TLS only | ❌ Plaintext | ✅ E2E DIDComm |
| **Authentication** | ⚠️ API keys | ❌ None | ✅ DID-based |
| **Decentralized** | ❌ Central auth | ❌ No | ✅ Yes |
| **Firewall Friendly** | ❌ Needs ports | ❌ Needs ports | ✅ Via mediator |
| **Credential Exposure** | ❌ In config | ❌ On network | ✅ Never exposed |
| **Verifiable** | ❌ No | ❌ No | ✅ Crypto signed |
| **Key Management** | ⚠️ Files | ⚠️ Files | ✅ OS keychain |
| **Zero Trust** | ⚠️ Partial | ❌ No | ✅ Yes |
| **Auditability** | ⚠️ Server logs | ⚠️ Distributed | ✅ Crypto proof |

---

## 🔗 Standards & Specifications

### Based On

1. **MCP (Model Context Protocol)**
   - Spec: https://spec.modelcontextprotocol.io
   - JSON-RPC 2.0 based
   - Tool-based architecture

2. **DIDComm Messaging**
   - Spec: https://identity.foundation/didcomm-messaging/spec/
   - W3C DID standard
   - End-to-end encrypted messaging

3. **DID (Decentralized Identifiers)**
   - Spec: https://www.w3.org/TR/did-core/
   - W3C Recommendation
   - Decentralized identity

### New Contribution

**MCP over DIDComm Transport**
- Message Type: `https://mcp.didcomm.org/jsonrpc/1.0`
- Wraps MCP JSON-RPC in DIDComm envelopes
- Maintains full MCP protocol compatibility
- Adds DIDComm security properties

---

## 💼 Business Benefits

### For Enterprises
- ✅ **Security**: End-to-end encryption, zero-trust
- ✅ **Compliance**: Full audit trail, verifiable access
- ✅ **Cost**: No VPN, no certificate management
- ✅ **Flexibility**: Works across clouds, firewalls
- ✅ **Scalability**: Decentralized architecture

### For Agent Developers
- ✅ **Standards**: Build once, work everywhere
- ✅ **Security**: No credential management burden
- ✅ **Discovery**: Find tools via DID resolution
- ✅ **Trust**: Verify enterprise identity
- ✅ **Privacy**: User controls their DID

### For End Users
- ✅ **Security**: Credentials never exposed
- ✅ **Privacy**: Decentralized identity
- ✅ **Control**: Own your agent's identity
- ✅ **Transparency**: Auditable interactions
- ✅ **Reliability**: No single point of failure

---

## 📚 Next Steps

### To Learn More
- [Security Hardening](../security/security-hardening.md) - Full technical specification
- [Architecture Flows](architecture-flows.md) - Detailed component flows
- [README](../../README.md) - Quick start guide

### To Implement
1. Clone repository: `git clone <repo>`
2. Review security proposal
3. Choose deployment topology
4. Follow integration checklist
5. Deploy and test

### To Contribute
- Open issues for questions
- Submit PRs for improvements
- Share your use case
- Help document patterns

---

## 🎉 Summary

**MCP over DIDComm = Secure + Standard + Decentralized AI Agent Communication**

This pattern provides:
- ✅ **Security** through DIDComm encryption
- ✅ **Standards** via MCP protocol
- ✅ **Decentralization** with DIDs
- ✅ **Simplicity** for both enterprises and agents
- ✅ **Scalability** for production deployments

**Ready to build secure AI agent integrations for your enterprise!**

---

*Built with ❤️ using [Affinidi TDK](https://github.com/affinidi/affinidi-tdk) and [rmcp](https://github.com/modelcontextprotocol/rust-sdk)*
