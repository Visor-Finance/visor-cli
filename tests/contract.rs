//! Checks the CLI against `openapi/openapi.json`, the spec the backend
//! generates. Run `scripts/sync-openapi.sh` to update it; a failure here
//! means the API changed in a way the CLI must follow.

use std::collections::BTreeSet;

use serde::de::DeserializeOwned;
use serde_json::{Map, Value, json};
use visor::api::ENDPOINTS;
use visor::models::{
    Account, Bill, Card, Category, Connection, Goal, InstallmentPlan, Investment, InvestmentSummary, Me,
    Notification, Overview, PixPayment, RecurringExpense, Space, Tag, Transaction,
};

fn spec() -> Value {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/openapi/openapi.json"))
        .expect("openapi/openapi.json exists");
    serde_json::from_str(&text).expect("openapi.json is JSON")
}

fn operations(spec: &Value) -> BTreeSet<(String, String)> {
    spec["paths"]
        .as_object()
        .unwrap()
        .iter()
        .flat_map(|(path, item)| {
            item.as_object()
                .unwrap()
                .keys()
                .filter(|method| ["get", "post", "patch", "put", "delete"].contains(&method.as_str()))
                .map(move |method| (method.to_uppercase(), path.clone()))
        })
        .collect()
}

/// Operations the API has that the CLI does not call yet. Each one is a
/// command to add; keep this list short.
const NOT_YET_COVERED: &[(&str, &str)] = &[("POST", "/v1/spaces/{space_id}/transactions/batch")];

#[test]
fn the_cli_covers_exactly_the_operations_in_the_spec() {
    let mut in_spec = operations(&spec());
    for (method, path) in NOT_YET_COVERED {
        assert!(
            in_spec.remove(&((*method).to_owned(), (*path).to_owned())),
            "{method} {path} is no longer in the spec; remove it from NOT_YET_COVERED"
        );
    }
    let in_cli: BTreeSet<(String, String)> = ENDPOINTS
        .iter()
        .map(|(method, path)| ((*method).to_owned(), (*path).to_owned()))
        .collect();

    let missing: Vec<_> = in_spec.difference(&in_cli).collect();
    let removed: Vec<_> = in_cli.difference(&in_spec).collect();
    assert!(
        missing.is_empty(),
        "the API has operations the CLI does not cover yet: {missing:?}"
    );
    assert!(
        removed.is_empty(),
        "the CLI calls operations the API no longer has: {removed:?}"
    );
}

#[test]
fn the_browser_login_urls_match_the_spec() {
    let spec = spec();
    let flow = &spec["components"]["securitySchemes"]["visor_cli_oauth"]["flows"]["authorizationCode"];
    let base = visor::oauth::DEFAULT_AUTH_URL;

    assert_eq!(flow["authorizationUrl"], format!("{base}/oauth/authorize"));
    assert_eq!(flow["tokenUrl"], format!("{base}/oauth/token"));
    assert_eq!(flow["refreshUrl"], format!("{base}/oauth/token"));
}

#[test]
fn the_spec_still_uses_bearer_personal_tokens() {
    let spec = spec();
    let scheme = &spec["components"]["securitySchemes"]["personal_api_token"];

    assert_eq!(scheme["type"], "http");
    assert_eq!(scheme["scheme"], "bearer");
}

/// Builds the smallest object the schema allows: required fields only,
/// nullable ones as `null`. If a model needs a field the spec does not
/// require, or reads it with the wrong type, deserializing it fails.
fn minimal(spec: &Value, schema: &Value) -> Value {
    if let Some(reference) = schema["$ref"].as_str() {
        let name = reference.rsplit('/').next().unwrap();
        return minimal(spec, &spec["components"]["schemas"][name]);
    }
    if schema["nullable"] == true {
        return Value::Null;
    }
    if let Some(variants) = schema["oneOf"].as_array().or(schema["anyOf"].as_array()) {
        return minimal(spec, &variants[0]);
    }
    match schema["type"].as_str() {
        Some("object") => {
            let required: BTreeSet<&str> = schema["required"]
                .as_array()
                .map(|fields| fields.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            let mut object = Map::new();
            for (field, property) in schema["properties"].as_object().into_iter().flatten() {
                if required.contains(field.as_str()) {
                    object.insert(field.clone(), minimal(spec, property));
                }
            }
            Value::Object(object)
        }
        Some("array") => json!([minimal(spec, &schema["items"])]),
        Some("integer") => json!(1),
        Some("number") => json!(1.5),
        Some("boolean") => json!(true),
        Some("string") => match &schema["enum"] {
            Value::Array(values) => values.iter().find(|v| v.is_string()).cloned().unwrap(),
            _ => json!("x"),
        },
        other => panic!("unhandled schema type {other:?} in {schema}"),
    }
}

fn decodes<T: DeserializeOwned>(schema: &str) {
    let spec = spec();
    let definition = &spec["components"]["schemas"][schema];
    assert!(
        definition.is_object(),
        "the spec no longer has the schema {schema}"
    );

    // A schema that a nullable field reuses can carry `nullable` itself
    // (`Transaction`, through the batch result); the object is what counts.
    let mut definition = definition.clone();
    definition.as_object_mut().unwrap().remove("nullable");
    let sample = minimal(&spec, &definition);
    if let Err(error) = serde_json::from_value::<T>(sample.clone()) {
        panic!("{schema} no longer matches the CLI model: {error}\nsample: {sample:#}");
    }
}

#[test]
fn every_model_decodes_the_minimal_object_of_its_schema() {
    decodes::<Me>("Me");
    decodes::<Space>("Space");
    decodes::<Account>("Account");
    decodes::<Category>("Category");
    decodes::<Tag>("Tag");
    decodes::<Transaction>("Transaction");
    decodes::<Card>("Card");
    decodes::<Bill>("Bill");
    decodes::<RecurringExpense>("RecurringExpense");
    decodes::<InstallmentPlan>("InstallmentPlan");
    decodes::<Goal>("Goal");
    decodes::<Investment>("Investment");
    decodes::<InvestmentSummary>("InvestmentSummary");
    decodes::<Connection>("Connection");
    decodes::<Notification>("Notification");
    decodes::<Overview>("Overview");
    decodes::<PixPayment>("PixPayment");
}

#[test]
fn pix_bodies_only_use_fields_the_spec_accepts() {
    let spec = spec();
    let schema = &spec["components"]["schemas"]["PixPaymentCreateRequest"];
    for field in [
        "pix_key",
        "amount",
        "description",
        "bank",
        "key_type",
        "link_device",
    ] {
        assert!(
            schema["properties"][field].is_object(),
            "PixPaymentCreateRequest no longer accepts {field}"
        );
    }
    let key_types: Vec<&str> = schema["properties"]["key_type"]["enum"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    for key_type in ["cpf", "cnpj", "phone", "email", "evp"] {
        assert!(
            key_types.contains(&key_type),
            "key_type no longer accepts {key_type}"
        );
    }
}

#[test]
fn transaction_bodies_only_use_fields_the_spec_accepts() {
    let spec = spec();
    let fields = |schema: &str| -> BTreeSet<String> {
        spec["components"]["schemas"][schema]["properties"]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect()
    };

    let create = fields("TransactionCreateRequest");
    for field in [
        "account_id",
        "category_id",
        "amount",
        "type",
        "description",
        "transacted_at",
        "merchant_name",
        "notes",
        "tag_ids",
        "ignored",
    ] {
        assert!(
            create.contains(field),
            "TransactionCreateRequest no longer accepts {field}"
        );
    }

    let update = fields("TransactionUpdateRequest");
    for field in [
        "amount",
        "description",
        "category_id",
        "type",
        "transacted_at",
        "notes",
        "tag_ids",
        "ignored",
    ] {
        assert!(
            update.contains(field),
            "TransactionUpdateRequest no longer accepts {field}"
        );
    }
}
