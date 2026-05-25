//! mcp-sales — Enterprise Sales MCP Server
mod types;
mod server;

#[cfg(feature = "pandadoc")]
mod pandadoc;
#[cfg(feature = "apollo")]
mod apollo;
#[cfg(feature = "calendly")]
mod calendly;
#[cfg(feature = "stripe-billing")]
mod stripe_billing;

use rmcp::{ServiceExt, transport::stdio};
use server::SalesServer;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let manifest = adk_mcp_sdk::ServerManifest::from_file(std::path::Path::new("mcp-server.toml"))?;
    let errors = manifest.validate();
    if !errors.is_empty() {
        for e in &errors { tracing::error!("manifest: {e}"); }
        anyhow::bail!("invalid mcp-server.toml ({} error(s))", errors.len());
    }

    // Initialize backends — each is optional
    let proposals: Option<Arc<dyn server::ProposalBackend>> = init_proposals();
    let sequences: Option<Arc<dyn server::SequenceBackend>> = init_sequences();
    let meetings: Option<Arc<dyn server::MeetingBackend>> = init_meetings().await;
    let cpq: Option<Arc<dyn server::CpqBackend>> = init_cpq();

    if proposals.is_none() && sequences.is_none() && meetings.is_none() && cpq.is_none() {
        anyhow::bail!("No backend configured. Set at least one of: PANDADOC_API_KEY, APOLLO_API_KEY, CALENDLY_TOKEN, STRIPE_SECRET_KEY");
    }

    let mut active = Vec::new();
    if proposals.is_some() { active.push("proposals"); }
    if sequences.is_some() { active.push("sequences"); }
    if meetings.is_some() { active.push("meetings"); }
    if cpq.is_some() { active.push("cpq"); }
    tracing::info!("{} v{} starting on stdio (backends: {})", manifest.display_name, manifest.version, active.join(", "));

    let server = SalesServer { proposals, sequences, meetings, cpq };
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

fn init_proposals() -> Option<Arc<dyn server::ProposalBackend>> {
    #[cfg(feature = "pandadoc")]
    if let Ok(key) = std::env::var("PANDADOC_API_KEY") {
        tracing::info!("PandaDoc proposals backend enabled");
        return Some(Arc::new(pandadoc::PandaDocBackend::new(key)));
    }
    None
}

fn init_sequences() -> Option<Arc<dyn server::SequenceBackend>> {
    #[cfg(feature = "apollo")]
    if let Ok(key) = std::env::var("APOLLO_API_KEY") {
        tracing::info!("Apollo.io sequences backend enabled");
        return Some(Arc::new(apollo::ApolloBackend::new(key)));
    }
    None
}

async fn init_meetings() -> Option<Arc<dyn server::MeetingBackend>> {
    #[cfg(feature = "calendly")]
    if let Ok(token) = std::env::var("CALENDLY_TOKEN") {
        match calendly::CalendlyBackend::new(token).await {
            Ok(backend) => { tracing::info!("Calendly meetings backend enabled"); return Some(Arc::new(backend)); }
            Err(e) => { tracing::warn!("Calendly init failed: {e}"); }
        }
    }
    None
}

fn init_cpq() -> Option<Arc<dyn server::CpqBackend>> {
    #[cfg(feature = "stripe-billing")]
    if let Ok(key) = std::env::var("STRIPE_SECRET_KEY") {
        tracing::info!("Stripe Billing CPQ backend enabled");
        return Some(Arc::new(stripe_billing::StripeBillingBackend::new(key)));
    }
    None
}
