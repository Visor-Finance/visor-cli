# visor

The command-line interface for the [Visor personal API](https://docs.visorfinance.app): spaces, accounts, transactions, cards, goals, and more, from your terminal or your scripts.

The personal API requires an active **Premium** plan.

## Install

```bash
# macOS and Linux
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Visor-Finance/visor-cli/releases/latest/download/visor-installer.sh | sh

# Homebrew
brew install Visor-Finance/tap/visor

# Windows (PowerShell)
powershell -ExecutionPolicy Bypass -c "irm https://github.com/Visor-Finance/visor-cli/releases/latest/download/visor-installer.ps1 | iex"

# From source
cargo install --git https://github.com/Visor-Finance/visor-cli
```

Shell completions: `visor completions zsh > "${fpath[1]}/_visor"` (also `bash`, `fish`, `powershell`, `elvish`). Release archives also include them, with a man page, under `completions/` and `man/`.

Verify a downloaded binary: `gh attestation verify visor-aarch64-apple-darwin.tar.xz --repo Visor-Finance/visor-cli`.

## Log in

```bash
visor auth login
```

The CLI opens Visor in your browser. Log in, choose **Read only** or **Read and record transactions** (needed for `tx add`, `tx edit`, and `tx rm`), and approve. The CLI stores the login in the system keychain and refreshes it as it expires. It appears as **Visor CLI** in [Settings > Connected apps](https://dashboard.visorfinance.app/settings/connected_apps); revoke it there to end it.

```bash
visor auth login --no-browser            # print the login page URL instead of opening it
visor auth login --with-token < token    # a personal API token, for machines without a browser
visor auth status                        # who you are and how you are logged in (also: visor whoami)
visor doctor                             # check the CLI version, the login, the API, and the space
visor auth logout                        # removes the stored login
```

On a server or in CI, create a personal API token in Settings and pass it with `--with-token` or `VISOR_TOKEN`. `VISOR_TOKEN` overrides the stored login. On a machine without a keychain (a headless Linux server, for example), set `VISOR_CREDENTIAL_STORE=file` to keep the login in `credentials.json` in the config directory, readable only by you.

## Use

```bash
visor spaces list
visor spaces use Pessoal                 # the space other commands act on

visor overview                           # balances, this month, projection
visor accounts list
visor categories list
visor tx list --from 2026-10-01 --category Mercado
visor tx get 3f2b1c4e-…

visor tx add --amount 18,90 --description Padaria --category Mercado --account Nubank
visor tx edit 3f2b1c4e-… --category Restaurantes --tag viagem
visor tx rm 3f2b1c4e-…                   # asks first; --yes to skip

visor cards list
visor cards bills <card-id> --from 2026-01-01 --to 2026-12-31
visor subscriptions list
visor fixed-bills list
visor installments list --status active
visor goals list
visor investments list
visor connections list
visor notifications list --unread
```

- Accounts, categories, tags, and spaces accept a **name or an ID**. A system category also answers to its key, such as `groceries`.
- `--space` (or `VISOR_SPACE`) picks a space for one command. Without it, commands use the space from `visor spaces use`, then your default personal space.
- Amounts are positive. `--type debit` (the default) is money out, `--type credit` is money in. `18,90` and `18.90` both work.
- `tx add` sends an `Idempotency-Key` and prints it. Pass the same `--idempotency-key` to retry safely: Visor records the transaction once.

## Scripting

`--json` prints the API response body unchanged, so the [API reference](https://docs.visorfinance.app) documents every field. Tables, notes, and errors never mix with it: data goes to stdout, everything else to stderr.

```bash
# Export a year of transactions
visor tx list --from 2026-01-01 --to 2026-12-31 --all --json | jq -c '.data[]' > 2026.jsonl

# Spending per category this month
visor tx list --from 2026-10-01 --type debit --all --json \
  | jq -r '.data | group_by(.category_id)[] | [.[0].category_id, (map(.amount | tonumber) | add)] | @tsv'
```

Lists return one page; `--all` follows every cursor, and `--after <cursor>` continues from the cursor a page prints. The CLI retries `429` (after `retry-after`) and `5xx` responses with backoff. The API allows 60 requests per minute per token.

Errors follow `--json` too: `{"error": {"code": "NOT_FOUND", "message": "...", "status": 404, "request_id": "...", "exit_code": 4}}` on standard error.

```bash
# A spreadsheet of this month
visor tx list --from 2026-10-01 --all --csv > outubro.csv

# Every HTTP request, for a support ticket (never prints the token)
visor tx list --debug
```

### Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Other failure |
| 2 | Invalid usage, such as a bad flag or an unknown category name |
| 3 | Not logged in, invalid or expired token, missing scope, or no Premium plan |
| 4 | Not found |
| 5 | Request rejected: validation, conflict, or idempotency conflict |
| 6 | Rate limited, network error, or Visor unavailable |

### Environment

| Variable | Effect |
|---|---|
| `VISOR_TOKEN` | Token to use instead of the stored one |
| `VISOR_SPACE` | Space name or ID, like `--space` |
| `VISOR_CREDENTIAL_STORE` | `keyring` (default) or `file` |
| `VISOR_CONFIG_DIR` | Config directory. Default: `$XDG_CONFIG_HOME/visor`, `~/.config/visor`, or `%APPDATA%\visor` |
| `NO_COLOR` | Disables color |
| `VISOR_DEBUG` | `1` logs each HTTP request, like `--debug` |
| `VISOR_NO_UPDATE_NOTIFIER` | `1` turns off the new-version notice (it is already off in CI and in pipes) |

## Development

```bash
cargo test                          # unit, end-to-end (mock API), and contract tests
cargo clippy --all-targets -- -D warnings
cargo fmt
```

- `src/client.rs` handles HTTP: auth header, retries, and the error envelope. `src/api.rs` lists every endpoint the CLI calls; `src/commands/` holds one handler per command.
- `tests/cli.rs` runs the real binary against a [mockito](https://docs.rs/mockito) server, with `VISOR_CONFIG_DIR` in a temp dir and the file credential store, so tests never touch your keychain or config.
- `tests/contract.rs` checks the CLI against `openapi/openapi.json`. It fails when the API gains or drops an operation, or when a model no longer decodes the schema's minimal object.

### Keeping up with the API

The backend generates `developer-docs/openapi.json` in [Visor-Finance/visor](https://github.com/Visor-Finance/visor). This repo keeps a copy:

```bash
scripts/sync-openapi.sh             # from ../visor
scripts/sync-openapi.sh --remote    # from GitHub main
cargo test --test contract          # what the CLI must follow
```

The `OpenAPI drift` workflow runs on weekdays and fails when the copy is stale. It needs a `VISOR_REPO_TOKEN` secret with read access to the backend repo's contents.

### Releasing

[dist](https://opensource.axo.dev/cargo-dist/) builds and publishes releases (`dist-workspace.toml`, `.github/workflows/release.yml`):

```bash
# bump `version` in Cargo.toml, commit, then
git tag v0.2.0 && git push --tags
```

The tag push builds macOS, Linux, and Windows binaries with shell, PowerShell, and Homebrew installers. Publishing the formula needs the `Visor-Finance/homebrew-tap` repository and a `HOMEBREW_TAP_TOKEN` secret.
