# Changelog

## [1.1.0] - 2026-08-13

### Changed
- Upgraded to rmcp 3.1.2 and raised the minimum supported Rust version to 1.94.1.
- Added MCP 2026-07-28 stateless request handling while retaining MCP 2025-11-25 initialization compatibility.

### Added
- Per-request identity and protocol metadata, on-demand discovery/cache hints, and the configured Tasks and sealed MRTR approval policies.

## [1.0.0] - 2026-05-25

### Added
- Initial release with 29 tools across 9 categories
- **4 backends:** PandaDoc, Apollo.io, Calendly, Stripe Billing
- **Proposals:** create, list, get, send, delete with full lifecycle tracking
- **Templates:** list, create from template with variable substitution
- **Signatures:** request, get status, list pending/completed
- **Tracking:** proposal views (who, when, duration), aggregate analytics
- **CPQ:** list products, calculate quote, apply discount, create formal quote
- **Sequences:** list, create, enroll contact, pause, get stats
- **Email Tracking:** send tracked email, get engagement (opens, clicks, replies)
- **Meetings:** get availability, create booking link, list scheduled
- **Forecasting:** pipeline forecast, quota progress, win/loss analysis
- Proposal lifecycle: draft → sent → viewed → waiting_signature → signed → completed
- Each backend is optional — server works with any combination
- Feature flags: default = pandadoc + stripe-billing
- Manifest validation on startup (adk-mcp-sdk 0.1.3)
- Health check reports active backends
