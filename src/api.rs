//! The operations of the personal API, one function per endpoint.
//!
//! Each function returns the response body as JSON so `--json` can print it
//! unchanged. [`ENDPOINTS`] lists every operation the CLI calls; the contract
//! test checks it against the vendored `openapi/openapi.json`.

use serde_json::Value;

use crate::client::{Client, Reply};
use crate::error::Result;
use crate::models::{Account, Category, Me, Space, Tag};

/// Every `(method, path template)` the CLI calls.
pub const ENDPOINTS: &[(&str, &str)] = &[
    ("GET", "/v1/me"),
    ("GET", "/v1/notifications"),
    ("POST", "/v1/pix-payments"),
    ("GET", "/v1/pix-payments/{pix_payment_id}"),
    ("GET", "/v1/spaces"),
    ("GET", "/v1/spaces/{space_id}/accounts"),
    ("GET", "/v1/spaces/{space_id}/cards"),
    ("GET", "/v1/spaces/{space_id}/cards/{card_id}/bills"),
    ("GET", "/v1/spaces/{space_id}/categories"),
    ("GET", "/v1/spaces/{space_id}/connections"),
    ("GET", "/v1/spaces/{space_id}/fixed-bills"),
    ("GET", "/v1/spaces/{space_id}/goals"),
    ("GET", "/v1/spaces/{space_id}/installment-plans"),
    ("GET", "/v1/spaces/{space_id}/investments"),
    ("GET", "/v1/spaces/{space_id}/overview"),
    ("GET", "/v1/spaces/{space_id}/subscriptions"),
    ("GET", "/v1/spaces/{space_id}/tags"),
    ("GET", "/v1/spaces/{space_id}/transactions"),
    ("POST", "/v1/spaces/{space_id}/transactions"),
    ("GET", "/v1/spaces/{space_id}/transactions/{transaction_id}"),
    ("PATCH", "/v1/spaces/{space_id}/transactions/{transaction_id}"),
    ("DELETE", "/v1/spaces/{space_id}/transactions/{transaction_id}"),
];

/// Query parameters as `(name, value)` pairs, skipping the unset ones.
#[derive(Debug, Default)]
pub struct Query(Vec<(&'static str, String)>);

impl Query {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn opt(mut self, name: &'static str, value: Option<impl ToString>) -> Self {
        if let Some(value) = value {
            self.0.push((name, value.to_string()));
        }
        self
    }

    #[must_use]
    pub fn flag(self, name: &'static str, on: bool) -> Self {
        self.opt(name, on.then_some(true))
    }

    #[must_use]
    pub fn as_slice(&self) -> &[(&'static str, String)] {
        &self.0
    }
}

/// A space-scoped collection such as `accounts` or `fixed-bills`.
fn space_path(space_id: &str, collection: &str) -> String {
    format!("/v1/spaces/{space_id}/{collection}")
}

fn transaction_path(space_id: &str, transaction_id: &str) -> String {
    format!("/v1/spaces/{space_id}/transactions/{transaction_id}")
}

impl Client {
    pub fn me(&self) -> Result<Value> {
        self.get_json("/v1/me", &[])
    }

    pub fn me_typed(&self) -> Result<Me> {
        data(self.me()?)
    }

    pub fn spaces(&self) -> Result<Value> {
        self.get_json("/v1/spaces", &[])
    }

    pub fn spaces_typed(&self) -> Result<Vec<Space>> {
        data(self.spaces()?)
    }

    /// Lists a space collection that takes no parameters beyond `query`.
    pub fn space_collection(&self, space_id: &str, collection: &str, query: &Query) -> Result<Value> {
        self.get_json(&space_path(space_id, collection), query.as_slice())
    }

    pub fn accounts_typed(&self, space_id: &str) -> Result<Vec<Account>> {
        let query = Query::new().flag("include_hidden", true);
        data(self.space_collection(space_id, "accounts", &query)?)
    }

    pub fn categories_typed(&self, space_id: &str) -> Result<Vec<Category>> {
        let query = Query::new().flag("include_hidden", true);
        data(self.space_collection(space_id, "categories", &query)?)
    }

    pub fn tags_typed(&self, space_id: &str) -> Result<Vec<Tag>> {
        data(self.space_collection(space_id, "tags", &Query::new())?)
    }

    pub fn card_bills(&self, space_id: &str, card_id: &str, query: &Query) -> Result<Value> {
        self.get_json(
            &format!("/v1/spaces/{space_id}/cards/{card_id}/bills"),
            query.as_slice(),
        )
    }

    pub fn notifications(&self, query: &Query) -> Result<Value> {
        self.get_json("/v1/notifications", query.as_slice())
    }

    /// Starts a Pix; the user confirms it at its `start_url`.
    pub fn create_pix_payment(&self, body: &Value, idempotency_key: &str) -> Result<Reply> {
        self.post_json("/v1/pix-payments", body, idempotency_key)
    }

    pub fn pix_payment(&self, pix_payment_id: &str) -> Result<Value> {
        self.get_json(&format!("/v1/pix-payments/{pix_payment_id}"), &[])
    }

    pub fn transactions(&self, space_id: &str, query: &Query) -> Result<Value> {
        self.space_collection(space_id, "transactions", query)
    }

    pub fn transaction(&self, space_id: &str, transaction_id: &str) -> Result<Value> {
        self.get_json(&transaction_path(space_id, transaction_id), &[])
    }

    pub fn create_transaction(&self, space_id: &str, body: &Value, idempotency_key: &str) -> Result<Reply> {
        self.post_json(&space_path(space_id, "transactions"), body, idempotency_key)
    }

    pub fn update_transaction(&self, space_id: &str, transaction_id: &str, body: &Value) -> Result<Value> {
        self.patch_json(&transaction_path(space_id, transaction_id), body)
            .map(|reply| reply.body)
    }

    pub fn delete_transaction(&self, space_id: &str, transaction_id: &str) -> Result<()> {
        self.delete(&transaction_path(space_id, transaction_id))
    }
}

/// Decodes the `data` member of a response body.
pub fn data<T: serde::de::DeserializeOwned>(mut body: Value) -> Result<T> {
    crate::client::decode(body["data"].take())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_skips_unset_values() {
        let query = Query::new()
            .opt("limit", Some(10))
            .opt("after", None::<String>)
            .flag("unread", false)
            .flag("include_hidden", true);

        assert_eq!(
            query.as_slice(),
            &[("limit", "10".to_owned()), ("include_hidden", "true".to_owned())]
        );
    }
}
