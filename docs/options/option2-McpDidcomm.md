# Option 2: MCP over DIDComm Transport

**Best for: Standard MCP with native encryption and decentralization**

---

## 🎯 Overview

Extend MCP to support DIDComm as a native transport layer (like stdio, SSE, HTTP). This keeps standard MCP protocol compatibility while adding end-to-end encryption, DID-based authentication, and decentralized architecture.

---

## 🏗️ Architecture

```
┌──────────────┐   stdio      ┌──────────────┐   DIDComm    ┌──────────────┐   DIDComm    ┌──────────────┐   DIDComm    ┌──────────────┐
│  AI Agent    │  JSON-RPC    │  MCP Client  │  MCP-over-   │  Mediator 1  │  Routes E2E  │  Mediator 2  │  Business    │  Password    │
│  (Claude)    │ ──────────── │ (has agent   │ ─DIDComm──── │  (Client's)  │ ─encrypted─── │  (Server's)  │ ─DIDComm──── │  Service     │
│              │              │  DID)        │  encrypted   │              │   messages   │              │  encrypted   │              │
└──────────────┘              └──────────────┘              └──────────────┘              └──────────────┘              └──────────────┘
                                      │                              │                              │                              │
                                      │                              │      MCP Protocol            │                              │
                                      │                              │   (initialize, tools/list,   │                              │
                                      │                              │    tools/call, etc.)         │                              │
                                      │                              │   Wrapped in DIDComm         │                              │
                                      └──────────────────────────────┴──────────────────────────────┴──────────────────────────────┘

Note:
• MCP Client registers with Mediator 1
• MCP Tool Server registers with Mediator 2
• Password Service also registers with mediator(s)
• Mediators can be same or different servers
• All messages remain end-to-end encrypted through mediator routing
```

---

## ✅ Benefits

| Benefit | Description |
|---------|-------------|
| **Native encryption** | DIDComm provides end-to-end encryption |
| **DID-based authentication** | No certificates or API keys needed |
| **Standard MCP** | Full MCP protocol compatibility |
| **Decentralized** | No central authority required |
| **Firewall friendly** | Works through mediators, no port forwarding |
| **Agent owns DID** | Full control over identity |
| **Backward compatible** | Existing MCP tools work unchanged |
| **Verifiable** | All messages cryptographically signed |
| **High availability** | Multiple mediators for redundancy and failover |

---

## 🔒 Security Properties

### Transport Layers
```
Application Layer:  MCP JSON-RPC
                    ↓
Encryption Layer:   DIDComm (E2E encrypted)
                    ↓
Routing Layer:      Mediator 1 → Mediator 2
                    (Routes E2E encrypted messages)
                    (Cannot decrypt messages)
                    ↓
Transport Layer:    HTTPS/WebSocket
```

### Key Features
- **End-to-end encryption**: Only sender and receiver can decrypt
- **Forward secrecy**: Ephemeral keys for each session
- **Authentication**: Sender verified via DID signature
- **Non-repudiation**: All messages signed
- **Privacy**: DIDs don't reveal IP addresses
- **Mediator routing**: Messages routed through mediators (cannot decrypt)
- **High availability**: Multiple mediators per component for redundancy

---

## 📊 Comparison

| Aspect | HTTP Transport | DIDComm Transport |
|--------|----------------|-------------------|
| Encryption | ❌ Plaintext | ✅ E2E DIDComm |
| Authentication | ❌ None | ✅ DID-based |
| Identity | ⚠️ IP/port | ✅ Decentralized DID |
| Firewall | ❌ Needs open ports | ✅ Via mediators |
| Verifiable | ❌ No | ✅ Crypto signed |
| Privacy | ❌ IP exposed | ✅ DID only |
| Standard MCP | ✅ Yes | ✅ Yes |
| High Availability | ⚠️ Single endpoint | ✅ Multiple mediators |

---

## ⚠️ Considerations

### Pros
- ✅ Standard MCP protocol
- ✅ Native encryption
- ✅ Decentralized
- ✅ Works through firewalls
- ✅ Agent owns DID

### Cons
- ⚠️ Network latency (~200-400ms via mediators)
- ⚠️ Requires mediator service(s) for routing
- ⚠️ More complex than in-process
- ⚠️ DIDComm overhead
- ⚠️ Each component needs mediator registration

---

## 🎯 Best For

- **Production deployments** where scalability matters
- **Multi-tenant** scenarios
- **Compliance** requiring audit trails
- **Decentralized** architectures
- **Enterprise** requiring verifiable access
- **Scenarios** where agent and backend are separate

---

## 🔗 Related Options

- [Option 1: In-Process MCP Server](option1-InProcess.md) - Maximum security, zero network
- [Option 3: HTTPS + mTLS](option3-HttpsMtls.md) - Traditional security

---

## 📚 Next Steps

1. Review MCP protocol requirements
2. Implement DIDComm transport trait
3. Create proof-of-concept tool server
4. Test with example agent
5. Measure performance characteristics