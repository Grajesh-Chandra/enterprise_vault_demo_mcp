# Option 3: HTTPS + mTLS

**Best for: Traditional enterprise security with certificate-based authentication**

---

## 🎯 Overview

Secure the existing bridge architecture using HTTPS with mutual TLS (mTLS). This approach uses traditional certificate-based authentication while moving DID ownership to the agent.

---

## 🏗️ Architecture

```
┌──────────────┐   stdio      ┌──────────────┐   HTTPS+mTLS ┌──────────────┐   DIDComm    ┌──────────────┐   DIDComm    ┌──────────────┐
│  AI Agent    │  JSON-RPC    │  MCP Server  │  encrypted   │   DIDComm    │  encrypted   │  Mediator    │  encrypted   │  Password    │
│  (Claude)    │ ─────────────│ (has agent   │ ─────────────│   Bridge     │ ─────────────│  (Cloud)     │ ─────────────│  Service     │
│              │              │  DID)        │   network    │ (relay only) │              │              │              │              │
└──────────────┘              └──────────────┘              └──────────────┘              └──────────────┘              └──────────────┘
                                      │                              │                              │                              │
                                      │     TLS Certificate          │                              │                              │
                                      │     Mutual Authentication    │                              │                              │
                                      └──────────────────────────────┘                              │                              │
                                                                                                    │                              │
                                                                                  Bridge & Service register with mediator(s)
                                                                                  Mediator routes E2E encrypted DIDComm messages
                                                                                  Can be same or different mediators
```

---

## ✅ Benefits

| Benefit | Description |
|---------|-------------|
| **TLS encryption** | Standard HTTPS protects transport layer (MCP ↔ Bridge) |
| **mTLS authentication** | Both client and server verify identities |
| **Agent owns DID** | MCP server manages agent's identity |
| **Bridge as relay** | Bridge doesn't decrypt messages |
| **Familiar ops model** | Traditional certificate management |
| **Auditable** | Standard HTTPS logging |
| **DIDComm backend** | Bridge ↔ Service uses mediators for routing |

---

## 🔒 Security Properties

### Encryption Layers
```
Application Layer:  MCP JSON-RPC
                    ↓
DIDComm Layer:      End-to-end encryption (via mediator routing)
                    ↓
TLS Layer:          Transport encryption + authentication (MCP ↔ Bridge only)
                    ↓
TCP Layer:          Network transport

Note: Bridge ↔ Service uses DIDComm via mediators (no TLS, just DIDComm)
```

### Key Features
- **TLS 1.3**: Modern encryption standards (MCP Server ↔ Bridge)
- **Certificate validation**: Both directions
- **Forward secrecy**: Ephemeral keys per session
- **Bridge isolation**: Cannot see plaintext
- **Standard monitoring**: HTTPS tooling
- **DIDComm via mediators**: Bridge ↔ Service communication
- **Dual security**: TLS for local + DIDComm for backend

---

## 📊 Comparison

| Aspect | HTTP (Current) | HTTPS + mTLS |
|--------|----------------|--------------||
| Transport encryption | ❌ Plaintext (local) | ✅ TLS 1.3 (local) |
| Authentication | ❌ None | ✅ Certificate-based |
| DID ownership | ❌ Bridge | ✅ Agent |
| Bridge role | ⚠️ Decrypts | ✅ Relay only |
| Backend communication | ✅ DIDComm via mediators | ✅ DIDComm via mediators |
| Ops complexity | ✅ Simple | ⚠️ Certificate mgmt |
| Performance | ✅ Fast | ⚠️ TLS overhead (local) |

---

## ⚠️ Considerations

### Pros
- ✅ Familiar security model
- ✅ Standard ops tooling
- ✅ TLS encryption
- ✅ Certificate-based auth
- ✅ Agent owns DID

### Cons
- ⚠️ Certificate management overhead
- ⚠️ Still has local network exposure (MCP ↔ Bridge)
- ⚠️ TLS performance overhead (local communication)
- ⚠️ Two processes to manage (three with mediator)
- ⚠️ Certificate rotation complexity
- ⚠️ Backend still requires mediator service

---

## 🎯 Best For

- **Traditional enterprises** with PKI infrastructure
- **Regulated industries** requiring standard protocols
- **Operations teams** familiar with TLS
- **Scenarios** where certificate management exists
- **Migration path** from current HTTP setup

---

## 🔗 Related Options

- [Option 1: In-Process MCP Server](option1-InProcess.md) - Maximum security
- [Option 2: MCP over DIDComm Transport](option2-McpDidcomm.md) - Native encryption

---

## 📚 Next Steps

1. Review certificate management processes
2. Generate test certificates
3. Update bridge with TLS
4. Test with mTLS client
5. Document certificate rotation
