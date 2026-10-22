//! End-to-end tests: run the `visor` binary against a mock API.

use assert_cmd::Command;
use mockito::{Matcher, Mock, Server, ServerGuard};
use predicates::prelude::*;
use serde_json::{Value, json};
use tempfile::TempDir;

const TOKEN: &str = "visor_pat_abcd2345_secret";
const SPACE: &str = "11111111-1111-4111-8111-111111111111";
const OTHER_SPACE: &str = "22222222-2222-4222-8222-222222222222";
const ACCOUNT: &str = "33333333-3333-4333-8333-333333333333";
const CATEGORY: &str = "44444444-4444-4444-8444-444444444444";
const TAG: &str = "55555555-5555-4555-8555-555555555555";
const TX: &str = "66666666-6666-4666-8666-666666666666";

struct Harness {
    server: ServerGuard,
    config: TempDir,
}

impl Harness {
    fn new() -> Self {
        Self {
            server: Server::new(),
            config: TempDir::new().unwrap(),
        }
    }

    /// The binary, isolated from the developer's own config and keychain.
    fn visor(&self) -> Command {
        let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("visor"));
        cmd.env("VISOR_API_URL", self.server.url())
            .env("VISOR_AUTH_URL", self.server.url())
            .env("VISOR_CONFIG_DIR", self.config.path())
            .env("VISOR_CREDENTIAL_STORE", "file")
            .env("VISOR_TOKEN", TOKEN)
            .env_remove("VISOR_SPACE");
        cmd
    }

    fn get(&mut self, path: &str, body: &Value) -> Mock {
        self.server
            .mock("GET", path)
            .match_query(Matcher::Any)
            .match_header("authorization", format!("Bearer {TOKEN}").as_str())
            .with_header("content-type", "application/json")
            .with_body(body.to_string())
            .create()
    }

    fn spaces(&mut self) -> Mock {
        self.get(
            "/v1/spaces",
            &json!({"data": [
                {"id": OTHER_SPACE, "name": "Casa", "type": "family", "default": false, "role": "member"},
                {"id": SPACE, "name": "Pessoal", "type": "personal", "default": true, "role": "owner"}
            ]}),
        )
    }

    fn reference_data(&mut self) {
        self.get(
            &format!("/v1/spaces/{SPACE}/accounts"),
            &json!({"data": [{
                "id": ACCOUNT, "name": "Nubank", "type": "bank", "source": "connected",
                "balance": "1520.30", "currency_code": "BRL", "hidden": false, "provider_name": "Nubank"
            }]}),
        );
        self.get(
            &format!("/v1/spaces/{SPACE}/categories"),
            &json!({"data": [{
                "id": CATEGORY, "key": "category.groceries", "name": "Groceries", "parent_id": null,
                "type": "expense", "kind": "system", "hidden": false
            }]}),
        );
        self.get(
            &format!("/v1/spaces/{SPACE}/tags"),
            &json!({"data": [{"id": TAG, "name": "viagem", "color": "#ff0000"}]}),
        );
    }
}

fn me() -> Value {
    json!({"data": {
        "id": "77777777-7777-4777-8777-777777777777", "name": "Ana", "email": "ana@example.com",
        "locale": "pt-BR", "display_currency_code": "BRL", "plan": "premium",
        "active_space_id": SPACE, "created_at": "2026-01-02T03:04:05Z"
    }})
}

fn transaction() -> Value {
    json!({
        "id": TX, "account_id": ACCOUNT, "category_id": CATEGORY, "tag_ids": [],
        "amount": "18.90", "type": "debit", "currency_code": "BRL", "source": "manual",
        "status": "posted", "ignored": false, "description": "Padaria", "merchant_name": null,
        "notes": null, "transacted_at": "2026-10-01T15:00:00Z",
        "created_at": "2026-10-01T15:00:00Z", "updated_at": "2026-10-01T15:00:00Z"
    })
}

#[test]
fn help_lists_commands_and_exit_codes() {
    let harness = Harness::new();

    harness
        .visor()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("transactions"))
        .stdout(predicate::str::contains("Exit codes"));
}

#[test]
fn version_prints_the_crate_version() {
    Harness::new()
        .visor()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn completions_generate_a_script() {
    Harness::new()
        .visor()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef visor"));
}

#[test]
fn commands_fail_with_exit_code_3_when_not_logged_in() {
    Harness::new()
        .visor()
        .env_remove("VISOR_TOKEN")
        .arg("me")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("visor auth login"));
}

#[test]
fn login_checks_the_token_then_stores_it() {
    let mut harness = Harness::new();
    let me_mock = harness.get("/v1/me", &me());
    harness.spaces();

    harness
        .visor()
        .env_remove("VISOR_TOKEN")
        .args(["auth", "login", "--with-token"])
        .write_stdin(format!("{TOKEN}\n"))
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "Logged in to Visor as Ana (ana@example.com)",
        ))
        .stderr(predicate::str::contains("Next steps"));
    me_mock.assert();

    let stored: Value = serde_json::from_str(
        &std::fs::read_to_string(harness.config.path().join("credentials.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(stored, json!({"kind": "personal_token", "token": TOKEN}));

    // The stored token now authenticates other commands.
    harness
        .visor()
        .env_remove("VISOR_TOKEN")
        .args(["auth", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("visor_pat_abcd2345_…"))
        .stdout(predicate::str::contains(TOKEN).not());
}

#[test]
fn login_rejects_a_token_the_api_refuses_and_stores_nothing() {
    let mut harness = Harness::new();
    harness
        .server
        .mock("GET", "/v1/me")
        .with_status(401)
        .with_body(r#"{"errors":[{"code":"TOKEN_REVOKED","message":"The token was revoked.","details":{},"request_id":"r1"}]}"#)
        .create();

    harness
        .visor()
        .env_remove("VISOR_TOKEN")
        .args(["auth", "login", "--with-token"])
        .write_stdin(TOKEN)
        .assert()
        .code(3)
        .stderr(predicate::str::contains("TOKEN_REVOKED"))
        .stderr(predicate::str::contains("Request ID: r1"));

    assert!(!harness.config.path().join("credentials.json").exists());
}

#[test]
fn login_rejects_something_that_is_not_a_token() {
    Harness::new()
        .visor()
        .args(["auth", "login", "--with-token"])
        .write_stdin("hunter2")
        .assert()
        .code(2)
        .stderr(predicate::str::contains("visor_pat_"));
}

#[test]
fn logout_removes_the_stored_token() {
    let harness = Harness::new();
    std::fs::write(
        harness.config.path().join("credentials.json"),
        json!({"kind": "personal_token", "token": TOKEN}).to_string(),
    )
    .unwrap();

    harness
        .visor()
        .args(["auth", "logout"])
        .assert()
        .success()
        .stderr(predicate::str::contains("revoke"));

    assert!(!harness.config.path().join("credentials.json").exists());
}

#[test]
fn me_prints_json_unchanged() {
    let mut harness = Harness::new();
    harness.get("/v1/me", &me());

    let output = harness
        .visor()
        .args(["me", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(serde_json::from_slice::<Value>(&output).unwrap(), me());
}

#[test]
fn spaces_use_saves_the_default_space_by_name() {
    let mut harness = Harness::new();
    harness.spaces();

    harness.visor().args(["spaces", "use", "casa"]).assert().success();

    let config = std::fs::read_to_string(harness.config.path().join("config.toml")).unwrap();
    assert!(config.contains(OTHER_SPACE), "{config}");

    // Later commands act on that space without fetching the space list.
    let accounts = harness.get(
        &format!("/v1/spaces/{OTHER_SPACE}/accounts"),
        &json!({"data": []}),
    );
    harness
        .visor()
        .args(["accounts", "list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("No accounts found."));
    accounts.assert();
}

#[test]
fn space_flag_overrides_the_default_space() {
    let mut harness = Harness::new();
    let tags = harness.get(&format!("/v1/spaces/{OTHER_SPACE}/tags"), &json!({"data": []}));

    harness
        .visor()
        .args(["tags", "list", "--space", OTHER_SPACE])
        .assert()
        .success();

    tags.assert();
}

#[test]
fn accounts_list_prints_a_table() {
    let mut harness = Harness::new();
    harness.spaces();
    harness.reference_data();

    harness
        .visor()
        .args(["accounts", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("NAME"))
        .stdout(predicate::str::contains("Nubank"))
        .stdout(predicate::str::contains("1520.30 BRL"));
}

#[test]
fn transaction_list_resolves_names_and_shows_the_next_page() {
    let mut harness = Harness::new();
    harness.spaces();
    harness.reference_data();
    let list = harness
        .server
        .mock("GET", format!("/v1/spaces/{SPACE}/transactions").as_str())
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("category_id".into(), CATEGORY.into()),
            Matcher::UrlEncoded("from_date".into(), "2026-10-01".into()),
            Matcher::UrlEncoded("limit".into(), "1".into()),
        ]))
        .with_body(json!({"data": [transaction()], "next_cursor": "c2"}).to_string())
        .create();

    harness
        .visor()
        .args([
            "tx",
            "list",
            "--category",
            "groceries",
            "--from",
            "2026-10-01",
            "--limit",
            "1",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Padaria"))
        .stdout(predicate::str::contains("Groceries"))
        .stdout(predicate::str::contains("-18.90 BRL"))
        .stderr(predicate::str::contains("--after c2"));
    list.assert();
}

#[test]
fn transaction_list_all_follows_every_cursor() {
    let mut harness = Harness::new();
    harness.spaces();
    let path = format!("/v1/spaces/{SPACE}/transactions");
    let mut second = transaction();
    second["id"] = json!("88888888-8888-4888-8888-888888888888");
    let first_page = harness
        .server
        .mock("GET", path.as_str())
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("limit".into(), "100".into()),
            Matcher::Regex("^limit=100$".into()),
        ]))
        .with_body(json!({"data": [transaction()], "next_cursor": "c2"}).to_string())
        .create();
    let second_page = harness
        .server
        .mock("GET", path.as_str())
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("after".into(), "c2".into()),
            Matcher::UrlEncoded("limit".into(), "100".into()),
        ]))
        .with_body(json!({"data": [second], "next_cursor": null}).to_string())
        .create();

    let output = harness
        .visor()
        .args(["tx", "list", "--all", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let body: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 2);
    assert_eq!(body["next_cursor"], Value::Null);
    first_page.assert();
    second_page.assert();
}

#[test]
fn transaction_add_sends_a_complete_body_with_an_idempotency_key() {
    let mut harness = Harness::new();
    harness.spaces();
    harness.reference_data();
    let create = harness
        .server
        .mock("POST", format!("/v1/spaces/{SPACE}/transactions").as_str())
        .match_header("idempotency-key", "padaria-1")
        .match_body(Matcher::Json(json!({
            "account_id": ACCOUNT,
            "category_id": CATEGORY,
            "amount": "18.90",
            "type": "debit",
            "description": "Padaria",
            "transacted_at": "2026-10-01T15:00:00+00:00",
            "tag_ids": [TAG],
        })))
        .with_status(201)
        .with_header("idempotent-replayed", "false")
        .with_body(json!({"data": transaction()}).to_string())
        .create();

    harness
        .visor()
        .args([
            "tx",
            "add",
            "--amount",
            "18,90",
            "--description",
            "Padaria",
            "--category",
            "Groceries",
            "--account",
            "nubank",
            "--tag",
            "viagem",
            "--at",
            "2026-10-01T15:00:00Z",
            "--idempotency-key",
            "padaria-1",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("Recorded. Idempotency key: padaria-1"))
        .stdout(predicate::str::contains("Padaria"));
    create.assert();
}

#[test]
fn transaction_add_reports_an_idempotent_replay() {
    let mut harness = Harness::new();
    harness.spaces();
    harness.reference_data();
    harness
        .server
        .mock("POST", format!("/v1/spaces/{SPACE}/transactions").as_str())
        .with_status(201)
        .with_header("idempotent-replayed", "true")
        .with_body(json!({"data": transaction()}).to_string())
        .create();

    harness
        .visor()
        .args([
            "tx",
            "add",
            "--amount",
            "18.90",
            "-d",
            "Padaria",
            "-c",
            CATEGORY,
            "-a",
            ACCOUNT,
            "--idempotency-key",
            "padaria-1",
            "--json",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("Already recorded"));
}

#[test]
fn transaction_add_rejects_a_bad_amount_before_calling_the_api() {
    Harness::new()
        .visor()
        .args([
            "tx", "add", "--amount", "-5", "-d", "x", "-c", CATEGORY, "-a", ACCOUNT,
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--type debit or --type credit"));
}

#[test]
fn transaction_add_explains_an_unknown_category() {
    let mut harness = Harness::new();
    harness.spaces();
    harness.reference_data();

    harness
        .visor()
        .args([
            "tx", "add", "--amount", "1", "-d", "x", "-c", "Pets", "-a", ACCOUNT,
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("visor categories list"));
}

#[test]
fn transaction_add_maps_validation_errors_to_exit_code_5() {
    let mut harness = Harness::new();
    harness.spaces();
    harness
        .server
        .mock("POST", format!("/v1/spaces/{SPACE}/transactions").as_str())
        .with_status(422)
        .with_body(
            json!({"errors": [{"code": "VALIDATION_FAILED", "message": "The body has invalid values.",
                "details": {"errors": [{"field": "account_id", "message": "is not an account of the space"}]}}]})
            .to_string(),
        )
        .create();

    harness
        .visor()
        .args([
            "tx", "add", "--amount", "1", "-d", "x", "-c", CATEGORY, "-a", ACCOUNT,
        ])
        .assert()
        .code(5)
        .stderr(predicate::str::contains(
            "account_id: is not an account of the space",
        ));
}

#[test]
fn transaction_edit_sends_only_the_given_fields() {
    let mut harness = Harness::new();
    harness.spaces();
    harness.reference_data();
    let update = harness
        .server
        .mock("PATCH", format!("/v1/spaces/{SPACE}/transactions/{TX}").as_str())
        .match_body(Matcher::Json(
            json!({"notes": "", "tag_ids": [], "ignored": true}),
        ))
        .with_body(json!({"data": transaction()}).to_string())
        .create();

    harness
        .visor()
        .args([
            "tx",
            "edit",
            TX,
            "--notes",
            "",
            "--clear-tags",
            "--ignored",
            "true",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("Updated."));
    update.assert();
}

#[test]
fn transaction_edit_needs_a_field() {
    let mut harness = Harness::new();
    harness.spaces();

    harness
        .visor()
        .args(["tx", "edit", TX])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("nothing to change"));
}

#[test]
fn transaction_delete_needs_yes_without_a_terminal() {
    let mut harness = Harness::new();
    harness.spaces();

    harness
        .visor()
        .args(["tx", "delete", TX])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("--yes"));
}

#[test]
fn transaction_delete_with_yes_deletes() {
    let mut harness = Harness::new();
    harness.spaces();
    let delete = harness
        .server
        .mock("DELETE", format!("/v1/spaces/{SPACE}/transactions/{TX}").as_str())
        .with_status(204)
        .create();

    harness.visor().args(["tx", "rm", TX, "--yes"]).assert().success();

    delete.assert();
}

#[test]
fn transaction_get_maps_not_found_to_exit_code_4() {
    let mut harness = Harness::new();
    harness.spaces();
    harness
        .server
        .mock("GET", format!("/v1/spaces/{SPACE}/transactions/{TX}").as_str())
        .with_status(404)
        .with_body(r#"{"errors":[{"code":"NOT_FOUND","message":"Not found."}]}"#)
        .create();

    harness
        .visor()
        .args(["tx", "get", TX])
        .assert()
        .code(4)
        .stderr(predicate::str::contains("NOT_FOUND"));
}

#[test]
fn transaction_ids_are_validated_before_building_a_path() {
    Harness::new()
        .visor()
        .args(["tx", "get", "../../me"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("is not an ID"));
}

#[test]
fn insufficient_scope_names_the_missing_scope() {
    let mut harness = Harness::new();
    harness.spaces();
    harness
        .server
        .mock("DELETE", format!("/v1/spaces/{SPACE}/transactions/{TX}").as_str())
        .with_status(403)
        .with_body(r#"{"errors":[{"code":"INSUFFICIENT_SCOPE","message":"Missing scope.","details":{"required_scope":"pfm:transactions:write"}}]}"#)
        .create();

    harness
        .visor()
        .args(["tx", "rm", TX, "-y"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("pfm:transactions:write"));
}

#[test]
fn overview_prints_balances_and_upcoming_bills() {
    let mut harness = Harness::new();
    harness.spaces();
    harness.get(
        &format!("/v1/spaces/{SPACE}/overview"),
        &json!({"data": {
            "period_start": "2026-10-01", "period_end": "2026-10-31", "currency_code": "BRL",
            "bank_balance": "5000.00", "credit_card_debt": "-1200.00", "open_bills_total": "1200.00",
            "net_worth": "3800.00",
            "upcoming_bills": [{"account_id": ACCOUNT, "description": "Fatura Nubank", "amount": "1200.00",
                "due_date": "2026-10-25", "amount_is_estimate": true}],
            "period": {"income": "8000.00", "expenses": "3000.00", "result": "5000.00"},
            "projection": {"current_balance": "5000.00", "expected_income": "0.00",
                "committed_outflows": {"bills": "1200.00", "installments": "0.00", "subscriptions": "50.00", "scheduled": "0.00"},
                "estimated_day_to_day": "900.00", "projected_balance": "2850.00"}
        }}),
    );

    harness
        .visor()
        .arg("overview")
        .assert()
        .success()
        .stdout(predicate::str::contains("Projected balance"))
        .stdout(predicate::str::contains("2850.00 BRL"))
        .stdout(predicate::str::contains("Fatura Nubank"))
        .stdout(predicate::str::contains("(estimate)"));
}

#[test]
fn rate_limit_is_retried_after_retry_after() {
    let mut harness = Harness::new();
    let limited = harness
        .server
        .mock("GET", "/v1/me")
        .with_status(429)
        .with_header("retry-after", "0")
        .with_body(r#"{"errors":[{"code":"RATE_LIMITED","message":"Too many requests."}]}"#)
        .expect(1)
        .create();
    harness.get("/v1/me", &me());

    harness
        .visor()
        .arg("me")
        .assert()
        .success()
        .stdout(predicate::str::contains("ana@example.com"));
    limited.assert();
}

#[test]
fn no_command_prints_the_help_without_a_banner_in_a_pipe() {
    Harness::new()
        .visor()
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: visor"))
        .stderr(predicate::str::contains("▀").not());
}

#[test]
fn whoami_and_auth_status_summarize_the_login() {
    let mut harness = Harness::new();
    harness.get("/v1/me", &me());
    harness.spaces();

    for args in [vec!["whoami"], vec!["auth", "status"]] {
        harness
            .visor()
            .args(&args)
            .assert()
            .success()
            .stdout(predicate::str::contains(
                "Logged in to Visor as Ana (ana@example.com)",
            ))
            .stdout(predicate::str::contains(
                "personal API token visor_pat_abcd2345_…",
            ))
            .stdout(predicate::str::contains("VISOR_TOKEN (environment)"))
            .stdout(predicate::str::contains("Space      Pessoal · 2 spaces"))
            .stdout(predicate::str::contains("secret").not());
    }
}

#[test]
fn auth_status_json_has_stable_keys_and_no_secret() {
    let mut harness = Harness::new();
    harness.get("/v1/me", &me());
    harness.spaces();

    let output = harness
        .visor()
        .args(["auth", "status", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let status: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(status["authenticated"], true);
    assert_eq!(status["user"]["email"], "ana@example.com");
    assert_eq!(
        status["login"],
        json!({"kind": "personal_token", "token": "visor_pat_abcd2345_…"})
    );
    assert_eq!(status["credential_source"], "env");
    assert_eq!(status["space"]["id"], SPACE);
    assert_eq!(status["spaces"], 2);
    assert!(!String::from_utf8_lossy(&output).contains("secret"));
}

#[test]
fn doctor_passes_and_reports_an_available_update() {
    let mut harness = Harness::new();
    harness.get("/v1/me", &me());
    harness.spaces();
    harness
        .server
        .mock("GET", "/releases/latest")
        .with_body(r#"{"tag_name":"v99.0.0"}"#)
        .create();

    harness
        .visor()
        .arg("doctor")
        .env(
            "VISOR_RELEASES_URL",
            format!("{}/releases/latest", harness.server.url()),
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("99.0.0 is out"))
        .stdout(predicate::str::contains("✓ Login"))
        .stdout(predicate::str::contains(
            "✓ Visor API    ana@example.com · premium plan",
        ))
        .stdout(predicate::str::contains(
            "✓ Space        Pessoal (your default space)",
        ));
}

#[test]
fn doctor_fails_when_not_logged_in() {
    let harness = Harness::new();

    let output = harness
        .visor()
        .args(["doctor", "--json"])
        .env_remove("VISOR_TOKEN")
        .env("VISOR_RELEASES_URL", "http://127.0.0.1:9/unreachable")
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    let report: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(report["ok"], false);
    let checks = report["checks"].as_array().unwrap();
    assert_eq!(checks[0]["name"], "CLI version");
    assert_eq!(checks[0]["status"], "warn");
    assert_eq!(checks[1]["name"], "Login");
    assert_eq!(checks[1]["status"], "fail");
}

#[test]
fn doctor_fails_when_the_api_rejects_the_token() {
    let mut harness = Harness::new();
    harness
        .server
        .mock("GET", "/v1/me")
        .with_status(401)
        .with_body(r#"{"errors":[{"code":"TOKEN_REVOKED","message":"The token was revoked."}]}"#)
        .create();

    harness
        .visor()
        .arg("doctor")
        .env("VISOR_RELEASES_URL", "http://127.0.0.1:9/unreachable")
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "✗ Visor API    The token was revoked. (TOKEN_REVOKED)",
        ))
        .stderr(predicate::str::contains("1 check(s) failed"));
}

#[test]
fn json_errors_are_json_on_stderr() {
    let mut harness = Harness::new();
    harness.spaces();
    harness
        .server
        .mock("GET", format!("/v1/spaces/{SPACE}/transactions/{TX}").as_str())
        .with_status(404)
        .with_header("x-request-id", "req-9")
        .with_body(r#"{"errors":[{"code":"NOT_FOUND","message":"Not found.","details":{}}]}"#)
        .create();

    let output = harness
        .visor()
        .args(["tx", "get", TX, "--json"])
        .assert()
        .code(4)
        .get_output()
        .clone();

    assert_eq!(output.stdout, b"");
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["code"], "NOT_FOUND");
    assert_eq!(error["error"]["status"], 404);
    assert_eq!(error["error"]["request_id"], "req-9");
    assert_eq!(error["error"]["exit_code"], 4);
}

#[test]
fn local_errors_are_json_too() {
    let output = Harness::new()
        .visor()
        .env_remove("VISOR_TOKEN")
        .args(["me", "--json"])
        .assert()
        .code(3)
        .get_output()
        .clone();

    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["code"], "NOT_LOGGED_IN");
}

#[test]
fn debug_logs_each_request_without_the_token() {
    let mut harness = Harness::new();
    harness
        .server
        .mock("GET", "/v1/me")
        .with_header("x-request-id", "req-1")
        .with_body(me().to_string())
        .create();

    harness
        .visor()
        .args(["me", "--debug"])
        .assert()
        .success()
        .stderr(predicate::str::contains("[debug] GET"))
        .stderr(predicate::str::contains("/v1/me → 200 in"))
        .stderr(predicate::str::contains("(request id req-1)"))
        .stderr(predicate::str::contains("secret").not());
}

#[test]
fn transaction_list_exports_csv_with_names_and_signed_amounts() {
    let mut harness = Harness::new();
    harness.spaces();
    harness.reference_data();
    let mut formula = transaction();
    formula["id"] = json!("88888888-8888-4888-8888-888888888888");
    formula["description"] = json!("=HYPERLINK(\"evil\")");
    formula["type"] = json!("credit");
    formula["amount"] = json!("1000.00");
    let mut tagged = transaction();
    tagged["tag_ids"] = json!([TAG]);
    tagged["notes"] = json!("pão, leite");
    harness
        .server
        .mock("GET", format!("/v1/spaces/{SPACE}/transactions").as_str())
        .match_query(Matcher::Any)
        .with_body(json!({"data": [tagged, formula], "next_cursor": null}).to_string())
        .create();

    let output = harness
        .visor()
        .args(["tx", "list", "--csv"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).unwrap();
    let lines: Vec<&str> = text.split("\r\n").collect();

    assert!(lines[0].starts_with("date,time,description,amount,currency,type,category,account,tags"));
    assert!(
        lines[1].contains(",-18.90,BRL,debit,Groceries,Nubank,viagem,"),
        "{}",
        lines[1]
    );
    assert!(lines[1].contains("\"pão, leite\""), "{}", lines[1]);
    assert!(lines[2].contains("\"'=HYPERLINK(\"\"evil\"\")\""), "{}", lines[2]);
    assert!(lines[2].contains(",1000.00,BRL,credit,"), "{}", lines[2]);
}

#[test]
fn open_prints_the_page_url() {
    Harness::new()
        .visor()
        .args(["open", "tokens", "--print"])
        .assert()
        .success()
        .stdout("https://dashboard.visorfinance.app/settings/connected_apps\n");
}

#[test]
fn no_update_notice_in_a_pipe() {
    let mut harness = Harness::new();
    harness.get("/v1/me", &me());
    let releases = harness
        .server
        .mock("GET", "/releases/latest")
        .with_body(r#"{"tag_name":"v99.0.0"}"#)
        .expect(0)
        .create();

    harness
        .visor()
        .arg("me")
        .env(
            "VISOR_RELEASES_URL",
            format!("{}/releases/latest", harness.server.url()),
        )
        .assert()
        .success()
        .stderr(predicate::str::contains("is out").not());
    releases.assert();
}

#[test]
fn transaction_list_stops_at_50_by_default() {
    let mut harness = Harness::new();
    harness.spaces();
    harness.reference_data();
    let first = harness
        .server
        .mock("GET", format!("/v1/spaces/{SPACE}/transactions").as_str())
        .match_query(Matcher::UrlEncoded("limit".into(), "50".into()))
        .with_body(json!({"data": vec![transaction(); 50], "next_cursor": "c2"}).to_string())
        .expect(1)
        .create();
    let more = harness
        .server
        .mock("GET", format!("/v1/spaces/{SPACE}/transactions").as_str())
        .match_query(Matcher::UrlEncoded("after".into(), "c2".into()))
        .expect(0)
        .create();

    harness
        .visor()
        .args(["tx", "list"])
        .assert()
        .success()
        .stderr(predicate::str::contains("raise `--limit`"));
    first.assert();
    more.assert();
}

#[test]
fn transaction_list_limit_spans_pages_and_stops_at_the_total() {
    let mut harness = Harness::new();
    harness.spaces();
    let path = format!("/v1/spaces/{SPACE}/transactions");
    let page = |count: usize, cursor: &str| {
        let rows: Vec<Value> = (0..count)
            .map(|i| {
                let mut tx = transaction();
                tx["id"] = json!(format!("00000000-0000-4000-8000-{i:012}"));
                tx
            })
            .collect();
        json!({"data": rows, "next_cursor": cursor}).to_string()
    };
    let first = harness
        .server
        .mock("GET", path.as_str())
        .match_query(Matcher::Regex("^limit=100$".into()))
        .with_body(page(100, "c2"))
        .expect(1)
        .create();
    // The second page asks only for what is left: 150 - 100 = 50.
    let second = harness
        .server
        .mock("GET", path.as_str())
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("after".into(), "c2".into()),
            Matcher::UrlEncoded("limit".into(), "50".into()),
        ]))
        .with_body(page(50, "c3"))
        .expect(1)
        .create();

    let output = harness
        .visor()
        .args(["tx", "list", "--limit", "150", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let body: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(body["data"].as_array().unwrap().len(), 150);
    assert_eq!(body["next_cursor"], "c3");
    first.assert();
    second.assert();
}

#[test]
fn all_and_limit_cannot_be_combined() {
    Harness::new()
        .visor()
        .args(["tx", "list", "--all", "--limit", "10"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("cannot be used with"));
}
