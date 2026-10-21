# visor-cli

Rust CLI for the Visor personal API (`https://api.visorfinance.app/v1`). The API contract is `openapi/openapi.json`, copied from the backend (`Visor-Finance/visor`, `developer-docs/openapi.json`) by `scripts/sync-openapi.sh`. Never edit it by hand.

## Rules

- A new API operation goes in `api::ENDPOINTS`, a model in `src/models.rs` (if a table shows it), a command in `src/cli.rs` + `src/commands/`, and an end-to-end test in `tests/cli.rs`. `tests/contract.rs` fails until `ENDPOINTS` matches the spec.
- `--json` prints the API body unchanged. Data goes to stdout; notes and errors go to stderr.
- Models keep enum-like fields as `String` and decimals as strings; never parse money as `f64`.
- Never print or log a full token; use `credentials::redact`.
- Exit codes in `error::exit` are a public contract; document any change in the README.
- Tests never touch the real keychain or config: use `VISOR_CONFIG_DIR` + `VISOR_CREDENTIAL_STORE=file`.

## Check

```bash
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

## Release

Use the `release` skill (`.claude/skills/release/SKILL.md`). Add user-facing changes under `## Unreleased` in `CHANGELOG.md` as you make them.
