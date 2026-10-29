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
            .env("BROWSER", "true")
            .env_remove("VISOR_FORCE_TTY")
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

const PIX: &str = "7b2d5e91-3c4a-4f68-9a0b-1c2d3e4f5a6b";
const START_URL: &str = "https://dashboard.visorfinance.app/pix/payments/continue?ref=SFMyNTY";
const END_TO_END_ID: &str = "E60746948202610221531abcdef123456";

fn pix(status: &str) -> Value {
    let pending = status == "pending";
    let completed = status == "completed";
    json!({"data": {
        "id": PIX, "status": status,
        "waiting_for": if pending { json!("payment_confirmation") } else { Value::Null },
        "amount": "20.00", "currency": "BRL", "pix_key_masked": "***.456.789-**",
        "recipient": if completed { json!("Maria da Silva") } else { Value::Null },
        "description": "Almoço", "bank": {"name": "Nubank", "linked": true},
        "start_url": if pending { json!(START_URL) } else { Value::Null },
        "end_to_end_id": if completed { json!(END_TO_END_ID) } else { Value::Null },
        "failure": if status == "failed" {
            json!({"code": "SALDO_INSUFICIENTE", "message": "Saldo insuficiente"})
        } else { Value::Null },
        "created_at": "2026-10-22T15:31:02Z",
        "paid_at": if completed { json!("2026-10-22T15:32:40Z") } else { Value::Null }
    }})
}

impl Harness {
    fn start_pix(&mut self, body: &Value) -> Mock {
        self.server
            .mock("POST", "/v1/pix-payments")
            .match_header("authorization", format!("Bearer {TOKEN}").as_str())
            .match_header("idempotency-key", Matcher::Regex("^[0-9a-f-]{36}$".into()))
            .match_body(Matcher::Json(body.clone()))
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(pix("pending").to_string())
            .create()
    }
}

#[test]
fn pix_shows_a_qr_code_and_waits_until_it_is_paid() {
    let mut harness = Harness::new();
    let start = harness.start_pix(&json!({
        "pix_key": "123.456.789-09", "amount": "20.00", "description": "Almoço", "bank": "Nubank"
    }));
    let poll = harness.get(&format!("/v1/pix-payments/{PIX}"), &pix("completed"));

    harness
        .visor()
        .args([
            "pix",
            "123.456.789-09",
            "20,00",
            "-d",
            "Almoço",
            "--bank",
            "Nubank",
            "--qr",
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "Pix of R$ 20,00 to ***.456.789-** from Nubank",
        ))
        .stderr(predicate::str::contains(
            "Scan this code with your phone's camera",
        ))
        .stderr(predicate::str::contains("█"))
        .stderr(predicate::str::contains(START_URL))
        .stderr(predicate::str::contains("Pix completed"))
        .stdout(predicate::str::contains(END_TO_END_ID))
        .stdout(predicate::str::contains("Maria da Silva"));
    start.assert();
    poll.assert();
}

#[test]
fn pix_without_a_terminal_prints_the_link_and_fails_when_the_pix_fails() {
    let mut harness = Harness::new();
    harness.start_pix(&json!({"pix_key": "maria@example.com", "amount": "20.00"}));
    harness.get(&format!("/v1/pix-payments/{PIX}"), &pix("failed"));

    harness
        .visor()
        .args(["pix", "maria@example.com", "20"])
        .assert()
        .code(5)
        .stderr(predicate::str::contains("Open this page to continue the Pix"))
        .stderr(predicate::str::contains(START_URL))
        .stderr(predicate::str::contains(
            "the Pix was not paid: Saldo insuficiente",
        ))
        .stdout(predicate::str::contains("failed"));
}

#[test]
fn pix_asks_where_to_confirm_on_a_terminal() {
    let mut harness = Harness::new();
    harness.start_pix(&json!({"pix_key": "maria@example.com", "amount": "20.00"}));
    harness.get(&format!("/v1/pix-payments/{PIX}"), &pix("completed"));

    // The phone is the default answer; the bank is linked, so the CLI says where.
    harness
        .visor()
        .args(["pix", "maria@example.com", "20"])
        .env("VISOR_FORCE_TTY", "1")
        .write_stdin("\n")
        .assert()
        .success()
        .stderr(predicate::str::contains("Where do you want to confirm the Pix?"))
        .stderr(predicate::str::contains("1 On your phone"))
        .stderr(predicate::str::contains(
            "Confirm on the device where you linked Nubank",
        ))
        .stderr(predicate::str::contains("Scan this code"));
}

#[test]
fn pix_no_wait_json_prints_the_started_pix() {
    let mut harness = Harness::new();
    harness.start_pix(&json!({"pix_key": "+5511987654321", "amount": "20.00", "key_type": "phone"}));
    let poll = harness
        .server
        .mock("GET", format!("/v1/pix-payments/{PIX}").as_str())
        .expect(0)
        .create();

    let output = harness
        .visor()
        .args([
            "pix",
            "+5511987654321",
            "20",
            "--key-type",
            "phone",
            "--no-wait",
            "--json",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let body: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(body["data"]["id"], PIX);
    assert_eq!(body["data"]["start_url"], START_URL);
    poll.assert();
}

#[test]
fn pix_status_reads_the_pix_once() {
    let mut harness = Harness::new();
    harness.get(&format!("/v1/pix-payments/{PIX}"), &pix("pending"));

    harness
        .visor()
        .args(["pix", "status", PIX])
        .assert()
        .success()
        .stdout(predicate::str::contains("pending"))
        .stdout(predicate::str::contains("R$ 20,00"))
        .stdout(predicate::str::contains(START_URL));
}

#[test]
fn pix_bank_required_lists_the_banks() {
    let mut harness = Harness::new();
    harness
        .server
        .mock("POST", "/v1/pix-payments")
        .with_status(422)
        .with_body(
            json!({"errors": [{"code": "BANK_REQUIRED", "message": "Send the bank.",
                               "details": {"banks": ["Nubank", "Itaú"]}}]})
            .to_string(),
        )
        .create();

    harness
        .visor()
        .args(["pix", "maria@example.com", "20"])
        .assert()
        .code(5)
        .stderr(predicate::str::contains(r#"--bank, such as "Nubank", "Itaú""#));
}

#[test]
fn pix_without_pix_access_and_no_terminal_says_what_to_run() {
    let mut harness = Harness::new();
    harness
        .server
        .mock("POST", "/v1/pix-payments")
        .with_status(403)
        .with_body(
            json!({"errors": [{"code": "INSUFFICIENT_SCOPE", "message": "Missing scope.",
                               "details": {"required_scope": "pix:payments:write"}}]})
            .to_string(),
        )
        .create();

    harness
        .visor()
        .args(["pix", "maria@example.com", "20"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("Run `visor pix` in a terminal"))
        .stderr(predicate::str::contains("/oauth/authorize").not());
}

#[test]
fn pix_rejects_a_negative_amount_before_calling_the_api() {
    Harness::new()
        .visor()
        .args(["pix", "maria@example.com", "-20"])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("a Pix amount is positive"));
}

#[test]
fn pix_asks_for_the_bank_on_a_terminal_and_retries_with_the_same_key() {
    let mut harness = Harness::new();
    let missing_bank = harness
        .server
        .mock("POST", "/v1/pix-payments")
        .match_header("idempotency-key", "pix-1")
        .match_body(Matcher::Json(
            json!({"pix_key": "maria@example.com", "amount": "20.00"}),
        ))
        .with_status(422)
        .with_body(
            json!({"errors": [{"code": "BANK_REQUIRED", "message": "Send the bank.",
                               "details": {"banks": ["Itaú", "Nubank"]}}]})
            .to_string(),
        )
        .create();
    let started = harness
        .server
        .mock("POST", "/v1/pix-payments")
        .match_header("idempotency-key", "pix-1")
        .match_body(Matcher::Json(
            json!({"pix_key": "maria@example.com", "amount": "20.00", "bank": "Nubank"}),
        ))
        .with_status(201)
        .with_body(pix("pending").to_string())
        .create();
    harness.get(&format!("/v1/pix-payments/{PIX}"), &pix("completed"));

    // 2 picks Nubank; Enter then confirms on this computer.
    harness
        .visor()
        .args(["pix", "maria@example.com", "20", "--idempotency-key", "pix-1"])
        .env("VISOR_FORCE_TTY", "1")
        .write_stdin("2\n\n")
        .assert()
        .success()
        .stderr(predicate::str::contains("Which bank does the Pix come from?"))
        .stderr(predicate::str::contains("1 Itaú"))
        .stderr(predicate::str::contains("2 Nubank"))
        .stderr(predicate::str::contains("Pix completed"));
    missing_bank.assert();
    started.assert();
}

#[test]
fn pix_asks_whether_an_ambiguous_key_is_a_cpf_or_a_phone() {
    let mut harness = Harness::new();
    harness
        .server
        .mock("POST", "/v1/pix-payments")
        .match_body(Matcher::Json(
            json!({"pix_key": "11987654321", "amount": "20.00"}),
        ))
        .with_status(422)
        .with_body(
            json!({"errors": [{"code": "AMBIGUOUS_PIX_KEY", "message": "CPF or phone.", "details": {}}]})
                .to_string(),
        )
        .create();
    let started = harness
        .server
        .mock("POST", "/v1/pix-payments")
        .match_body(Matcher::Json(
            json!({"pix_key": "11987654321", "amount": "20.00", "key_type": "phone"}),
        ))
        .with_status(201)
        .with_body(pix("pending").to_string())
        .create();

    harness
        .visor()
        .args(["pix", "11987654321", "20", "--no-wait"])
        .env("VISOR_FORCE_TTY", "1")
        .write_stdin("2\n")
        .assert()
        .success()
        .stderr(predicate::str::contains(
            "Is 11987654321 a CPF or a phone number?",
        ))
        .stdout(predicate::str::contains(START_URL));
    started.assert();
}

#[test]
fn pix_link_device_asks_the_api_to_link_the_bank_again() {
    let mut harness = Harness::new();
    let start =
        harness.start_pix(&json!({"pix_key": "maria@example.com", "amount": "20.00", "link_device": true}));

    harness
        .visor()
        .args(["pix", "maria@example.com", "20", "--link-device", "--no-wait"])
        .assert()
        .success();
    start.assert();
}

/// A release endpoint on the harness server that reports `tag`.
fn latest_release(harness: &mut Harness, tag: &str) -> String {
    harness
        .server
        .mock("GET", "/releases/latest")
        .with_body(json!({ "tag_name": tag }).to_string())
        .create();
    format!("{}/releases/latest", harness.server.url())
}

/// A copy of the test binary at `relative` under `root`, so it looks
/// installed by Homebrew or the installer.
#[cfg(unix)]
fn installed_copy(root: &std::path::Path, relative: &str) -> std::path::PathBuf {
    let exe = root.join(relative);
    std::fs::create_dir_all(exe.parent().unwrap()).unwrap();
    std::fs::copy(assert_cmd::cargo::cargo_bin!("visor"), &exe).unwrap();
    exe
}

/// Shell lines for a stub `brew` that links a new `<prefix>/bin/visor`
/// reporting `version`, and running any other command as `ran: <args>`
/// with exit code 3.
#[cfg(unix)]
fn link_new_visor(prefix: &std::path::Path, version: &str) -> String {
    let visor = prefix.join("bin/visor");
    format!(
        "printf '#!/bin/sh\\nif [ \"$1\" = --version ]; then echo visor {version}; else echo \"ran: $*\"; exit 3; fi\\n' > '{0}'\nchmod +x '{0}'\n",
        visor.display()
    )
}

#[cfg(unix)]
fn executable(path: &std::path::Path, script: &str) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, script).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn update_check_says_when_this_is_the_latest_version() {
    let mut harness = Harness::new();
    let releases = latest_release(&mut harness, &format!("v{}", env!("CARGO_PKG_VERSION")));

    harness
        .visor()
        .args(["update", "--check"])
        .env("VISOR_RELEASES_URL", releases)
        .assert()
        .success()
        .stdout(predicate::str::contains("is the latest version"));
}

#[test]
fn update_check_exits_1_with_json_when_a_new_version_is_out() {
    let mut harness = Harness::new();
    let releases = latest_release(&mut harness, "v99.0.0");

    let output = harness
        .visor()
        .args(["update", "--check", "--json"])
        .env("VISOR_RELEASES_URL", releases)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("UPDATE_AVAILABLE"))
        .get_output()
        .stdout
        .clone();

    let body: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(body["latest"], "99.0.0");
    assert_eq!(body["update_available"], true);
    assert_eq!(body["updated"], false);
    assert_eq!(body["installed_with"], "unknown");
}

#[test]
fn update_refuses_a_binary_it_did_not_install() {
    let mut harness = Harness::new();
    let releases = latest_release(&mut harness, "v99.0.0");

    harness
        .visor()
        .arg("update")
        .env("VISOR_RELEASES_URL", releases)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("cannot replace it"))
        .stderr(predicate::str::contains("visor-cli#install"));
}

#[test]
fn update_fails_with_6_when_github_does_not_answer() {
    let harness = Harness::new();

    harness
        .visor()
        .arg("update")
        .env("VISOR_RELEASES_URL", "http://127.0.0.1:9/unreachable")
        .assert()
        .code(6)
        .stderr(predicate::str::contains("latest version from GitHub"));
}

#[cfg(unix)]
#[test]
fn update_runs_brew_upgrade_for_a_homebrew_install() {
    let mut harness = Harness::new();
    let releases = latest_release(&mut harness, "v99.0.0");
    let prefix = TempDir::new().unwrap();
    let exe = installed_copy(prefix.path(), "Cellar/visor/0.5.0/bin/visor");
    let log = prefix.path().join("brew.log");
    executable(
        &prefix.path().join("bin/brew"),
        &format!(
            "#!/bin/sh\necho \"$@\" > '{}'\necho brewing\n{}",
            log.display(),
            link_new_visor(prefix.path(), "99.0.0")
        ),
    );

    let output = Command::new(&exe)
        .args(["update", "--json"])
        .env("VISOR_CONFIG_DIR", harness.config.path())
        .env("VISOR_RELEASES_URL", releases)
        .assert()
        .success()
        .stderr(predicate::str::contains("brewing"))
        .get_output()
        .stdout
        .clone();

    let body: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(body["installed_with"], "homebrew");
    assert_eq!(body["updated"], true);
    assert_eq!(
        std::fs::read_to_string(log).unwrap().trim(),
        "upgrade visor-finance/tap/visor"
    );
}

#[cfg(unix)]
#[test]
fn update_reports_a_failed_brew_upgrade() {
    let mut harness = Harness::new();
    let releases = latest_release(&mut harness, "v99.0.0");
    let prefix = TempDir::new().unwrap();
    let exe = installed_copy(prefix.path(), "Cellar/visor/0.5.0/bin/visor");
    executable(
        &prefix.path().join("bin/brew"),
        "#!/bin/sh\necho 'Error: Your Command Line Tools are too outdated.' >&2\nexit 1\n",
    );

    Command::new(&exe)
        .arg("update")
        .env("VISOR_CONFIG_DIR", harness.config.path())
        .env("VISOR_RELEASES_URL", releases)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Command Line Tools are too outdated"))
        .stderr(predicate::str::contains("visor was not updated"));
}

#[cfg(unix)]
#[test]
fn update_runs_the_release_installer_into_the_same_directory() {
    let mut harness = Harness::new();
    let releases = latest_release(&mut harness, "v99.0.0");
    let installer = harness
        .server
        .mock("GET", "/download/v99.0.0/visor-installer.sh")
        // Like dist's installer: write elsewhere, then `mv` over the binary
        // (Linux refuses to write into a running executable).
        .with_body(
            "set -e\nprintf '#!/bin/sh\\necho visor 99.0.0\\n' > \"$VISOR_UNMANAGED_INSTALL/visor.tmp\"\nchmod +x \"$VISOR_UNMANAGED_INSTALL/visor.tmp\"\nmv \"$VISOR_UNMANAGED_INSTALL/visor.tmp\" \"$VISOR_UNMANAGED_INSTALL/visor\"\necho \"path $VISOR_NO_MODIFY_PATH\"\n",
        )
        .create();
    let cargo_home = TempDir::new().unwrap();
    let exe = installed_copy(cargo_home.path(), "bin/visor");

    Command::new(&exe)
        .arg("update")
        .env("VISOR_CONFIG_DIR", harness.config.path())
        .env("VISOR_RELEASES_URL", releases)
        .env(
            "VISOR_RELEASE_DOWNLOAD_URL",
            format!("{}/download", harness.server.url()),
        )
        .env("CARGO_HOME", cargo_home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated visor to 99.0.0"))
        .stdout(predicate::str::contains("path 1"));

    installer.assert();
    assert!(std::fs::read_to_string(&exe).unwrap().contains("visor 99.0.0"));
}

#[cfg(unix)]
#[test]
fn update_prints_the_cargo_command_for_a_cargo_install() {
    let mut harness = Harness::new();
    let releases = latest_release(&mut harness, "v99.0.0");
    let cargo_home = TempDir::new().unwrap();
    let exe = installed_copy(cargo_home.path(), "bin/visor");
    std::fs::write(
        cargo_home.path().join(".crates2.json"),
        r#"{"installs":{"visor 0.5.0 (git+https://github.com/Visor-Finance/visor-cli#677cfbb)":{}}}"#,
    )
    .unwrap();

    Command::new(&exe)
        .arg("update")
        .env("VISOR_CONFIG_DIR", harness.config.path())
        .env("VISOR_RELEASES_URL", releases)
        .env("CARGO_HOME", cargo_home.path())
        .assert()
        .code(1)
        .stderr(predicate::str::contains("cargo install --git"));
}

#[cfg(unix)]
#[test]
fn update_fails_when_brew_installs_nothing_new() {
    let mut harness = Harness::new();
    let releases = latest_release(&mut harness, "v99.0.0");
    let prefix = TempDir::new().unwrap();
    let exe = installed_copy(prefix.path(), "Cellar/visor/0.5.0/bin/visor");
    executable(
        &prefix.path().join("bin/brew"),
        &format!("#!/bin/sh\n{}", link_new_visor(prefix.path(), "0.5.0")),
    );

    Command::new(&exe)
        .arg("update")
        .env("VISOR_CONFIG_DIR", harness.config.path())
        .env("VISOR_RELEASES_URL", releases)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("visor 99.0.0 was not installed"))
        .stderr(predicate::str::contains("Homebrew may not have it yet"));
}

/// A Homebrew install of this build whose last release check found
/// `latest`, with automatic updates forced on as on a terminal.
#[cfg(unix)]
struct AutoUpdate {
    prefix: TempDir,
    config: TempDir,
    exe: std::path::PathBuf,
}

#[cfg(unix)]
impl AutoUpdate {
    fn new(latest: &str, brew_script: &str) -> Self {
        let prefix = TempDir::new().unwrap();
        let config = TempDir::new().unwrap();
        let exe = installed_copy(prefix.path(), "Cellar/visor/0.5.0/bin/visor");
        let script = brew_script
            .replace("LOG", &prefix.path().join("brew.log").display().to_string())
            .replace("LINK", &link_new_visor(prefix.path(), latest));
        executable(&prefix.path().join("bin/brew"), &format!("#!/bin/sh\n{script}"));
        std::fs::write(
            config.path().join("update-check.json"),
            json!({ "checked_at": chrono_now(), "latest": latest }).to_string(),
        )
        .unwrap();
        Self { prefix, config, exe }
    }

    fn visor(&self) -> Command {
        let mut cmd = Command::new(&self.exe);
        cmd.env("VISOR_CONFIG_DIR", self.config.path())
            .env("VISOR_UPDATE_CHECK", "1")
            .env("VISOR_RELEASES_URL", "http://127.0.0.1:9/unreachable")
            .env_remove("VISOR_NO_UPDATE_NOTIFIER")
            .env_remove("CI");
        cmd
    }

    fn brew_log(&self) -> Option<String> {
        std::fs::read_to_string(self.prefix.path().join("brew.log")).ok()
    }
}

#[cfg(unix)]
fn chrono_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .try_into()
        .unwrap()
}

/// This build's version with `patch` added to its patch number.
#[cfg(unix)]
fn patch_ahead(patch: u64) -> String {
    let mut parts: Vec<u64> = env!("CARGO_PKG_VERSION")
        .split('.')
        .map(|part| part.parse().unwrap())
        .collect();
    parts[2] += patch;
    format!("{}.{}.{}", parts[0], parts[1], parts[2])
}

#[cfg(unix)]
#[test]
fn a_new_minor_version_is_installed_before_the_command_which_runs_in_it() {
    let auto = AutoUpdate::new("99.0.0", "echo \"$@\" > 'LOG'\nLINK");

    auto.visor()
        .args(["open", "--print"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("Automatically updating visor"))
        .stdout(predicate::str::contains("ran: open --print"));

    assert_eq!(auto.brew_log().unwrap().trim(), "upgrade visor-finance/tap/visor");
}

#[cfg(unix)]
#[test]
fn a_failed_automatic_update_runs_the_command_anyway_and_waits_an_hour() {
    let auto = AutoUpdate::new("99.0.0", "echo \"$@\" >> 'LOG'\nexit 1");

    auto.visor()
        .args(["open", "--print"])
        .assert()
        .success()
        .stderr(predicate::str::contains("The automatic update failed"))
        .stdout(predicate::str::contains("https://dashboard.visorfinance.app"));

    // Within the hour there is no second attempt, before or after the command.
    auto.visor()
        .args(["open", "--print"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Automatically updating").not());
    assert_eq!(auto.brew_log().unwrap().lines().count(), 1);
}

#[cfg(unix)]
#[test]
fn automatic_updates_turned_off_only_show_the_notice() {
    let auto = AutoUpdate::new("99.0.0", "echo \"$@\" > 'LOG'\nLINK");

    auto.visor()
        .args(["settings", "autoupdate", "disable"])
        .assert()
        .success()
        .stdout("Automatic updates: disabled\n");
    auto.visor()
        .args(["open", "--print"])
        .assert()
        .success()
        .stderr(predicate::str::contains("99.0.0 is out"));

    assert!(auto.brew_log().is_none());
}

#[cfg(unix)]
#[test]
fn a_new_patch_is_installed_in_the_background_after_the_command() {
    let latest = patch_ahead(1);
    let auto = AutoUpdate::new(&latest, "echo \"$@\" > 'LOG'\nLINK");

    auto.visor()
        .args(["open", "--print"])
        .assert()
        .success()
        .stdout(predicate::str::contains("https://dashboard.visorfinance.app"))
        .stderr(predicate::str::contains("is out").not())
        .stderr(predicate::str::contains("Automatically updating").not());

    // The background `visor update` checks GitHub itself; here it cannot
    // reach it, so it stops before brew and says so in update.log.
    let log = auto.config.path().join("update.log");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let text = loop {
        let text = std::fs::read_to_string(&log).unwrap_or_default();
        if text.contains("error") || std::time::Instant::now() > deadline {
            break text;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    };
    assert!(text.contains("latest version from GitHub"), "{text}");
}

#[cfg(unix)]
#[test]
fn json_output_updates_silently() {
    let auto = AutoUpdate::new("99.0.0", "echo \"$@\" > 'LOG'\necho brewing\nLINK");

    auto.visor()
        .args(["open", "--print", "--json"])
        .assert()
        .code(3)
        .stderr(predicate::str::contains("Automatically updating").not())
        .stderr(predicate::str::contains("brewing").not())
        .stdout(predicate::str::contains("ran: open --print --json"));
}

#[test]
fn settings_autoupdate_is_on_by_default_and_saved_in_the_config() {
    let harness = Harness::new();

    harness
        .visor()
        .args(["settings", "autoupdate"])
        .assert()
        .success()
        .stdout("Automatic updates: enabled\n");
    harness
        .visor()
        .args(["settings", "autoupdate", "disable", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""auto_update": false"#));

    let config = std::fs::read_to_string(harness.config.path().join("config.toml")).unwrap();
    assert!(config.contains("auto_update = false"), "{config}");
}
