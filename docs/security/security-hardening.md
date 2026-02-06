# Security Hardening Proposal

This document outlines security improvements for the MCP architecture, focusing on eliminating network vulnerabilities and proper key management.

## 🎯 Executive Summary

**Current Issue:** MCP server communicates with bridge via plaintext HTTP, exposing passwords on the network.

**Recommended Solution:** **MCP over DIDComm Transport**
- Native DIDComm encryption (end-to-end)
- DID-based authentication (no certificates needed)
- Standard MCP protocol compatibility
- Agent owns its DID and keys
- Works through firewalls via mediators
- Mediators route messages but cannot decrypt (E2E encrypted)
- Decentralized and verifiable
- High availability with multiple mediators

**Alternative Solutions:**
1. **In-Process MCP Server** - Maximum security, zero network exposure
2. **HTTPS + mTLS** - Traditional approach, certificate-based

**Key Innovation:** Extend MCP to support DIDComm as a native transport layer (like stdio, SSE, HTTP), enabling secure, decentralized AI agent communication.

---

## 🔒 Current Security Model (As-Is)

### Architecture
```
┌──────────────┐   stdio      ┌──────────────┐   HTTP       ┌──────────────┐   DIDComm    ┌──────────────┐   DIDComm    ┌──────────────┐
│  AI Agent    │  JSON-RPC    │  MCP Server  │  plaintext   │   DIDComm    │  encrypted   │  Mediator    │  encrypted   │  Password    │
│  (Claude)    │ ──────────── │ (separate    │ ─────────────│   Bridge     │ ─────────────│  (Cloud)     │ ─────────────│  Service     │
│              │              │  process)    │   network    │ (persistent) │              │              │              │              │
└──────────────┘              └──────────────┘              └──────────────┘              └──────────────┘              └──────────────┘

Note: Bridge & Service register with mediator(s) for DIDComm message routing
```

### Current Vulnerabilities

#### ❌ **Critical: Unencrypted HTTP**
```rust
// Current implementation in mcp-server
let response = self.client
    .post(format!("{}/bridge", self.bridge_url))  // HTTP plaintext!
    .json(&request)                                // Passwords in clear text
    .send()
    .await?;
```

**Risk:**
- Passwords transmitted in plaintext over HTTP
- Vulnerable to man-in-the-middle attacks
- Local network sniffing can capture credentials
- No authentication between MCP server and bridge

#### ❌ **Moderate: Process Separation**
- MCP server runs as separate process
- Inter-process communication via network stack
- Potential for process injection/debugging
- No shared memory protection

#### ❌ **Moderate: Key Management Split**
- Bridge owns persistent DID and keys
- Agent has no control over its identity
- Trust delegation to bridge process
- Cannot audit bridge behavior

---

## 🛡️ Proposed Security Options

We have designed three comprehensive security options to address the vulnerabilities. Each option is fully documented in separate files:

### 📋 Security Options Overview

#### [Option 1: In-Process MCP Server](../options/option1-InProcess.md)
**Best for: Maximum security with zero network exposure**

- ✅ **No local network exposure** - MCP server runs in agent's process
- ✅ **OS-level protection** - Memory isolation via process boundaries
- ✅ **No HTTP** - Direct function calls
- ✅ **Agent owns DID** - Full control over identity and keys
- ✅ **Reduced attack surface** - Eliminate bridge as separate service
- ✅ **Simpler deployment** - One process, one configuration
- ✅ **Better performance** - No network latency for MCP calls
- ✅ **Firewall friendly** - DIDComm to backend via mediators

**[→ View Full Implementation Guide](../options/option1-InProcess.md)**

---

#### [Option 2: MCP over DIDComm Transport](../options/option2-McpDidcomm.md)
**Best for: Standard MCP with native encryption and decentralization**

- ✅ **Native encryption** - DIDComm provides end-to-end encryption
- ✅ **DID-based authentication** - No certificates or API keys needed
- ✅ **Standard MCP** - Full MCP protocol compatibility
- ✅ **Decentralized** - No central authority required
- ✅ **Firewall friendly** - Works through mediators
- ✅ **High availability** - Multiple mediators for redundancy
- ✅ **Verifiable** - All messages cryptographically signed
- ✅ **Mediator routing** - Messages routed but never decrypted by mediators

**[→ View Full Implementation Guide](../options/option2-McpDidcomm.md)**

---

#### [Option 3: HTTPS + mTLS](../options/option3-HttpsMtls.md)
**Best for: Traditional enterprise security with certificate-based authentication**

- ✅ **TLS encryption** - Standard HTTPS protects transport layer (local only)
- ✅ **mTLS authentication** - Both client and server verify identities
- ✅ **Agent owns DID** - MCP server manages agent's identity
- ✅ **Familiar ops model** - Traditional certificate management
- ✅ **Auditable** - Standard HTTPS logging
- ✅ **DIDComm backend** - Bridge to Service uses mediators

**[→ View Full Implementation Guide](../options/option3-HttpsMtls.md)**

---

## 📊 Quick Comparison

For a detailed side-by-side comparison of all three options, see **[Options Comparison Guide](../options/optionsComparison.md)**.

### Summary

| Feature | Option 1 | Option 2 | Option 3 |
|---------|----------|----------|----------|
| **Security** | Maximum | High | Medium |
| **Network** | None | DIDComm E2E | HTTPS+mTLS |
| **Complexity** | Medium | Medium | Low |
| **Performance** | Best | Good | Good |
| **Best For** | Desktop apps | Production | Quick migration |

---

## 🔑 Key Management Architecture

### Proposed: Agent-Managed DIDs

All three options move DID ownership to the agent, ensuring proper key custody and control.

**Key Principles:**
- Agent creates and owns its DID
- Secrets stored in OS keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- MCP servers/bridges only relay messages
- Full audit trail of all cryptographic operations

For detailed key management implementation, see individual option guides.

---

## 📊 Security Comparison

| Aspect | Current | With TLS | MCP/DIDComm | In-Process |
|--------|---------|----------|-------------|------------|
| **Transport Security** | ❌ Plaintext HTTP | ✅ HTTPS + mTLS (local) | ✅ E2E DIDComm | ✅ N/A (no local network) |
| **Password Exposure** | ❌ Network plaintext | ✅ TLS encrypted | ✅ DIDComm encrypted | ✅ In-memory only |
| **DID Ownership** | ❌ Bridge owns | ✅ Agent owns | ✅ Agent owns | ✅ Agent owns |
| **Attack Surface** | ❌ Large (HTTP + Bridge) | ⚠️ Medium (HTTPS + Bridge) | ✅ Small (DIDComm only) | ✅ Smallest (process only) |
| **Network Exposure** | ❌ Yes (plaintext) | ⚠️ Yes (TLS local) | ⚠️ Yes (E2E encrypted) | ✅ Backend only (via mediators) |
| **Process Boundary** | ❌ Crosses | ⚠️ Crosses | ⚠️ Crosses | ✅ Same process |
| **Key Management** | ❌ Bridge controls | ✅ Agent controls | ✅ Agent controls | ✅ Agent controls |
| **Authentication** | ❌ None | ✅ mTLS certs | ✅ DID-based crypto | ✅ N/A |
| **Decentralized** | ❌ No | ⚠️ Partial (backend) | ✅ Yes | ✅ Yes (if desired) |
| **Standard MCP** | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes |
| **Performance** | ⚠️ Network latency | ⚠️ TLS overhead (local) | ⚠️ Via mediators | ✅ Function call |
| **Deployment** | ⚠️ 2 services + mediator | ⚠️ 2 services + certs + mediator | ⚠️ 2 services + mediators | ✅ 1 process + mediator |
| **Firewall Friendly** | ⚠️ Requires ports | ⚠️ Requires ports (local) | ✅ Via mediators | ✅ Via mediators |
| **Mediator Usage** | ✅ Backend only | ✅ Backend only | ✅ All DIDComm | ✅ Backend only |
| **Verifiable** | ❌ No | ⚠️ Cert-based | ✅ Crypto signatures | ✅ N/A |

---

## 🎯 Recommendations

### Recommended Approach Comparison

**Best for Maximum Security:** In-Process MCP Server (Option 1)
- Zero network exposure
- OS-level memory protection
- Simplest deployment

**Best for Standard MCP Compatibility:** MCP over DIDComm Transport (Option 2)
- Standard MCP protocol
- Native DIDComm encryption and authentication
- Decentralized identity
- Works through firewalls via mediator
- **Recommended for production enterprise deployments**

**Best for Legacy Systems:** HTTPS + mTLS (Option 3)
- Traditional security model
- Certificate-based authentication
- Familiar to ops teams

---

## 📖 Related Documentation

### Detailed Implementation Guides
- [Option 1: In-Process MCP Server](../options/option1-InProcess.md) - Complete implementation with code examples
- [Option 2: MCP over DIDComm Transport](../options/option2-McpDidcomm.md) - DIDComm transport specification
- [Option 3: HTTPS + mTLS](../options/option3-HttpsMtls.md) - Traditional security approach
- [Options Comparison](../options/optionsComparison.md) - Decision guide and comparison matrix

### Architecture Documentation
- [Architecture Flows](../architecture/architecture-flows.md) - Current architecture and data flows
- [MCP/DIDComm Architecture](../architecture/mcp-didcomm-architecture.md) - Visual architecture guide

### Project Documentation
- [README.md](../../README.md) - Quick start guide

---

## 💡 Key Insights

### Security Findings
1. **Current vulnerability**: HTTP transmits passwords in plaintext on localhost
2. **Agent should own DID**: Better security model, proper key custody
3. **Multiple solutions available**: Three viable options with different trade-offs

### Recommendations
- **For maximum security**: [Option 1 - In-Process](../options/option1-InProcess.md)
- **For production deployments**: [Option 2 - MCP/DIDComm Transport](../options/option2-McpDidcomm.md) ⭐ Recommended
- **For quick migration**: [Option 3 - HTTPS + mTLS](../options/option3-HttpsMtls.md)

### Benefits of Proposed Solutions
- All options eliminate plaintext password transmission
- Agent controls its own identity and keys
- Maintains MCP protocol compatibility
- Incremental migration path available
- DIDComm end-to-end encryption preserved
