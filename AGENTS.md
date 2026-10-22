# visor-cli

Rust CLI for the Visor personal API (`https://api.visorfinance.app/v1`). The API contract is `openapi/openapi.json`, copied from the backend (`Visor-Finance/visor`, `developer-docs/openapi.json`) by `scripts/sync-openapi.sh`. Never edit it by hand.

## Rules

- A new API operation goes in `api::ENDPOINTS`, a model in `src/models.rs` (if a table shows it), a command in `src/cli.rs` + `src/commands/`, and an end-to-end test in `tests/cli.rs`. `tests/contract.rs` fails until `ENDPOINTS` matches the spec.
- `--json` prints the API body unchanged. Data goes to stdout; notes and errors go to stderr.
- Models keep enum-like fields as `String` and decimals as strings; never parse money as `f64`.
- Never print or log a full token; use `credentials::redact`.
- Exit codes in `error::exit` are a public contract; document any change in the README.
- Tests never touch the real keychain or config: use `VISOR_CONFIG_DIR` + `VISOR_CREDENTIAL_STORE=file`.

## CLI behavior checklist

Every new or changed command meets these, the way `gh`, Resend, and Fly do. Check them before you call a command done, and compare with those CLIs when you design new UX.

- **Bounded by default.** A list returns at most `--limit` results (default 50) and fetches pages itself; `--all` is the only way to fetch everything. Never fetch an unbounded collection by default.
- **Never silent.** Any wait for Visor shows the loader (`activity::start` / `activity::set`); a multi-page fetch shows its running count; a rate-limit wait says how long. Prompts and output stop the loader first (`output::line` and `output::note` already do).
- **Respect the rate limit.** The API allows 60 requests a minute per token in fixed one-minute windows. Send requests one at a time, never in parallel, and wait the full `retry-after` on `429` (`RetryPolicy`). Prefer one request with filters (`--from`, `--to`) over many.
- **Terminal-aware.** Color, the banner, spinners, and prompts appear only on a terminal and never under `NO_COLOR`; a pipe gets data only, on stdout.
- **Scriptable.** Every command with data supports `--json` (stable keys, no secrets), and errors follow `--json` too. A command never prompts without a terminal: it offers a flag instead (`--yes`, `--with-token`).
- **Safe.** Destructive actions confirm unless `--yes`. Creates send an `Idempotency-Key`. Retries never repeat a request the API may have processed (no `5xx` retry for `DELETE`).
- **Choices, not dead ends.** When the API answers with the options it needs (such as `details.banks` on `BANK_REQUIRED`, or CPF or phone on `AMBIGUOUS_PIX_KEY`), a terminal run lists them, asks the user to pick one (`commands::choose`), and retries the same request, with the same `Idempotency-Key`. Without a terminal, or with `--json`, it never asks: the error lists the options and the flag that picks one (`--bank`, `--key-type`). Show the options the API returns; never make the user rerun the command just to type one.
- **Actionable errors.** An error says what to run next (`visor auth login`, `--limit`, a Settings URL) and maps to a documented exit code.
- **Documented.** Every flag has a help line; non-obvious commands have examples in `after_help`; user-visible changes go under `## Unreleased` in `CHANGELOG.md`; regenerate `dist-assets/` (see below).

## Check

```bash
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

## Release

Use the `release` skill (`.claude/skills/release/SKILL.md`). Add user-facing changes under `## Unreleased` in `CHANGELOG.md` as you make them.

## Generated files

`dist-assets/` holds the shell completions and the man page that dist packs into each release. After changing a command or flag, run `VISOR_UPDATE_GENERATED=1 cargo test --test generated`; the test fails until they match.
