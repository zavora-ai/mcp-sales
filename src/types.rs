//! Unified sales types — proposals, quotes, sequences, meetings, and forecasting.
use serde::{Deserialize, Serialize};

// ─── Proposal Lifecycle ──────────────────────────────────────────────────────

/// Proposal document lifecycle state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Draft,
    Sent,
    Viewed,
    WaitingApproval,
    Approved,
    WaitingSignature,
    PartiallySigned,
    Signed,
    Completed,
    Declined,
    Expired,
    Voided,
}

/// A sales proposal/document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: String,
    pub name: String,
    pub status: ProposalStatus,
    pub recipients: Vec<Recipient>,
    pub total_value: Option<f64>,
    pub currency: Option<String>,
    pub created_at: Option<String>,
    pub sent_at: Option<String>,
    pub completed_at: Option<String>,
    pub expiration_date: Option<String>,
    pub template_id: Option<String>,
    pub url: Option<String>,
}

/// A proposal recipient.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Recipient {
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: Option<String>,
    pub has_signed: bool,
}

/// A proposal template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: Option<String>,
}

/// Proposal view/engagement event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewEvent {
    pub recipient_email: String,
    pub viewed_at: String,
    pub duration_seconds: Option<u32>,
    pub pages_viewed: Option<u32>,
}

/// Signature status for a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureInfo {
    pub document_id: String,
    pub document_name: String,
    pub status: String,
    pub signers: Vec<SignerStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignerStatus {
    pub email: String,
    pub name: Option<String>,
    pub signed: bool,
    pub signed_at: Option<String>,
}

// ─── CPQ (Configure-Price-Quote) ─────────────────────────────────────────────

/// A product in the pricing catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub unit_price: f64,
    pub currency: String,
    pub recurring: Option<String>,
}

/// A line item in a quote.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct QuoteLineItem {
    pub product_id: Option<String>,
    pub description: String,
    pub quantity: u32,
    pub unit_price: f64,
    pub discount_percent: Option<f64>,
    pub total: f64,
}

/// A calculated quote.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub id: Option<String>,
    pub line_items: Vec<QuoteLineItem>,
    pub subtotal: f64,
    pub discount_total: f64,
    pub total: f64,
    pub currency: String,
    pub valid_until: Option<String>,
    pub url: Option<String>,
}

// ─── Sequences & Email Tracking ──────────────────────────────────────────────

/// A sales engagement sequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sequence {
    pub id: String,
    pub name: String,
    pub steps: u32,
    pub active_contacts: u32,
    pub status: String,
    pub open_rate: Option<f64>,
    pub reply_rate: Option<f64>,
}

/// Email engagement data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailEngagement {
    pub email_id: String,
    pub subject: String,
    pub recipient: String,
    pub sent_at: String,
    pub opens: u32,
    pub clicks: u32,
    pub replied: bool,
    pub bounced: bool,
}

// ─── Meetings ────────────────────────────────────────────────────────────────

/// An available time slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSlot {
    pub start: String,
    pub end: String,
}

/// A booking link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookingLink {
    pub id: String,
    pub url: String,
    pub name: String,
    pub duration_minutes: u32,
}

/// A scheduled meeting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meeting {
    pub id: String,
    pub name: String,
    pub start_time: String,
    pub end_time: String,
    pub attendees: Vec<String>,
    pub location: Option<String>,
    pub status: String,
}

// ─── Forecasting ─────────────────────────────────────────────────────────────

/// Pipeline forecast data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast {
    pub period: String,
    pub weighted_pipeline: f64,
    pub best_case: f64,
    pub commit: f64,
    pub closed_won: f64,
    pub currency: String,
}

/// Quota progress.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaProgress {
    pub period: String,
    pub quota: f64,
    pub attainment: f64,
    pub attainment_percent: f64,
    pub deals_closed: u32,
    pub currency: String,
}

/// Win/loss analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinLossAnalysis {
    pub period: String,
    pub total_deals: u32,
    pub won: u32,
    pub lost: u32,
    pub win_rate: f64,
    pub avg_deal_size: f64,
    pub avg_sales_cycle_days: u32,
    pub top_loss_reasons: Vec<String>,
}

// ─── Proposal Analytics ──────────────────────────────────────────────────────

/// Aggregate proposal analytics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalAnalytics {
    pub total_sent: u32,
    pub total_viewed: u32,
    pub total_signed: u32,
    pub conversion_rate: f64,
    pub avg_time_to_view_hours: f64,
    pub avg_time_to_sign_days: f64,
}
