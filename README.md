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
