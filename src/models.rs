//! Typed views of the API resources, used for tables and name lookups.
//!
//! `--json` prints the response body unchanged, so these types only need the
//! fields the CLI reads. Enum-like fields stay `String` so a value the API
//! adds later does not break an older CLI. Decimals stay strings so no cent
//! is lost to floating point.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Me {
    pub id: String,
    pub name: String,
    pub email: String,
    pub locale: String,
    pub display_currency_code: String,
    pub plan: String,
    pub active_space_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Space {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub default: bool,
    pub role: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub source: String,
    pub balance: String,
    pub currency_code: String,
    pub hidden: bool,
    pub provider_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Category {
    pub id: String,
    pub key: Option<String>,
    pub name: String,
    pub parent_id: Option<String>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    #[serde(rename = "kind")]
    pub origin: String,
    pub hidden: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub account_id: String,
    pub category_id: Option<String>,
    pub tag_ids: Vec<String>,
    pub amount: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub currency_code: String,
    pub source: String,
    pub status: String,
    pub ignored: bool,
    pub description: String,
    pub merchant_name: Option<String>,
    pub notes: Option<String>,
    pub transacted_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Card {
    pub id: String,
    pub name: String,
    pub provider_name: Option<String>,
    pub currency_code: String,
    pub credit_limit: Option<String>,
    pub used_limit: Option<String>,
    pub available_limit: Option<String>,
    pub current_bill_close_date: Option<String>,
    pub current_bill_due_date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Bill {
    pub id: Option<String>,
    pub status: String,
    pub close_date: Option<String>,
    pub due_date: Option<String>,
    pub total_amount: Option<String>,
    pub paid_amount: Option<String>,
    pub minimum_payment: Option<String>,
    pub amount_is_estimate: bool,
    pub currency_code: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecurringExpense {
    pub id: String,
    pub name: String,
    pub amount: String,
    pub currency_code: String,
    pub frequency: Option<String>,
    pub next_due_date: Option<String>,
    pub last_paid_date: Option<String>,
    pub active: bool,
    pub category_id: Option<String>,
    pub account_id: Option<String>,
    pub service_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InstallmentPlan {
    pub id: String,
    pub description: String,
    pub account_id: Option<String>,
    pub installment_amount: String,
    pub total_amount: String,
    pub remaining_amount: String,
    pub currency_code: String,
    pub total_installments: i64,
    pub paid_installments: i64,
    pub first_due_date: String,
    pub next_due_date: Option<String>,
    pub status: String,
    pub category_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Goal {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub status: String,
    pub target_amount: String,
    pub current_amount: String,
    pub currency_code: String,
    pub progress_percent: i64,
    pub target_date: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Investment {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub subtype: Option<String>,
    pub category: String,
    pub source: String,
    pub balance: String,
    pub currency_code: String,
    pub provider_name: Option<String>,
    pub contributes_to_net_worth: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InvestmentSummary {
    pub total_value: String,
    pub currency_code: String,
    pub count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Connection {
    pub id: String,
    pub provider_name: Option<String>,
    pub status: String,
    pub paused: bool,
    pub requires_reauth: bool,
    pub last_successful_sync_at: Option<String>,
    pub next_auto_sync_at: Option<String>,
    pub account_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Notification {
    pub id: String,
    pub topic: String,
    pub title: String,
    pub body: String,
    pub read: bool,
    pub read_at: Option<String>,
    pub space_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Overview {
    pub period_start: String,
    pub period_end: String,
    pub currency_code: String,
    pub bank_balance: String,
    pub credit_card_debt: String,
    pub open_bills_total: String,
    pub net_worth: String,
    pub upcoming_bills: Vec<OverviewUpcomingBill>,
    pub period: OverviewPeriod,
    pub projection: OverviewProjection,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OverviewPeriod {
    pub income: String,
    pub expenses: String,
    pub result: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OverviewProjection {
    pub current_balance: String,
    pub expected_income: String,
    pub committed_outflows: OverviewCommittedOutflows,
    pub estimated_day_to_day: String,
    pub projected_balance: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OverviewCommittedOutflows {
    pub bills: String,
    pub installments: String,
    pub subscriptions: String,
    pub scheduled: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OverviewUpcomingBill {
    pub account_id: String,
    pub description: String,
    pub amount: String,
    pub due_date: String,
    pub amount_is_estimate: bool,
}
