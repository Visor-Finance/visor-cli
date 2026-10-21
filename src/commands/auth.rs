use std::io::{self, IsTerminal, Read};

use super::{Context, client_for};
use crate::api;
use crate::cli::{AuthCommand, Global};
use crate::client::Client;
use crate::credentials::{self, Credential, Store};
use crate::error::{Error, Result, TOKENS_URL};
use crate::models::Me;
use crate::oauth;
use crate::output::{self, Format};

pub fn run(command: AuthCommand, store: &Store, global: &Global, format: Format) -> Result<()> {
    match command {
        AuthCommand::Login { with_token: true, .. } => login_with_token(store, &global.api_url, format),
        AuthCommand::Login { no_browser, .. } => {
            login_in_browser(store, &global.api_url, &global.auth_url, !no_browser, format)
        }
        AuthCommand::Logout => logout(store),
        AuthCommand::Status => status(store, global, format),
    }
}

fn login_in_browser(
    store: &Store,
    api_url: &str,
    auth_url: &str,
    open_browser: bool,
    format: Format,
) -> Result<()> {
    let credential = oauth::login(auth_url, open_browser)?;
    finish_login(store, api_url, &credential, format)
}

fn login_with_token(store: &Store, api_url: &str, format: Format) -> Result<()> {
    let token = read_token()?;
    if !token.starts_with("visor_pat_") {
        return Err(Error::usage(
            "that is not a Visor personal API token; tokens start with `visor_pat_`",
        ));
    }
    finish_login(store, api_url, &Credential::PersonalToken { token }, format)
}

/// Checks the credential with the API, then stores it, so a bad token
/// never replaces a good login.
fn finish_login(store: &Store, api_url: &str, credential: &Credential, format: Format) -> Result<()> {
    let body = Client::new(api_url, credential.bearer())?.me()?;
    let me: Me = api::data(body.clone())?;
    store.save(credential)?;

    if format == Format::Json {
        output::json(&body)?;
    }
    output::note(&format!(
        "Logged in as {} ({}) with {}. Stored in {store}.",
        me.email,
        me.plan,
        access(credential)
    ));
    if std::env::var_os("VISOR_TOKEN").is_some() {
        output::note("Note: VISOR_TOKEN is set and takes precedence over this login.");
    }
    Ok(())
}

/// What the credential lets the CLI do, as Settings names it.
fn access(credential: &Credential) -> String {
    match credential {
        Credential::PersonalToken { .. } => "a personal API token".to_owned(),
        Credential::OAuth { scope, .. } if scope.split(' ').any(|s| s == "pfm:transactions:write") => {
            "read and record transactions access".to_owned()
        }
        Credential::OAuth { .. } => "read-only access".to_owned(),
    }
}

fn read_token() -> Result<String> {
    let token = if io::stdin().is_terminal() {
        output::note(&format!(
            "Create a personal API token at {TOKENS_URL}\n\
             (Read only, or Read and record transactions), then paste it below."
        ));
        rpassword::prompt_password("Token: ").map_err(|error| Error::io("cannot read the token", error))?
    } else {
        let mut token = String::new();
        io::stdin()
            .read_to_string(&mut token)
            .map_err(|error| Error::io("cannot read the token from standard input", error))?;
        token
    };

    let token = token.trim().to_owned();
    if token.is_empty() {
        return Err(Error::usage("the token is empty"));
    }
    Ok(token)
}

fn logout(store: &Store) -> Result<()> {
    match store.load().ok().flatten() {
        Some(credential) => {
            store.delete()?;
            output::note(&format!("Removed the login from {store}."));
            let revoke = match credential {
                Credential::PersonalToken { .. } => "The token still works until you revoke it",
                Credential::OAuth { .. } => "To end the login on Visor too, revoke Visor CLI",
            };
            output::note(&format!("{revoke} at {TOKENS_URL}."));
        }
        // An unreadable credential is removed all the same.
        None if store.delete()? => output::note(&format!("Removed the login from {store}.")),
        None => output::note("No stored login to remove."),
    }
    if std::env::var_os("VISOR_TOKEN").is_some() {
        output::note("VISOR_TOKEN is still set in this environment.");
    }
    Ok(())
}

fn status(store: &Store, global: &Global, format: Format) -> Result<()> {
    let (credential, source) = credentials::resolve(store)?.ok_or(Error::NotLoggedIn)?;
    let (method, access) = match &credential {
        Credential::OAuth { .. } => ("browser login (Visor CLI)".to_owned(), access(&credential)),
        Credential::PersonalToken { token } => (
            format!("personal API token {}", credentials::redact(token)),
            "as set on the token in Settings".to_owned(),
        ),
    };
    let body = client_for(&global.api_url, &global.auth_url, credential, &source)?.me()?;

    if format == Format::Json {
        return output::json(&body);
    }
    let me: Me = api::data(body)?;
    output::print_fields(&[
        ("Logged in as", format!("{} <{}>", me.name, me.email)),
        ("Plan", me.plan),
        ("Login", method),
        ("Access", access),
        ("From", source.to_string()),
    ])
}

pub fn me(ctx: &Context) -> Result<()> {
    let body = ctx.client.me()?;
    if ctx.json() {
        return output::json(&body);
    }
    let me: Me = api::data(body)?;
    output::print_fields(&[
        ("Name", me.name),
        ("Email", me.email),
        ("Plan", me.plan),
        ("Locale", me.locale),
        ("Currency", me.display_currency_code),
        ("Active space", output::opt(me.active_space_id.as_deref())),
        ("ID", me.id),
        ("Member since", output::local_date(&me.created_at)),
    ])
}
