# Changelog

Each `## <version>` section becomes the notes of its GitHub release. Write for
people who use the CLI: what they can do now, what changed, what to watch for.

## Unreleased

- The CLI works on the Pro plan too, like the personal API. The `PLAN_REQUIRED` hint and the help now say Pro or Premium.
- The man page in the release archives shows the right version (0.3.0 shipped it with 0.2.0 in its header).

## 0.3.0 - 2026-10-21

- `--json` errors are JSON too: `{"error": {"code", "message", "exit_code", ...}}` on standard error, with the HTTP status, details, and request ID for an API error.
- `--debug` (or `VISOR_DEBUG=1`) prints each HTTP request on standard error: method, URL, status, time, and request ID, never the token.
- `visor tx list --csv` exports transactions for a spreadsheet, with names instead of IDs and signed amounts. Text that a spreadsheet would run as a formula is escaped.
- `visor open [page]` opens the dashboard, a page of it, the docs, or the releases in the browser; `--print` prints the URL.
- The CLI tells you once a day, at most, when a new version is out. It never slows a command, and it stays quiet in a pipe, in CI, or with `VISOR_NO_UPDATE_NOTIFIER=1`.
- Release archives include shell completions (bash, zsh, fish) and a man page, and every artifact has a GitHub attestation: `gh attestation verify <file> --repo Visor-Finance/visor-cli`.

## 0.2.0 - 2026-10-21

- `visor auth login` now logs in through the browser: approve the login in Visor and choose its access. The login refreshes itself and appears as **Visor CLI** in Settings > Connected apps.
- `visor auth login --no-browser` prints the login page URL instead of opening a browser.
- After a login, the CLI shows the account (plan, access, login method, where the login is stored, and the space commands act on) and what to try next.
- `visor whoami` and `visor auth status` show the same account summary; `--json` returns it with stable keys and no secrets.
- `visor doctor` checks the CLI version against the latest release, the login, the Visor API, and the space; it exits with 1 when a check fails.
- `visor` without a command shows the Visor banner and the help. The banner appears only on a color terminal and never in a pipe or under `NO_COLOR`.
- **Changed:** `visor auth status --json` returns the account summary instead of the `/v1/me` body. Use `visor me --json` for the body.
- **Changed:** `visor auth login` without options no longer prompts for a personal API token. Use `visor auth login --with-token`, which reads it from standard input or a hidden prompt. Stored tokens, `--with-token < file`, and `VISOR_TOKEN` work as before.

## 0.1.0 - 2026-10-21

First release.

- `visor auth login | logout | status` with personal API tokens stored in the system keychain, or in a file with `VISOR_CREDENTIAL_STORE=file`. `VISOR_TOKEN` overrides both.
- Every operation of the personal API: spaces, accounts, categories, tags, transactions (list, get, add, edit, delete), cards and bills, subscriptions, fixed bills, installments, goals, investments, connections, overview, and notifications.
- Accounts, categories, tags, and spaces accept a name or an ID.
- `--json` prints the API response body unchanged; `--all` follows every page.
- Documented exit codes; automatic retries for rate limits and server errors.
- Shell completions with `visor completions <shell>`.
