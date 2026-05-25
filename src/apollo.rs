//! Apollo.io API backend — sequences, email tracking, prospecting.
use crate::types::*;
use crate::server::SequenceBackend;
use anyhow::Result;
use reqwest::Client;

const BASE: &str = "https://api.apollo.io/v1";

#[derive(Clone)]
pub struct ApolloBackend { http: Client, key: String }

impl ApolloBackend {
    pub fn new(key: String) -> Self { Self { http: Client::new(), key } }
    async fn get(&self, path: &str) -> Result<serde_json::Value> {
        Ok(self.http.get(format!("{BASE}/{path}")).header("X-Api-Key", &self.key).send().await?.error_for_status()?.json().await?)
    }
    async fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        Ok(self.http.post(format!("{BASE}/{path}")).header("X-Api-Key", &self.key).json(body).send().await?.error_for_status()?.json().await?)
    }
}

#[async_trait::async_trait]
impl SequenceBackend for ApolloBackend {
    async fn list_sequences(&self, limit: u32) -> Result<Vec<Sequence>> {
        let resp = self.post("emailer_campaigns/search", &serde_json::json!({"per_page": limit})).await?;
        Ok(resp["emailer_campaigns"].as_array().map(|a| a.iter().map(|s| Sequence { id: s["id"].as_str().unwrap_or("").into(), name: s["name"].as_str().unwrap_or("").into(), steps: s["num_steps"].as_u64().unwrap_or(0) as u32, active_contacts: s["active_count"].as_u64().unwrap_or(0) as u32, status: if s["active"].as_bool() == Some(true) { "active" } else { "paused" }.into(), open_rate: s["unique_open_rate"].as_f64(), reply_rate: s["unique_reply_rate"].as_f64() }).collect()).unwrap_or_default())
    }

    async fn create_sequence(&self, name: &str, steps: &[String]) -> Result<Sequence> {
        let step_configs: Vec<_> = steps.iter().enumerate().map(|(i, body)| serde_json::json!({"type": "auto_email", "priority": i + 1, "exact_datetime": null, "body_template": body})).collect();
        let resp = self.post("emailer_campaigns", &serde_json::json!({"name": name, "emailer_steps": step_configs})).await?;
        let s = &resp["emailer_campaign"];
        Ok(Sequence { id: s["id"].as_str().unwrap_or("").into(), name: name.into(), steps: steps.len() as u32, active_contacts: 0, status: "active".into(), open_rate: None, reply_rate: None })
    }

    async fn enroll_contact(&self, sequence_id: &str, email: &str, first_name: Option<&str>, last_name: Option<&str>) -> Result<()> {
        let mut contact = serde_json::json!({"email": email});
        if let Some(f) = first_name { contact["first_name"] = f.into(); }
        if let Some(l) = last_name { contact["last_name"] = l.into(); }
        self.post("emailer_campaigns/add_contact_ids", &serde_json::json!({"emailer_campaign_id": sequence_id, "contact_ids": [], "emails": [email]})).await?;
        Ok(())
    }

    async fn pause_sequence(&self, sequence_id: &str) -> Result<()> {
        self.post(&format!("emailer_campaigns/{sequence_id}/pause"), &serde_json::json!({})).await?;
        Ok(())
    }

    async fn get_sequence_stats(&self, id: &str) -> Result<Sequence> {
        let resp = self.get(&format!("emailer_campaigns/{id}")).await?;
        let s = &resp["emailer_campaign"];
        Ok(Sequence { id: s["id"].as_str().unwrap_or("").into(), name: s["name"].as_str().unwrap_or("").into(), steps: s["num_steps"].as_u64().unwrap_or(0) as u32, active_contacts: s["active_count"].as_u64().unwrap_or(0) as u32, status: if s["active"].as_bool() == Some(true) { "active" } else { "paused" }.into(), open_rate: s["unique_open_rate"].as_f64(), reply_rate: s["unique_reply_rate"].as_f64() })
    }

    async fn send_tracked_email(&self, to: &str, subject: &str, body: &str, _from_name: Option<&str>) -> Result<EmailEngagement> {
        let resp = self.post("emails", &serde_json::json!({"to": to, "subject": subject, "body": body})).await?;
        Ok(EmailEngagement { email_id: resp["id"].as_str().unwrap_or("").into(), subject: subject.into(), recipient: to.into(), sent_at: chrono::Utc::now().to_rfc3339(), opens: 0, clicks: 0, replied: false, bounced: false })
    }

    async fn get_email_engagement(&self, limit: u32) -> Result<Vec<EmailEngagement>> {
        let resp = self.post("emails/search", &serde_json::json!({"per_page": limit})).await?;
        Ok(resp["emails"].as_array().map(|a| a.iter().map(|e| EmailEngagement { email_id: e["id"].as_str().unwrap_or("").into(), subject: e["subject"].as_str().unwrap_or("").into(), recipient: e["to"].as_str().unwrap_or("").into(), sent_at: e["sent_at"].as_str().unwrap_or("").into(), opens: e["open_count"].as_u64().unwrap_or(0) as u32, clicks: e["click_count"].as_u64().unwrap_or(0) as u32, replied: e["replied"].as_bool().unwrap_or(false), bounced: e["bounced"].as_bool().unwrap_or(false) }).collect()).unwrap_or_default())
    }
}
