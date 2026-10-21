# Changelog

Each `## <version>` section becomes the notes of its GitHub release. Write for
people who use the CLI: what they can do now, what changed, what to watch for.

## Unreleased

- `visor auth login` now logs in through the browser: approve the login in Visor and choose its access. The login refreshes itself and appears as **Visor CLI** in Settings > Connected apps.
- `visor auth login --no-browser` prints the login page URL instead of opening a browser.
- **Changed:** `visor auth login` without options no longer prompts for a personal API token. Use `visor auth login --with-token`, which reads it from standard input or a hidden prompt. Stored tokens, `--with-token < file`, and `VISOR_TOKEN` work as before.

## 0.1.0 - 2026-10-21

First release.

- `visor auth login | logout | status` with personal API tokens stored in the system keychain, or in a file with `VISOR_CREDENTIAL_STORE=file`. `VISOR_TOKEN` overrides both.
- Every operation of the personal API: spaces, accounts, categories, tags, transactions (list, get, add, edit, delete), cards and bills, subscriptions, fixed bills, installments, goals, investments, connections, overview, and notifications.
- Accounts, categories, tags, and spaces accept a name or an ID.
- `--json` prints the API response body unchanged; `--all` follows every page.
- Documented exit codes; automatic retries for rate limits and server errors.
- Shell completions with `visor completions <shell>`.
