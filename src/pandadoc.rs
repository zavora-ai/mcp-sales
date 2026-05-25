//! PandaDoc API backend — proposals, templates, signatures, tracking.
use crate::types::*;
use crate::server::ProposalBackend;
use anyhow::Result;
use reqwest::Client;

const BASE: &str = "https://api.pandadoc.com/public/v1";

#[derive(Clone)]
pub struct PandaDocBackend { http: Client, key: String }

impl PandaDocBackend {
    pub fn new(key: String) -> Self { Self { http: Client::new(), key } }
    async fn get(&self, path: &str) -> Result<serde_json::Value> {
        Ok(self.http.get(format!("{BASE}/{path}")).header("Authorization", format!("API-Key {}", self.key)).send().await?.error_for_status()?.json().await?)
    }
    async fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        Ok(self.http.post(format!("{BASE}/{path}")).header("Authorization", format!("API-Key {}", self.key)).json(body).send().await?.error_for_status()?.json().await?)
    }
    async fn delete_req(&self, path: &str) -> Result<()> {
        self.http.delete(format!("{BASE}/{path}")).header("Authorization", format!("API-Key {}", self.key)).send().await?.error_for_status()?; Ok(())
    }
    fn parse_proposal(d: &serde_json::Value) -> Proposal {
        Proposal { id: d["id"].as_str().unwrap_or("").into(), name: d["name"].as_str().unwrap_or("").into(), status: Self::parse_status(d["status"].as_str().unwrap_or("")), recipients: d["recipients"].as_array().map(|a| a.iter().map(|r| Recipient { email: r["email"].as_str().unwrap_or("").into(), first_name: r["first_name"].as_str().map(Into::into), last_name: r["last_name"].as_str().map(Into::into), role: r["role"].as_str().map(Into::into), has_signed: r["has_completed"].as_bool().unwrap_or(false) }).collect()).unwrap_or_default(), total_value: d["grand_total"]["amount"].as_str().and_then(|s| s.parse().ok()), currency: d["grand_total"]["currency"].as_str().map(Into::into), created_at: d["date_created"].as_str().map(Into::into), sent_at: d["date_sent"].as_str().map(Into::into), completed_at: d["date_completed"].as_str().map(Into::into), expiration_date: d["expiration_date"].as_str().map(Into::into), template_id: d["template_id"].as_str().map(Into::into), url: d["links"].as_array().and_then(|a| a.first()).and_then(|l| l["href"].as_str()).map(Into::into) }
    }
    fn parse_status(s: &str) -> ProposalStatus {
        match s { "document.draft" | "draft" => ProposalStatus::Draft, "document.sent" | "sent" => ProposalStatus::Sent, "document.viewed" | "viewed" => ProposalStatus::Viewed, "document.waiting_approval" => ProposalStatus::WaitingApproval, "document.approved" => ProposalStatus::Approved, "document.waiting_pay" | "document.waiting_signature" => ProposalStatus::WaitingSignature, "document.completed" | "completed" => ProposalStatus::Completed, "document.declined" | "declined" => ProposalStatus::Declined, "document.expired" | "expired" => ProposalStatus::Expired, "document.voided" | "voided" => ProposalStatus::Voided, _ => ProposalStatus::Draft }
    }
}

#[async_trait::async_trait]
impl ProposalBackend for PandaDocBackend {
    async fn create_proposal(&self, name: &str, recipients: &[Recipient], _template_id: Option<&str>, content: Option<&str>) -> Result<Proposal> {
        let recips: Vec<_> = recipients.iter().map(|r| serde_json::json!({"email": r.email, "first_name": r.first_name, "last_name": r.last_name, "role": r.role.as_deref().unwrap_or("signer")})).collect();
        let mut body = serde_json::json!({"name": name, "recipients": recips});
        if let Some(c) = content { body["content"] = serde_json::json!([{"type": "text", "data": {"text": c}}]); }
        let resp = self.post("documents", &body).await?;
        Ok(Self::parse_proposal(&resp))
    }

    async fn list_proposals(&self, status: Option<&str>, limit: u32) -> Result<Vec<Proposal>> {
        let mut path = format!("documents?count={limit}");
        if let Some(s) = status { path.push_str(&format!("&status={s}")); }
        let resp = self.get(&path).await?;
        Ok(resp["results"].as_array().map(|a| a.iter().map(|d| Self::parse_proposal(d)).collect()).unwrap_or_default())
    }

    async fn get_proposal(&self, id: &str) -> Result<Proposal> {
        let resp = self.get(&format!("documents/{id}/details")).await?;
        Ok(Self::parse_proposal(&resp))
    }

    async fn send_proposal(&self, id: &str, message: Option<&str>) -> Result<Proposal> {
        let body = serde_json::json!({"message": message.unwrap_or("Please review and sign"), "silent": false});
        self.post(&format!("documents/{id}/send"), &body).await?;
        self.get_proposal(id).await
    }

    async fn delete_proposal(&self, id: &str) -> Result<()> { self.delete_req(&format!("documents/{id}")).await }

    async fn list_templates(&self) -> Result<Vec<Template>> {
        let resp = self.get("templates").await?;
        Ok(resp["results"].as_array().map(|a| a.iter().map(|t| Template { id: t["id"].as_str().unwrap_or("").into(), name: t["name"].as_str().unwrap_or("").into(), description: t["description"].as_str().map(Into::into), created_at: t["date_created"].as_str().map(Into::into) }).collect()).unwrap_or_default())
    }

    async fn create_from_template(&self, template_id: &str, name: &str, recipients: &[Recipient], variables: Option<&serde_json::Value>) -> Result<Proposal> {
        let recips: Vec<_> = recipients.iter().map(|r| serde_json::json!({"email": r.email, "first_name": r.first_name, "last_name": r.last_name, "role": r.role.as_deref().unwrap_or("signer")})).collect();
        let mut body = serde_json::json!({"name": name, "template_uuid": template_id, "recipients": recips});
        if let Some(v) = variables { body["tokens"] = v.clone(); }
        let resp = self.post("documents", &body).await?;
        Ok(Self::parse_proposal(&resp))
    }

    async fn request_signature(&self, id: &str) -> Result<SignatureInfo> {
        self.post(&format!("documents/{id}/send"), &serde_json::json!({"message": "Please sign", "silent": false})).await?;
        self.get_signature_status(id).await
    }

    async fn get_signature_status(&self, id: &str) -> Result<SignatureInfo> {
        let doc = self.get(&format!("documents/{id}/details")).await?;
        let signers = doc["recipients"].as_array().map(|a| a.iter().map(|r| SignerStatus { email: r["email"].as_str().unwrap_or("").into(), name: r["first_name"].as_str().map(|f| format!("{} {}", f, r["last_name"].as_str().unwrap_or(""))), signed: r["has_completed"].as_bool().unwrap_or(false), signed_at: r["completed_date"].as_str().map(Into::into) }).collect()).unwrap_or_default();
        Ok(SignatureInfo { document_id: id.into(), document_name: doc["name"].as_str().unwrap_or("").into(), status: doc["status"].as_str().unwrap_or("").into(), signers })
    }

    async fn list_signatures(&self, limit: u32) -> Result<Vec<SignatureInfo>> {
        let resp = self.get(&format!("documents?status=document.waiting_signature&count={limit}")).await?;
        let mut sigs = Vec::new();
        for d in resp["results"].as_array().unwrap_or(&vec![]) {
            sigs.push(SignatureInfo { document_id: d["id"].as_str().unwrap_or("").into(), document_name: d["name"].as_str().unwrap_or("").into(), status: d["status"].as_str().unwrap_or("").into(), signers: vec![] });
        }
        Ok(sigs)
    }

    async fn get_proposal_views(&self, id: &str) -> Result<Vec<ViewEvent>> {
        let resp = self.get(&format!("documents/{id}/details")).await?;
        Ok(resp["recipients"].as_array().map(|a| a.iter().filter(|r| r["has_opened"].as_bool() == Some(true)).map(|r| ViewEvent { recipient_email: r["email"].as_str().unwrap_or("").into(), viewed_at: r["opened_date"].as_str().unwrap_or("").into(), duration_seconds: None, pages_viewed: None }).collect()).unwrap_or_default())
    }

    async fn get_proposal_analytics(&self) -> Result<ProposalAnalytics> {
        let sent = self.list_proposals(Some("document.sent"), 100).await?.len() as u32;
        let viewed = self.list_proposals(Some("document.viewed"), 100).await?.len() as u32;
        let completed = self.list_proposals(Some("document.completed"), 100).await?.len() as u32;
        let total = sent + viewed + completed;
        Ok(ProposalAnalytics { total_sent: total, total_viewed: viewed + completed, total_signed: completed, conversion_rate: if total > 0 { completed as f64 / total as f64 * 100.0 } else { 0.0 }, avg_time_to_view_hours: 0.0, avg_time_to_sign_days: 0.0 })
    }

    async fn get_win_loss_analysis(&self, _period: Option<&str>) -> Result<WinLossAnalysis> {
        let completed = self.list_proposals(Some("document.completed"), 100).await?.len() as u32;
        let declined = self.list_proposals(Some("document.declined"), 100).await?.len() as u32;
        let total = completed + declined;
        Ok(WinLossAnalysis { period: "current".into(), total_deals: total, won: completed, lost: declined, win_rate: if total > 0 { completed as f64 / total as f64 * 100.0 } else { 0.0 }, avg_deal_size: 0.0, avg_sales_cycle_days: 0, top_loss_reasons: vec![] })
    }
}
