use std::io::{self, IsTerminal, Read};

use super::Context;
use crate::api;
use crate::cli::AuthCommand;
use crate::client::Client;
use crate::credentials::{self, Credential, Store};
use crate::error::{Error, Result, TOKENS_URL};
use crate::models::Me;
use crate::output::{self, Format};

pub fn run(command: AuthCommand, store: &Store, api_url: &str, format: Format) -> Result<()> {
    match command {
        AuthCommand::Login { with_token } => login(store, api_url, with_token, format),
        AuthCommand::Logout => logout(store),
        AuthCommand::Status => status(store, api_url, format),
    }
}

fn login(store: &Store, api_url: &str, with_token: bool, format: Format) -> Result<()> {
    let token = read_token(with_token)?;
    if !token.starts_with("visor_pat_") {
        return Err(Error::usage(
            "that is not a Visor personal API token; tokens start with `visor_pat_`",
        ));
    }

    // Check the token before storing it, so a typo never replaces a good login.
    let body = Client::new(api_url, &token)?.me()?;
    let me: Me = api::data(body.clone())?;
    store.save(&Credential::PersonalToken { token })?;

    if format == Format::Json {
        output::json(&body)?;
    }
    output::note(&format!(
        "Logged in as {} ({}). Token stored in {store}.",
        me.email, me.plan
    ));
    if std::env::var_os("VISOR_TOKEN").is_some() {
        output::note("Note: VISOR_TOKEN is set and takes precedence over the stored token.");
    }
    Ok(())
}

fn read_token(with_token: bool) -> Result<String> {
    let token = if with_token {
        let mut token = String::new();
        io::stdin()
            .read_to_string(&mut token)
            .map_err(|error| Error::io("cannot read the token from standard input", error))?;
        token
    } else if io::stdin().is_terminal() {
        output::note(&format!(
            "Create a personal API token at {TOKENS_URL}\n\
             (Read only, or Read and record transactions), then paste it below."
        ));
        rpassword::prompt_password("Token: ").map_err(|error| Error::io("cannot read the token", error))?
    } else {
        return Err(Error::usage(
            "no terminal to prompt for the token; pipe it with `visor auth login --with-token`",
        ));
    };

    let token = token.trim().to_owned();
    if token.is_empty() {
        return Err(Error::usage("the token is empty"));
    }
    Ok(token)
}

fn logout(store: &Store) -> Result<()> {
    if store.delete()? {
        output::note(&format!("Removed the token from {store}."));
        output::note(&format!(
            "The token still works until you revoke it at {TOKENS_URL}."
        ));
    } else {
        output::note("No stored token to remove.");
    }
    if std::env::var_os("VISOR_TOKEN").is_some() {
        output::note("VISOR_TOKEN is still set in this environment.");
    }
    Ok(())
}

fn status(store: &Store, api_url: &str, format: Format) -> Result<()> {
    let (credential, source) = credentials::resolve(store)?.ok_or(Error::NotLoggedIn)?;
    let body = Client::new(api_url, credential.bearer())?.me()?;

    if format == Format::Json {
        return output::json(&body);
    }
    let me: Me = api::data(body)?;
    output::print_fields(&[
        ("Logged in as", format!("{} <{}>", me.name, me.email)),
        ("Plan", me.plan),
        ("Token", credentials::redact(credential.bearer())),
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
