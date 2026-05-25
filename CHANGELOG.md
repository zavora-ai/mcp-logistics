# Changelog

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
