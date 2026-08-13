# Logistics MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-logistics.svg)](https://crates.io/crates/mcp-logistics)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)
[![Registry Ready](https://img.shields.io/badge/ADK_Registry-Ready-green.svg)](https://www.zavora.ai)

Unified logistics MCP server with **16 tools** across **3 backends** — ShipEngine (FedEx/UPS/DHL/USPS), Shippo, and Google Maps route optimization. Shipments, rates, tracking, labels, address validation, and delivery route planning.

## Key Principles

- **Multi-carrier** — compare rates across FedEx, UPS, DHL, USPS via ShipEngine or Shippo
- **Route optimization** — Google Maps for multi-stop delivery planning and ETAs
- **Full lifecycle** — create shipment → get rates → book → track → deliver
- **No credential exposure** — API keys stay in env vars
- **Single binary** — no Node.js, no Python

## Tools (16)

| Category | Tools | Risk |
|----------|-------|------|
| Shipments | `create_shipment`, `list_shipments`, `get_shipment`, `cancel_shipment` | read / external_write |
| Rates | `get_rates`, `compare_rates`, `list_carriers`, `book_rate` | read / financial_action |
| Tracking | `track_shipment`, `get_tracking_events` | read_only |
| Labels | `generate_label`, `get_label` | read / internal_write |
| Addresses | `validate_address` | read_only |
| Routes | `optimize_route`, `get_eta`, `get_distance` | read_only |

## Backends

| Backend | Region | Use Case | Default |
|---------|--------|----------|:---:|
| **ShipEngine** | US, EU, Global | Multi-carrier (FedEx, UPS, DHL, USPS) | ✅ |
| **Shippo** | US, EU, Global | Alternative multi-carrier | ❌ |
| **Google Maps** | Global | Route optimization, ETAs, distance | ❌ |

## Installation

```bash
cargo install mcp-logistics --features all-backends
```

### Feature flags

```bash
# Default: ShipEngine
cargo install mcp-logistics

# All backends
cargo install mcp-logistics --features all-backends

# Specific
cargo install mcp-logistics --no-default-features --features "shippo,routes"
```

## Configuration

### ShipEngine
```bash
export SHIPENGINE_API_KEY="TEST_xxx"
```

### Shippo
```bash
export SHIPPO_TOKEN="shippo_test_xxx"
```

### Google Maps (route optimization)
```bash
export GOOGLE_MAPS_KEY="AIzaSy..."
```

## Client Configuration

```json
{
  "mcpServers": {
    "logistics": {
      "command": "mcp-logistics",
      "args": [],
      "env": {
        "SHIPENGINE_API_KEY": "TEST_xxx",
        "GOOGLE_MAPS_KEY": "AIzaSy..."
      }
    }
  }
}
```

## Usage Examples

```
"Get shipping rates from New York to Los Angeles for a 5kg package"
→ get_rates(origin: {...}, destination: {...}, parcels: [{weight_kg: 5}])

"Compare rates sorted by price"
→ compare_rates(..., sort_by: "price")

"Book the cheapest rate"
→ book_rate(id: "rate-123")

"Track my shipment"
→ track_shipment(id: "ship-456")

"Optimize delivery route for 5 stops"
→ optimize_route(stops: ["Stop A", "Stop B", "Stop C", "Stop D", "Stop E"])

"How far is it from Nairobi to Mombasa?"
→ get_distance(origin: "Nairobi, Kenya", destination: "Mombasa, Kenya")

"Validate this shipping address"
→ validate_address(address: {street1: "123 Main St", city: "New York", state: "NY", postal_code: "10001", country: "US"})
```

## Registry Compliance

- **HealthCheck** — verifies shipping backend connectivity
- **mcp-server.toml** — 16 tools with risk classes
- **Manifest validation** — startup fails fast on invalid manifest
- **Structured tracing** — `RUST_LOG` env-filter

## Contributors

| [<img src="https://github.com/jkmaina.png" width="80px;" alt=""/><br /><sub><b>James Karanja Maina</b></sub>](https://github.com/jkmaina) |
|:---:|

## License

Apache-2.0

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

## rmcp and MCP compatibility

This server is built with [`rmcp` 3.1.2](https://github.com/modelcontextprotocol/rust-sdk/releases/tag/rmcp-v3.1.2) and requires Rust 1.94.1 or newer. The rmcp 3 rollout retains legacy MCP initialization compatibility and targets MCP protocol revisions `2025-11-25` and `2026-07-28`.

## MCP 2026-07-28 rollout (P4 workflow/business)

This server uses `rmcp` 3.1.2 and `adk-mcp-sdk` 0.2 with a minimum supported
Rust version of **1.94.1**. It accepts stateless MCP 2026 requests with
per-request protocol, client identity, and capability metadata while retaining
the legacy MCP 2025-11-25 initialize flow for ordinary tools.

- **Tasks:** `generate_label`, `optimize_route`
- **MRTR approvals:** `create_shipment`, `cancel_shipment`, `book_rate`
- **Discovery and routing:** rmcp serves on-demand discovery and validates the
  per-request protocol envelope; HTTP deployments can route with `Mcp-Method`
  and `Mcp-Name`. The packaged binary currently uses stdio.
- **Caching:** `tools/list` returns a public `ttlMs` of 60,000 for MCP 2026;
  rmcp omits the cache fields for legacy clients.
- **Deprecated extensions:** this server does not add new Roots, Sampling, or
  dynamic client-registration dependencies.

Protected tools require `MCP_REQUEST_STATE_KEY` with at least 32 high-entropy
bytes. All replicas must share that key so sealed approval state can resume on
another instance. Approval state is bound to the client identity, tool, and
arguments and expires after two minutes. Missing identity, invalid state,
rejection, or legacy protocol use fails closed. Task records are process-local
for the current stdio runtime; use a durable task store before deploying the
server behind scale-to-zero HTTP infrastructure.
