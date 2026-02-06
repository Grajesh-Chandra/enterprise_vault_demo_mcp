# Option 1: In-Process MCP Server

**Best for: Maximum security with zero network exposure**

---

## 🎯 Overview

Embed the MCP server directly into the AI agent process, eliminating all network communication between the agent and MCP server. Only encrypted DIDComm messages leave the process boundary.

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────┐   DIDComm    ┌──────────────┐   DIDComm   ┌──────────────┐
│  AI Agent Process                       │  encrypted   │  Mediator    │  encrypted  │  Password    │
│  ┌──────────────┐   ┌──────────────┐    │ ──────────── │  (Cloud)     │ ─────────── │  Service     │
│  │  AI Agent    │──>│  MCP Server  │    │              │              │             │              │
│  │  (Claude)    │   │  (in-process │─── ┼─────────────>│  Routes E2E  │────────────>│              │
│  │              │   │   library)   │    │              │  encrypted   │             │              │
│  └──────────────┘   └──────────────┘    │              │  messages    │             │              │
│         │                  │            │              │              │             │              │
│         │                  ↓            │              │  (Cannot     │             │              │
│         │         ┌──────────────┐      │              │   decrypt)   │             │              │
│         └────────>│  Agent DID   │      │              │              │             │              │
│                   │  Key Manager │      │              └──────────────┘             │              │
│                   │  Registers   │      │                                           │              │
│                   │  w/ Mediator │      │                                           │              │
│                   └──────────────┘      │                                           └──────────────┘
└─────────────────────────────────────────┘

         Single Process Boundary
         No Network Calls (except DIDComm via mediator)
         OS-Level Memory Protection

Notes:
• Agent registers with mediator for message routing
• Service also registers with mediator (can be same or different)
• All messages end-to-end encrypted (mediator cannot decrypt)
• No plaintext data ever leaves agent process
```

---

## ✅ Benefits

| Benefit | Description |
|---------|-------------|
| **Zero network exposure** | MCP server runs inside agent's process, no local network calls |
| **OS-level protection** | Memory isolation via process boundaries |
| **No HTTP** | Direct function calls only |
| **Agent owns DID** | Full control over identity and keys |
| **Reduced attack surface** | Eliminates bridge as separate service |
| **Simplest deployment** | One process, one configuration |
| **Best performance** | No network latency for MCP calls, DIDComm via mediator |
| **Firewall friendly** | Works through mediators, no port forwarding needed |

---

## 🔒 Security Properties

### Process Memory Layout
```
Agent Process Memory Layout:
┌─────────────────────────────────────┐
│  Agent Code                         │
│                                     │
│  ┌───────────────────────────────┐ │
│  │  MCP Server (in-process)      │ │
│  │  - DIDComm client             │ │
│  │  - Registers with mediator    │ │
│  │  - No HTTP                    │ │
│  │  - Direct function calls      │ │
│  └───────────────────────────────┘ │
│                                     │
│  ┌───────────────────────────────┐ │
│  │  Agent DID Key Manager        │ │
│  │  - Private keys               │ │
│  │  - Encrypted at rest          │ │
│  │  - OS keychain integration    │ │
│  │  - Mediator DID configured    │ │
│  └───────────────────────────────┘ │
│                                     │
└─────────────────────────────────────┘
        ↕ OS Process Boundary
   No data leaves process except
   encrypted DIDComm messages
   (routed through mediator)

External Communication:
┌──────────────┐           ┌──────────────┐
│  Mediator    │ ◄────────►│  Password    │
│  (Cloud)     │  Routes   │  Service     │
│              │  E2E enc  │              │
└──────────────┘           └──────────────┘
```

---

## 📊 Comparison

| Aspect | Current (HTTP) | In-Process |
|--------|----------------|------------|
| Network exposure | ❌ HTTP localhost | ✅ None |
| Process boundary | ❌ Crosses | ✅ Same process |
| Attack surface | ❌ Large | ✅ Minimal |
| Performance | ⚠️ ~10-50ms | ✅ <1ms |
| Deployment | ⚠️ 2+ services | ✅ 1 process |
| Key management | ❌ Bridge owns | ✅ Agent owns |

---

## ⚠️ Considerations

### Pros
- ✅ Maximum security
- ✅ Best performance
- ✅ Simplest architecture
- ✅ Agent controls everything

### Cons
- ⚠️ Requires agent code changes
- ⚠️ Not suitable for sandboxed agents
- ⚠️ Tight coupling with agent
- ⚠️ Harder to update independently

---

## 🎯 Best For

- **Desktop applications** (Claude Desktop, custom agents)
- **Mobile apps** where security is critical
- **High-security environments** (finance, healthcare)
- **Performance-critical** applications
- **Scenarios** where agent has full control

---

## 🔗 Related Options

- [Option 2: MCP over DIDComm Transport](option2-McpDidcomm.md) - Keep processes separate, use DIDComm
- [Option 3: HTTPS + mTLS](option3-HttpsMtls.md) - Traditional security approach

---

## 📚 Next Steps

1. Review security requirements
2. Assess agent architecture compatibility
3. Create proof-of-concept with example
4. Test on target platforms (macOS/Windows/Linux)
5. Measure performance improvements
