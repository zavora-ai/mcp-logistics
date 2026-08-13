# Changelog

## [1.2.0] - 2026-08-13

### Changed
- Upgraded to rmcp 3.1.2 and raised the minimum supported Rust version to 1.94.1.
- Added MCP 2026-07-28 stateless request handling while retaining MCP 2025-11-25 initialization compatibility.

### Added
- Per-request identity and protocol metadata, on-demand discovery/cache hints, and the configured Tasks and sealed MRTR approval policies.

## [1.1.0] - 2026-05-25

### Removed
- Sendy backend (company collapsed)

### Changed
- Default feature: shipengine only
- 3 backends: ShipEngine, Shippo, Google Maps

## [1.0.0] - 2026-05-25

### Added
- Initial release with 16 tools across 6 categories
- **4 backends:** ShipEngine (US/EU multi-carrier), Shippo, Sendy (Africa), Google Maps
- Shipments: create, list, get, cancel
- Rates: get_rates, compare_rates, list_carriers, book_rate
- Tracking: track_shipment, get_tracking_events
- Labels: generate_label, get_label
- Addresses: validate_address
- Routes: optimize_route, get_eta, get_distance
- Feature flags — default: shipengine + sendy
- Manifest validation and health check on startup
