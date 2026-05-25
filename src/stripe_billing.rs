//! Stripe Billing API backend — products, pricing, quotes, forecasting.
use crate::types::*;
use crate::server::CpqBackend;
use anyhow::Result;
use reqwest::Client;

const BASE: &str = "https://api.stripe.com/v1";

#[derive(Clone)]
pub struct StripeBillingBackend { http: Client, key: String }

impl StripeBillingBackend {
    pub fn new(key: String) -> Self { Self { http: Client::new(), key } }
    async fn get(&self, path: &str) -> Result<serde_json::Value> {
        Ok(self.http.get(format!("{BASE}/{path}")).basic_auth(&self.key, None::<&str>).send().await?.error_for_status()?.json().await?)
    }
    async fn post_form(&self, path: &str, form: &[(&str, &str)]) -> Result<serde_json::Value> {
        Ok(self.http.post(format!("{BASE}/{path}")).basic_auth(&self.key, None::<&str>).form(form).send().await?.error_for_status()?.json().await?)
    }
}

#[async_trait::async_trait]
impl CpqBackend for StripeBillingBackend {
    async fn list_products(&self, limit: u32) -> Result<Vec<Product>> {
        let resp = self.get(&format!("products?limit={limit}&active=true")).await?;
        let mut products = Vec::new();
        for p in resp["data"].as_array().unwrap_or(&vec![]) {
            let price_resp = self.get(&format!("prices?product={}&limit=1&active=true", p["id"].as_str().unwrap_or(""))).await?;
            let price = price_resp["data"].as_array().and_then(|a| a.first());
            products.push(Product { id: p["id"].as_str().unwrap_or("").into(), name: p["name"].as_str().unwrap_or("").into(), description: p["description"].as_str().map(Into::into), unit_price: price.and_then(|pr| pr["unit_amount"].as_f64()).map(|a| a / 100.0).unwrap_or(0.0), currency: price.and_then(|pr| pr["currency"].as_str()).unwrap_or("usd").to_uppercase(), recurring: price.and_then(|pr| pr["recurring"]["interval"].as_str()).map(Into::into) });
        }
        Ok(products)
    }

    async fn calculate_quote(&self, items: &[QuoteLineItem], currency: &str) -> Result<Quote> {
        let subtotal: f64 = items.iter().map(|i| i.unit_price * i.quantity as f64).sum();
        let discount_total: f64 = items.iter().map(|i| i.unit_price * i.quantity as f64 * i.discount_percent.unwrap_or(0.0) / 100.0).sum();
        let total = subtotal - discount_total;
        let calculated_items: Vec<QuoteLineItem> = items.iter().map(|i| { let disc = i.unit_price * i.quantity as f64 * i.discount_percent.unwrap_or(0.0) / 100.0; QuoteLineItem { product_id: i.product_id.clone(), description: i.description.clone(), quantity: i.quantity, unit_price: i.unit_price, discount_percent: i.discount_percent, total: i.unit_price * i.quantity as f64 - disc } }).collect();
        Ok(Quote { id: None, line_items: calculated_items, subtotal, discount_total, total, currency: currency.to_uppercase(), valid_until: None, url: None })
    }

    async fn apply_discount(&self, _quote_id: &str, percent: Option<f64>, fixed: Option<f64>) -> Result<Quote> {
        // In a real implementation, this would modify a stored quote
        let discount = percent.unwrap_or(0.0).max(fixed.unwrap_or(0.0));
        Ok(Quote { id: Some(_quote_id.into()), line_items: vec![], subtotal: 0.0, discount_total: discount, total: 0.0, currency: "USD".into(), valid_until: None, url: None })
    }

    async fn create_quote(&self, items: &[QuoteLineItem], customer_email: &str, currency: &str, valid_days: Option<u32>) -> Result<Quote> {
        // Create customer
        let cust_resp = self.post_form("customers", &[("email", customer_email)]).await?;
        let customer_id = cust_resp["id"].as_str().unwrap_or("").to_string();

        // Create quote via JSON (Stripe also accepts JSON for complex objects)
        let _line_items: Vec<serde_json::Value> = items.iter().map(|item| {
            serde_json::json!({"price_data": {"currency": currency.to_lowercase(), "unit_amount": (item.unit_price * 100.0) as i64, "product_data": {"name": item.description}}, "quantity": item.quantity})
        }).collect();

        // Use form encoding for the quote
        let mut params = vec![("customer".to_string(), customer_id.clone())];
        for (i, item) in items.iter().enumerate() {
            params.push((format!("line_items[{}][price_data][currency]", i), currency.to_lowercase()));
            params.push((format!("line_items[{}][price_data][unit_amount]", i), ((item.unit_price * 100.0) as i64).to_string()));
            params.push((format!("line_items[{}][price_data][product_data][name]", i), item.description.clone()));
            params.push((format!("line_items[{}][quantity]", i), item.quantity.to_string()));
        }
        if let Some(days) = valid_days {
            let expires = chrono::Utc::now().timestamp() + (days as i64 * 86400);
            params.push(("expires_at".to_string(), expires.to_string()));
        }

        let form_pairs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        let resp = self.post_form("quotes", &form_pairs).await?;
        let quote_id = resp["id"].as_str().unwrap_or("").to_string();

        // Finalize
        let finalized = self.post_form(&format!("quotes/{quote_id}/finalize"), &[]).await?;
        let total = finalized["amount_total"].as_f64().map(|a| a / 100.0).unwrap_or(0.0);

        Ok(Quote { id: Some(quote_id), line_items: items.to_vec(), subtotal: total, discount_total: 0.0, total, currency: currency.to_uppercase(), valid_until: finalized["expires_at"].as_i64().map(|t| chrono::DateTime::from_timestamp(t, 0).map(|d| d.to_rfc3339()).unwrap_or_default()), url: finalized["hosted_invoice_url"].as_str().map(Into::into) })
    }

    async fn get_pipeline_forecast(&self, _period: Option<&str>) -> Result<Forecast> {
        let resp = self.get("quotes?limit=100&status=open").await?;
        let empty = vec![];
        let quotes = resp["data"].as_array().unwrap_or(&empty);
        let weighted: f64 = quotes.iter().map(|q| q["amount_total"].as_f64().unwrap_or(0.0) / 100.0 * 0.5).sum();
        let best_case: f64 = quotes.iter().map(|q| q["amount_total"].as_f64().unwrap_or(0.0) / 100.0).sum();
        Ok(Forecast { period: "current".into(), weighted_pipeline: weighted, best_case, commit: weighted * 0.7, closed_won: 0.0, currency: "USD".into() })
    }

    async fn get_quota_progress(&self, _period: Option<&str>) -> Result<QuotaProgress> {
        let resp = self.get("quotes?limit=100&status=accepted").await?;
        let empty = vec![];
        let quotes = resp["data"].as_array().unwrap_or(&empty);
        let attainment: f64 = quotes.iter().map(|q| q["amount_total"].as_f64().unwrap_or(0.0) / 100.0).sum();
        let quota = 100000.0;
        Ok(QuotaProgress { period: "current".into(), quota, attainment, attainment_percent: attainment / quota * 100.0, deals_closed: quotes.len() as u32, currency: "USD".into() })
    }
}
