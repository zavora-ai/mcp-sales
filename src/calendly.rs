//! Calendly API backend — meetings, availability, booking links.
use crate::types::*;
use crate::server::MeetingBackend;
use anyhow::Result;
use reqwest::Client;

const BASE: &str = "https://api.calendly.com";

#[derive(Clone)]
pub struct CalendlyBackend { http: Client, token: String, user_uri: String }

impl CalendlyBackend {
    pub async fn new(token: String) -> Result<Self> {
        let http = Client::new();
        let resp: serde_json::Value = http.get(format!("{BASE}/users/me")).bearer_auth(&token).send().await?.error_for_status()?.json().await?;
        let user_uri = resp["resource"]["uri"].as_str().unwrap_or("").to_string();
        Ok(Self { http, token, user_uri })
    }
    async fn get(&self, path: &str) -> Result<serde_json::Value> {
        Ok(self.http.get(format!("{BASE}/{path}")).bearer_auth(&self.token).send().await?.error_for_status()?.json().await?)
    }
    async fn post(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        Ok(self.http.post(format!("{BASE}/{path}")).bearer_auth(&self.token).json(body).send().await?.error_for_status()?.json().await?)
    }
}

#[async_trait::async_trait]
impl MeetingBackend for CalendlyBackend {
    async fn get_availability(&self, start: Option<&str>, end: Option<&str>) -> Result<Vec<TimeSlot>> {
        let now = chrono::Utc::now();
        let s = start.unwrap_or(&now.format("%Y-%m-%dT%H:%M:%SZ").to_string()).to_string();
        let e = end.unwrap_or(&(now + chrono::Duration::days(7)).format("%Y-%m-%dT%H:%M:%SZ").to_string()).to_string();
        let resp = self.get(&format!("user_availability_schedules?user={}", urlenc(&self.user_uri))).await?;
        let mut slots = Vec::new();
        if let Some(schedules) = resp["collection"].as_array() {
            for sched in schedules {
                if let Some(rules) = sched["rules"].as_array() {
                    for rule in rules {
                        if let (Some(from), Some(to)) = (rule["intervals"].as_array().and_then(|a| a.first()).and_then(|i| i["from"].as_str()), rule["intervals"].as_array().and_then(|a| a.first()).and_then(|i| i["to"].as_str())) {
                            slots.push(TimeSlot { start: format!("{} {from}", s.split('T').next().unwrap_or("")), end: format!("{} {to}", s.split('T').next().unwrap_or("")) });
                        }
                    }
                }
            }
        }
        if slots.is_empty() { slots.push(TimeSlot { start: s, end: e }); }
        Ok(slots)
    }

    async fn create_booking_link(&self, name: &str, duration: u32) -> Result<BookingLink> {
        let resp = self.post("scheduling_links", &serde_json::json!({"max_event_count": 1, "owner": self.user_uri, "owner_type": "User"})).await?;
        let url = resp["resource"]["booking_url"].as_str().unwrap_or("").to_string();
        Ok(BookingLink { id: resp["resource"]["uri"].as_str().unwrap_or("").into(), url, name: name.into(), duration_minutes: duration })
    }

    async fn list_scheduled_meetings(&self, limit: u32) -> Result<Vec<Meeting>> {
        let resp = self.get(&format!("scheduled_events?user={}&count={limit}&status=active", urlenc(&self.user_uri))).await?;
        Ok(resp["collection"].as_array().map(|a| a.iter().map(|e| Meeting { id: e["uri"].as_str().unwrap_or("").into(), name: e["name"].as_str().unwrap_or("").into(), start_time: e["start_time"].as_str().unwrap_or("").into(), end_time: e["end_time"].as_str().unwrap_or("").into(), attendees: e["event_memberships"].as_array().map(|m| m.iter().filter_map(|a| a["user_email"].as_str().map(Into::into)).collect()).unwrap_or_default(), location: e["location"]["location"].as_str().map(Into::into), status: e["status"].as_str().unwrap_or("active").into() }).collect()).unwrap_or_default())
    }
}

fn urlenc(s: &str) -> String {
    s.bytes().map(|b| match b { b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(), _ => format!("%{:02X}", b) }).collect()
}
