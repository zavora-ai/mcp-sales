# Sales MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-sales.svg)](https://crates.io/crates/mcp-sales)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)
[![Registry Ready](https://img.shields.io/badge/ADK_Registry-Ready-green.svg)](https://www.zavora.ai)

Enterprise sales MCP server with **29 tools** across **4 backends** — PandaDoc, Apollo.io, Calendly, and Stripe Billing. Proposals with lifecycle management, CPQ (configure-price-quote), sales sequences, meeting scheduling, and revenue forecasting.

## Architecture

<p align="center">
  <img src="https://raw.githubusercontent.com/zavora-ai/mcp-sales/main/docs/assets/architecture.svg" alt="MCP Sales Architecture" width="800"/>
</p>

## Key Principles

- **Full sales lifecycle** — draft → sent → viewed → signed → won with tracking at every stage
- **Modular backends** — each backend is optional; use any combination
- **Revenue-focused** — proposals, quotes, and forecasting drive pipeline visibility
- **Engagement tracking** — know when prospects open emails, view proposals, click links
- **No credential exposure** — API keys stay in env vars

## Sales Flow

```
Prospect → Sequence → Meeting → Quote → Proposal → Signature → Won ✓
   (Apollo)  (Apollo)  (Calendly) (Stripe) (PandaDoc) (PandaDoc)
```

## Tools (29)

| Category | Tools | Backend | Risk |
|----------|-------|---------|------|
| Proposals | `create_proposal`, `list_proposals`, `get_proposal`, `send_proposal`, `delete_proposal` | PandaDoc | internal/external write |
| Templates | `list_templates`, `create_from_template` | PandaDoc | read / internal write |
| Signatures | `request_signature`, `get_signature_status`, `list_signatures` | PandaDoc | external write / read |
| Tracking | `get_proposal_views`, `get_proposal_analytics` | PandaDoc | read_only |
| CPQ | `list_products`, `calculate_quote`, `apply_discount`, `create_quote` | Stripe | read / external write |
| Sequences | `list_sequences`, `create_sequence`, `enroll_contact`, `pause_sequence`, `get_sequence_stats` | Apollo | internal/external write |
| Email | `send_tracked_email`, `get_email_engagement` | Apollo | external write / read |
| Meetings | `get_availability`, `create_booking_link`, `list_scheduled_meetings` | Calendly | read / internal write |
| Forecasting | `get_pipeline_forecast`, `get_quota_progress`, `get_win_loss_analysis` | Stripe/PandaDoc | read_only |

## Proposal Lifecycle

```
draft → sent → viewed → waiting_signature → partially_signed → signed → completed
                                                                    ↘ declined / expired / voided
```

## Backends

| Backend | Covers | Auth | Default |
|---------|--------|------|:---:|
| **PandaDoc** | Proposals, templates, signatures, tracking | API Key | ✅ |
| **Apollo.io** | Sequences, email tracking, engagement | API Key | ❌ |
| **Calendly** | Availability, booking links, meetings | OAuth2 | ❌ |
| **Stripe Billing** | Products, quotes, CPQ, forecasting | Secret Key | ✅ |

## Installation

```bash
cargo install mcp-sales --features all-backends
```

### Feature flags

```bash
# Default: PandaDoc + Stripe Billing
cargo install mcp-sales

# All backends
cargo install mcp-sales --features all-backends

# Specific
cargo install mcp-sales --no-default-features --features "pandadoc,calendly"
```

## Configuration

### PandaDoc
```bash
export PANDADOC_API_KEY="your-api-key"
```

### Apollo.io
```bash
export APOLLO_API_KEY="your-api-key"
```

### Calendly
```bash
export CALENDLY_TOKEN="your-personal-access-token"
```

### Stripe Billing
```bash
export STRIPE_SECRET_KEY="sk_live_xxx"
```

## Client Configuration

```json
{
  "mcpServers": {
    "sales": {
      "command": "mcp-sales",
      "args": [],
      "env": {
        "PANDADOC_API_KEY": "xxx",
        "APOLLO_API_KEY": "xxx",
        "CALENDLY_TOKEN": "xxx",
        "STRIPE_SECRET_KEY": "sk_xxx"
      }
    }
  }
}
```

## Usage Examples

### Create and send a proposal
```
"Create a proposal for Acme Corp's enterprise license"
→ create_proposal(name: "Acme Enterprise License", recipients: [{email: "cto@acme.com", first_name: "John", role: "signer"}])

"Send it with a personal message"
→ send_proposal(id: "doc-123", message: "Hi John, here's the proposal we discussed")

"Has John viewed it?"
→ get_proposal_views(id: "doc-123")

"Request his signature"
→ request_signature(id: "doc-123")
```

### Build and send a quote
```
"What products do we have?"
→ list_products()

"Calculate a quote for 10 seats of Pro plan at $99/mo"
→ calculate_quote(line_items: [{description: "Pro Plan", quantity: 10, unit_price: 99, total: 990}], currency: "USD")

"Create a formal quote and send to the customer"
→ create_quote(line_items: [...], customer_email: "cto@acme.com", valid_days: 30)
```

### Run an outbound sequence
```
"Show me our active sequences"
→ list_sequences()

"Enroll sarah@prospect.com in the Enterprise Outbound sequence"
→ enroll_contact(sequence_id: "seq-456", email: "sarah@prospect.com", first_name: "Sarah")

"How's that sequence performing?"
→ get_sequence_stats(id: "seq-456")
```

### Schedule a meeting
```
"What's my availability next week?"
→ get_availability(date_range_start: "2026-06-01", date_range_end: "2026-06-07")

"Create a booking link for a 30-min demo"
→ create_booking_link(name: "Product Demo", duration_minutes: 30)
```

### Revenue forecasting
```
"What's our pipeline forecast?"
→ get_pipeline_forecast()

"How are we tracking against quota?"
→ get_quota_progress()

"Show win/loss analysis"
→ get_win_loss_analysis()
```

## Registry Compliance

- **HealthCheck** — reports which backends are active
- **mcp-server.toml** — 29 tools with risk classes and credential bindings
- **Manifest validation** — startup fails fast on invalid manifest
- **Structured tracing** — `RUST_LOG` env-filter

## Contributors

| [<img src="https://github.com/jkmaina.png" width="80px;" alt=""/><br /><sub><b>James Karanja Maina</b></sub>](https://github.com/jkmaina) |
|:---:|

## License

Apache-2.0

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.
