use std::io::{self, IsTerminal, Read};

use super::status::Status;
use super::{Context, client_for};
use crate::api;
use crate::banner;
use crate::cli::{AuthCommand, Global};
use crate::client::Client;
use crate::config::Config;
use crate::credentials::{self, Credential, Source, Store};
use crate::error::{Error, Result, TOKENS_URL};
use crate::models::Me;
use crate::oauth;
use crate::output::{self, Format, Paint};

pub fn run(
    command: AuthCommand,
    store: &Store,
    config: &Config,
    global: &Global,
    format: Format,
) -> Result<()> {
    match command {
        AuthCommand::Login { with_token: true, .. } => {
            banner::print();
            let credential = Credential::PersonalToken { token: read_token()? };
            finish_login(store, config, &global.api_url, &credential, format)
        }
        AuthCommand::Login { no_browser, .. } => {
            banner::print();
            let credential = oauth::login(&global.auth_url, !no_browser)?;
            finish_login(store, config, &global.api_url, &credential, format)
        }
        AuthCommand::Logout => logout(store),
        AuthCommand::Status => status(store, config, global, format),
    }
}

/// Checks the credential with the API, then stores it, so a bad token
/// never replaces a good login. Then shows the account, like `fly auth
/// login` does, and what to try next.
fn finish_login(
    store: &Store,
    config: &Config,
    api_url: &str,
    credential: &Credential,
    format: Format,
) -> Result<()> {
    if let Credential::PersonalToken { token } = credential
        && !token.starts_with("visor_pat_")
    {
        return Err(Error::usage(
            "that is not a Visor personal API token; tokens start with `visor_pat_`",
        ));
    }
    let client = Client::new(api_url, credential.bearer())?;
    let status = Status::fetch(&client, credential.clone(), Source::Store(store.clone()), config)?;
    store.save(credential)?;

    if format == Format::Json {
        output::json(&status.to_json())?;
    } else {
        let paint = Paint::stderr();
        output::note(&format!("\n{}", status.render(paint)));
        output::note(&format!(
            "{}\n  {}  balances, this period, and the projection\n  {}   recent transactions\n  \
             {} choose the space commands act on\n  {}     check the CLI, the login, and the API\n",
            paint.bold("Next steps"),
            paint.bold("visor overview"),
            paint.bold("visor tx list"),
            paint.bold("visor spaces use"),
            paint.bold("visor doctor"),
        ));
    }
    if std::env::var_os("VISOR_TOKEN").is_some() {
        output::note(&format!(
            "{} VISOR_TOKEN is set, so the CLI keeps using it instead of this login. Unset it to use this login.",
            Paint::stderr().warn()
        ));
    }
    Ok(())
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

/// `visor auth status` and `visor whoami`.
pub fn status(store: &Store, config: &Config, global: &Global, format: Format) -> Result<()> {
    let (credential, source) = credentials::resolve(store)?.ok_or(Error::NotLoggedIn)?;
    let client = client_for(&global.api_url, &global.auth_url, credential.clone(), &source)?;
    let status = Status::fetch(&client, credential, source, config)?;

    match format {
        Format::Json => output::json(&status.to_json()),
        Format::Table => output::line(status.render(Paint::stdout()).trim_end()),
    }
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
