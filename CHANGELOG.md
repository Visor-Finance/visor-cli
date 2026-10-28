# Changelog

Each `## <version>` section becomes the notes of its GitHub release. Write for
people who use the CLI: what they can do now, what changed, what to watch for.

## Unreleased

## 0.6.0 - 2026-10-28

- `visor update` updates the CLI the way it was installed: it runs `brew upgrade visor-finance/tap/visor` for Homebrew, or the new release's installer into the same directory for the shell and PowerShell installers. A `cargo install` build prints the `cargo install` command to run. If Homebrew or the installer fails, its own error stays on screen and visor is left as it was.
- `visor update --check` compares this version with the latest release and exits with code 1 when a new one is out; with `--json` it prints `current`, `latest`, `update_available`, and `installed_with`.
- The new-version notice and `visor doctor` now say to run `visor update`.

## 0.5.0 - 2026-10-28

- `visor pix <key> <amount>` sends a Pix that you confirm with biometrics at your bank (Premium). It asks where you confirm: on this computer (it opens the page) or on your phone (it shows a QR code). Then it waits until the Pix is paid and prints its Pix ID. `--qr` and `--open` skip the question, `--no-wait` prints the Pix and its link and exits, and `--bank`, `--key-type`, and `-d` fill in the rest.
- `visor pix status <id>` shows a Pix you started.
- On a terminal, when Visor needs a choice, `visor pix` asks for it and goes on: which of your banks the Pix comes from, or whether a key is a CPF or a phone. In a script, the error lists the options for `--bank` or `--key-type`.
- The first `visor pix` opens the browser so you can allow Pix, then swaps the stored login for the new one and ends the old one. You do not log in again.
- A Pix that fails, is canceled, or expires exits with code 5 and says why. Pix errors say what to do next: the banks you can pass to `--bank`, `--key-type cpf|phone` for a key that can be either, and the monthly limit with its renewal date.
- `visor pix --link-device` links the bank again on the device you confirm with. A bank link (its passkey) lives on the device that made it, so another device showed "Dispositivo não reconhecido". The device question now starts on the phone and says where the bank is linked.
- Questions on a terminal use a picker: ↑ ↓ to choose, Enter to confirm, Esc to cancel, with a line on what each option does. Without a real terminal, the CLI asks for the number instead.
- `BROWSER` picks the command that opens URLs, and `VISOR_FORCE_TTY=1` lets a wrapper answer prompts.

## 0.4.0 - 2026-10-21

- **Changed:** `--limit` on `tx list` and `notifications list` is now the most results to return (default 50), not the page size. The CLI fetches as many pages as it needs, like `gh`'s `--limit`. Before, a list returned one page and `--all` was the only way past it.
- `--all` and `--limit` cannot be combined. A list that spans pages shows a running count on the terminal.
- A loader shows while the CLI waits for Visor, on a terminal only, after 150 ms so fast commands never flicker. A multi-page list shows its running count.
- When the API rate limit is reached, the CLI waits the whole `retry-after` (up to a minute) and says so, instead of failing a long `--all` export after 30 s.
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
