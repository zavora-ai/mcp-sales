//! MCP tool router for sales operations.
use adk_mcp_sdk::{HealthCheck, HealthStatus};
use crate::types::*;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;
use std::sync::Arc;

// ─── Input types ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateProposalInput { pub name: String, pub recipients: Vec<Recipient>, #[serde(default)] pub template_id: Option<String>, #[serde(default)] pub content: Option<String> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListInput { #[serde(default)] pub status: Option<String>, #[serde(default = "d20")] pub limit: u32 }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct IdInput { pub id: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendProposalInput { pub id: String, #[serde(default)] pub message: Option<String> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateFromTemplateInput { pub template_id: String, pub name: String, pub recipients: Vec<Recipient>, #[serde(default)] pub variables: Option<serde_json::Value> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CalculateQuoteInput { pub line_items: Vec<QuoteLineItem>, #[serde(default = "d_usd")] pub currency: String }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ApplyDiscountInput { pub quote_id: String, #[serde(default)] pub percent: Option<f64>, #[serde(default)] pub fixed_amount: Option<f64> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateQuoteInput { pub line_items: Vec<QuoteLineItem>, pub customer_email: String, #[serde(default = "d_usd")] pub currency: String, #[serde(default)] pub valid_days: Option<u32> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateSequenceInput { pub name: String, pub steps: Vec<String> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EnrollContactInput { pub sequence_id: String, pub email: String, #[serde(default)] pub first_name: Option<String>, #[serde(default)] pub last_name: Option<String> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SendTrackedEmailInput { pub to: String, pub subject: String, pub body: String, #[serde(default)] pub from_name: Option<String> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AvailabilityInput { #[serde(default)] pub date_range_start: Option<String>, #[serde(default)] pub date_range_end: Option<String> }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateBookingInput { pub name: String, #[serde(default = "d30")] pub duration_minutes: u32 }
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PeriodInput { #[serde(default)] pub period: Option<String> }

fn d20() -> u32 { 20 }
fn d30() -> u32 { 30 }
fn d_usd() -> String { "USD".into() }

// ─── Backend traits ──────────────────────────────────────────────────────────

#[async_trait::async_trait]
pub trait ProposalBackend: Send + Sync {
    async fn create_proposal(&self, name: &str, recipients: &[Recipient], template_id: Option<&str>, content: Option<&str>) -> anyhow::Result<Proposal>;
    async fn list_proposals(&self, status: Option<&str>, limit: u32) -> anyhow::Result<Vec<Proposal>>;
    async fn get_proposal(&self, id: &str) -> anyhow::Result<Proposal>;
    async fn send_proposal(&self, id: &str, message: Option<&str>) -> anyhow::Result<Proposal>;
    async fn delete_proposal(&self, id: &str) -> anyhow::Result<()>;
    async fn list_templates(&self) -> anyhow::Result<Vec<Template>>;
    async fn create_from_template(&self, template_id: &str, name: &str, recipients: &[Recipient], variables: Option<&serde_json::Value>) -> anyhow::Result<Proposal>;
    async fn request_signature(&self, id: &str) -> anyhow::Result<SignatureInfo>;
    async fn get_signature_status(&self, id: &str) -> anyhow::Result<SignatureInfo>;
    async fn list_signatures(&self, limit: u32) -> anyhow::Result<Vec<SignatureInfo>>;
    async fn get_proposal_views(&self, id: &str) -> anyhow::Result<Vec<ViewEvent>>;
    async fn get_proposal_analytics(&self) -> anyhow::Result<ProposalAnalytics>;
    async fn get_win_loss_analysis(&self, period: Option<&str>) -> anyhow::Result<WinLossAnalysis>;
}

#[async_trait::async_trait]
pub trait SequenceBackend: Send + Sync {
    async fn list_sequences(&self, limit: u32) -> anyhow::Result<Vec<Sequence>>;
    async fn create_sequence(&self, name: &str, steps: &[String]) -> anyhow::Result<Sequence>;
    async fn enroll_contact(&self, sequence_id: &str, email: &str, first_name: Option<&str>, last_name: Option<&str>) -> anyhow::Result<()>;
    async fn pause_sequence(&self, sequence_id: &str) -> anyhow::Result<()>;
    async fn get_sequence_stats(&self, id: &str) -> anyhow::Result<Sequence>;
    async fn send_tracked_email(&self, to: &str, subject: &str, body: &str, from_name: Option<&str>) -> anyhow::Result<EmailEngagement>;
    async fn get_email_engagement(&self, limit: u32) -> anyhow::Result<Vec<EmailEngagement>>;
}

#[async_trait::async_trait]
pub trait MeetingBackend: Send + Sync {
    async fn get_availability(&self, start: Option<&str>, end: Option<&str>) -> anyhow::Result<Vec<TimeSlot>>;
    async fn create_booking_link(&self, name: &str, duration: u32) -> anyhow::Result<BookingLink>;
    async fn list_scheduled_meetings(&self, limit: u32) -> anyhow::Result<Vec<Meeting>>;
}

#[async_trait::async_trait]
pub trait CpqBackend: Send + Sync {
    async fn list_products(&self, limit: u32) -> anyhow::Result<Vec<Product>>;
    async fn calculate_quote(&self, items: &[QuoteLineItem], currency: &str) -> anyhow::Result<Quote>;
    async fn apply_discount(&self, quote_id: &str, percent: Option<f64>, fixed: Option<f64>) -> anyhow::Result<Quote>;
    async fn create_quote(&self, items: &[QuoteLineItem], customer_email: &str, currency: &str, valid_days: Option<u32>) -> anyhow::Result<Quote>;
    async fn get_pipeline_forecast(&self, period: Option<&str>) -> anyhow::Result<Forecast>;
    async fn get_quota_progress(&self, period: Option<&str>) -> anyhow::Result<QuotaProgress>;
}

// ─── Server ──────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct SalesServer {
    pub proposals: Option<Arc<dyn ProposalBackend>>,
    pub sequences: Option<Arc<dyn SequenceBackend>>,
    pub meetings: Option<Arc<dyn MeetingBackend>>,
    pub cpq: Option<Arc<dyn CpqBackend>>,
}

#[tool_router]
impl SalesServer {
    // ─── Proposals ───────────────────────────────────────────────────────────
    #[tool(description = "Create a new proposal/document in draft state")]
    async fn create_proposal(&self, Parameters(i): Parameters<CreateProposalInput>) -> String {
        match &self.proposals { Some(p) => match p.create_proposal(&i.name, &i.recipients, i.template_id.as_deref(), i.content.as_deref()).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Proposal backend not configured".into() }
    }
    #[tool(description = "List proposals with optional status filter")]
    async fn list_proposals(&self, Parameters(i): Parameters<ListInput>) -> String {
        match &self.proposals { Some(p) => match p.list_proposals(i.status.as_deref(), i.limit).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Proposal backend not configured".into() }
    }
    #[tool(description = "Get proposal details including status, recipients, and metadata")]
    async fn get_proposal(&self, Parameters(i): Parameters<IdInput>) -> String {
        match &self.proposals { Some(p) => match p.get_proposal(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Proposal backend not configured".into() }
    }
    #[tool(description = "Send a draft proposal to recipients for review/signature")]
    async fn send_proposal(&self, Parameters(i): Parameters<SendProposalInput>) -> String {
        match &self.proposals { Some(p) => match p.send_proposal(&i.id, i.message.as_deref()).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Proposal backend not configured".into() }
    }
    #[tool(description = "Delete a proposal")]
    async fn delete_proposal(&self, Parameters(i): Parameters<IdInput>) -> String {
        match &self.proposals { Some(p) => match p.delete_proposal(&i.id).await { Ok(()) => "Proposal deleted".into(), Err(e) => format!("Error: {e}") }, None => "Proposal backend not configured".into() }
    }
    // ─── Templates ───────────────────────────────────────────────────────────
    #[tool(description = "List available proposal/document templates")]
    async fn list_templates(&self) -> String {
        match &self.proposals { Some(p) => match p.list_templates().await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Proposal backend not configured".into() }
    }
    #[tool(description = "Create a new proposal from a template with variable substitution")]
    async fn create_from_template(&self, Parameters(i): Parameters<CreateFromTemplateInput>) -> String {
        match &self.proposals { Some(p) => match p.create_from_template(&i.template_id, &i.name, &i.recipients, i.variables.as_ref()).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Proposal backend not configured".into() }
    }
    // ─── Signatures ──────────────────────────────────────────────────────────
    #[tool(description = "Request e-signature on a sent proposal")]
    async fn request_signature(&self, Parameters(i): Parameters<IdInput>) -> String {
        match &self.proposals { Some(p) => match p.request_signature(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Proposal backend not configured".into() }
    }
    #[tool(description = "Get signature status for a proposal")]
    async fn get_signature_status(&self, Parameters(i): Parameters<IdInput>) -> String {
        match &self.proposals { Some(p) => match p.get_signature_status(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Proposal backend not configured".into() }
    }
    #[tool(description = "List all documents pending or completed signature")]
    async fn list_signatures(&self, Parameters(i): Parameters<ListInput>) -> String {
        match &self.proposals { Some(p) => match p.list_signatures(i.limit).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Proposal backend not configured".into() }
    }
    // ─── Tracking ────────────────────────────────────────────────────────────
    #[tool(description = "Get view/open events for a proposal (who viewed, when, duration)")]
    async fn get_proposal_views(&self, Parameters(i): Parameters<IdInput>) -> String {
        match &self.proposals { Some(p) => match p.get_proposal_views(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Proposal backend not configured".into() }
    }
    #[tool(description = "Get aggregate analytics for proposals (conversion rates, avg time to sign)")]
    async fn get_proposal_analytics(&self) -> String {
        match &self.proposals { Some(p) => match p.get_proposal_analytics().await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Proposal backend not configured".into() }
    }
    // ─── CPQ ─────────────────────────────────────────────────────────────────
    #[tool(description = "List products/services from the pricing catalog")]
    async fn list_products(&self, Parameters(i): Parameters<ListInput>) -> String {
        match &self.cpq { Some(c) => match c.list_products(i.limit).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "CPQ backend not configured".into() }
    }
    #[tool(description = "Calculate a quote with line items, quantities, and pricing")]
    async fn calculate_quote(&self, Parameters(i): Parameters<CalculateQuoteInput>) -> String {
        match &self.cpq { Some(c) => match c.calculate_quote(&i.line_items, &i.currency).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "CPQ backend not configured".into() }
    }
    #[tool(description = "Apply a discount (percentage or fixed) to a quote")]
    async fn apply_discount(&self, Parameters(i): Parameters<ApplyDiscountInput>) -> String {
        match &self.cpq { Some(c) => match c.apply_discount(&i.quote_id, i.percent, i.fixed_amount).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "CPQ backend not configured".into() }
    }
    #[tool(description = "Create a formal quote in Stripe (can be sent to customer)")]
    async fn create_quote(&self, Parameters(i): Parameters<CreateQuoteInput>) -> String {
        match &self.cpq { Some(c) => match c.create_quote(&i.line_items, &i.customer_email, &i.currency, i.valid_days).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "CPQ backend not configured".into() }
    }
    // ─── Sequences ───────────────────────────────────────────────────────────
    #[tool(description = "List sales engagement sequences")]
    async fn list_sequences(&self, Parameters(i): Parameters<ListInput>) -> String {
        match &self.sequences { Some(s) => match s.list_sequences(i.limit).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Sequence backend not configured".into() }
    }
    #[tool(description = "Create a new outbound sequence with steps")]
    async fn create_sequence(&self, Parameters(i): Parameters<CreateSequenceInput>) -> String {
        match &self.sequences { Some(s) => match s.create_sequence(&i.name, &i.steps).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Sequence backend not configured".into() }
    }
    #[tool(description = "Enroll a contact into a sequence")]
    async fn enroll_contact(&self, Parameters(i): Parameters<EnrollContactInput>) -> String {
        match &self.sequences { Some(s) => match s.enroll_contact(&i.sequence_id, &i.email, i.first_name.as_deref(), i.last_name.as_deref()).await { Ok(()) => "Contact enrolled".into(), Err(e) => format!("Error: {e}") }, None => "Sequence backend not configured".into() }
    }
    #[tool(description = "Pause a running sequence for a contact")]
    async fn pause_sequence(&self, Parameters(i): Parameters<IdInput>) -> String {
        match &self.sequences { Some(s) => match s.pause_sequence(&i.id).await { Ok(()) => "Sequence paused".into(), Err(e) => format!("Error: {e}") }, None => "Sequence backend not configured".into() }
    }
    #[tool(description = "Get performance stats for a sequence (open rate, reply rate, meetings booked)")]
    async fn get_sequence_stats(&self, Parameters(i): Parameters<IdInput>) -> String {
        match &self.sequences { Some(s) => match s.get_sequence_stats(&i.id).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Sequence backend not configured".into() }
    }
    // ─── Email Tracking ──────────────────────────────────────────────────────
    #[tool(description = "Send a tracked email with open/click tracking")]
    async fn send_tracked_email(&self, Parameters(i): Parameters<SendTrackedEmailInput>) -> String {
        match &self.sequences { Some(s) => match s.send_tracked_email(&i.to, &i.subject, &i.body, i.from_name.as_deref()).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Sequence backend not configured".into() }
    }
    #[tool(description = "Get engagement data for sent emails (opens, clicks, replies)")]
    async fn get_email_engagement(&self, Parameters(i): Parameters<ListInput>) -> String {
        match &self.sequences { Some(s) => match s.get_email_engagement(i.limit).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Sequence backend not configured".into() }
    }
    // ─── Meetings ────────────────────────────────────────────────────────────
    #[tool(description = "Get available time slots for scheduling a meeting")]
    async fn get_availability(&self, Parameters(i): Parameters<AvailabilityInput>) -> String {
        match &self.meetings { Some(m) => match m.get_availability(i.date_range_start.as_deref(), i.date_range_end.as_deref()).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Meeting backend not configured".into() }
    }
    #[tool(description = "Create a one-time or reusable booking link")]
    async fn create_booking_link(&self, Parameters(i): Parameters<CreateBookingInput>) -> String {
        match &self.meetings { Some(m) => match m.create_booking_link(&i.name, i.duration_minutes).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Meeting backend not configured".into() }
    }
    #[tool(description = "List upcoming and past scheduled meetings")]
    async fn list_scheduled_meetings(&self, Parameters(i): Parameters<ListInput>) -> String {
        match &self.meetings { Some(m) => match m.list_scheduled_meetings(i.limit).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Meeting backend not configured".into() }
    }
    // ─── Forecasting ─────────────────────────────────────────────────────────
    #[tool(description = "Get weighted pipeline forecast based on deal stages and probabilities")]
    async fn get_pipeline_forecast(&self, Parameters(i): Parameters<PeriodInput>) -> String {
        match &self.cpq { Some(c) => match c.get_pipeline_forecast(i.period.as_deref()).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "CPQ backend not configured".into() }
    }
    #[tool(description = "Get quota attainment progress for current period")]
    async fn get_quota_progress(&self, Parameters(i): Parameters<PeriodInput>) -> String {
        match &self.cpq { Some(c) => match c.get_quota_progress(i.period.as_deref()).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "CPQ backend not configured".into() }
    }
    #[tool(description = "Get win/loss analysis with reasons and trends")]
    async fn get_win_loss_analysis(&self, Parameters(i): Parameters<PeriodInput>) -> String {
        match &self.proposals { Some(p) => match p.get_win_loss_analysis(i.period.as_deref()).await { Ok(v) => serde_json::to_string_pretty(&v).unwrap(), Err(e) => format!("Error: {e}") }, None => "Proposal backend not configured".into() }
    }
}

#[async_trait::async_trait]
impl HealthCheck for SalesServer {
    async fn check_health(&self) -> HealthStatus {
        let mut healthy = false;
        let mut msg = Vec::new();
        if self.proposals.is_some() { healthy = true; msg.push("proposals"); }
        if self.sequences.is_some() { healthy = true; msg.push("sequences"); }
        if self.meetings.is_some() { healthy = true; msg.push("meetings"); }
        if self.cpq.is_some() { healthy = true; msg.push("cpq"); }
        HealthStatus { healthy, message: Some(format!("backends: {}", msg.join(", "))), latency_ms: Some(1) }
    }
}

adk_mcp_sdk::mcp_2026_server! {
    server: SalesServer,
    task_tools: ["get_pipeline_forecast"],
    approval_tools: ["send_proposal", "delete_proposal", "request_signature", "create_quote", "enroll_contact", "send_tracked_email"],
    cache_ttl_ms: 60_000,
}
