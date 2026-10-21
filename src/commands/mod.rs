//! Command handlers. Each one fetches, then prints the API body for `--json`
//! or a table otherwise.

mod auth;
mod resources;
mod transactions;

use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::sync::Arc;

use clap::CommandFactory;
use serde_json::Value;

use crate::cli::{Cli, Command, PageArgs};
use crate::client::Client;
use crate::config::{self, Config};
use crate::credentials::{self, Credential, Source, Store};
use crate::error::{Error, Result};
use crate::oauth::Session;
use crate::output::{self, Format};
use crate::{input, resolve};

pub fn run(cli: Cli) -> Result<()> {
    let format = if cli.global.json {
        Format::Json
    } else {
        Format::Table
    };

    let command = match cli.command {
        Command::Completions { shell } => {
            clap_complete::generate(shell, &mut Cli::command(), "visor", &mut io::stdout());
            return Ok(());
        }
        Command::Auth(command) => {
            let config_dir = config::config_dir()?;
            let store = Store::from_env(&config_dir)?;
            return auth::run(command, &store, &cli.global, format);
        }
        command => command,
    };

    let ctx = Context::new(
        &cli.global.api_url,
        &cli.global.auth_url,
        cli.global.space,
        format,
    )?;
    match command {
        Command::Me => auth::me(&ctx),
        Command::Spaces(command) => resources::spaces(&ctx, command),
        Command::Transactions(command) => transactions::run(&ctx, command),
        Command::Accounts(command) => resources::accounts(&ctx, command),
        Command::Categories(command) => resources::categories(&ctx, command),
        Command::Tags(command) => resources::tags(&ctx, command),
        Command::Cards(command) => resources::cards(&ctx, command),
        Command::Subscriptions(command) => resources::recurring(&ctx, command, "subscriptions"),
        Command::FixedBills(command) => resources::recurring(&ctx, command, "fixed-bills"),
        Command::Installments(command) => resources::installments(&ctx, command),
        Command::Goals(command) => resources::goals(&ctx, command),
        Command::Investments(command) => resources::investments(&ctx, command),
        Command::Connections(command) => resources::connections(&ctx, command),
        Command::Overview => resources::overview(&ctx),
        Command::Notifications(command) => resources::notifications(&ctx, command),
        Command::Auth(_) | Command::Completions { .. } => unreachable!("handled above"),
    }
}

/// What every authenticated command needs.
pub struct Context {
    pub client: Client,
    pub format: Format,
    pub config_dir: PathBuf,
    pub config: Config,
    space: Option<String>,
}

impl Context {
    fn new(api_url: &str, auth_url: &str, space: Option<String>, format: Format) -> Result<Self> {
        let config_dir = config::config_dir()?;
        let config = Config::load(&config_dir)?;
        let store = Store::from_env(&config_dir)?;
        let (credential, source) = credentials::resolve(&store)?.ok_or(Error::NotLoggedIn)?;

        Ok(Self {
            client: client_for(api_url, auth_url, credential, &source)?,
            format,
            config_dir,
            config,
            space: space.filter(|space| !space.trim().is_empty()),
        })
    }

    /// The space to act on: `--space`, then `visor spaces use`, then the
    /// default personal space.
    pub fn space_id(&self) -> Result<String> {
        if let Some(space) = &self.space {
            if input::is_id(space) {
                return input::id(space).map_err(Error::Usage);
            }
            let spaces = self.client.spaces_typed()?;
            return resolve::find(&spaces, space, "space").map(|space| space.id.clone());
        }
        if let Some(space) = &self.config.default_space {
            return Ok(space.clone());
        }
        let spaces = self.client.spaces_typed()?;
        resolve::default_space(&spaces).map(|space| space.id.clone())
    }

    #[must_use]
    pub fn json(&self) -> bool {
        self.format == Format::Json
    }
}

/// A client for the credential: a browser login refreshes itself and saves
/// each refresh in the store it came from.
pub(crate) fn client_for(
    api_url: &str,
    auth_url: &str,
    credential: Credential,
    source: &Source,
) -> Result<Client> {
    match (&credential, source) {
        (Credential::OAuth { .. }, Source::Store(store)) => Client::with_source(
            api_url,
            Arc::new(Session::new(auth_url, store.clone(), credential)),
        ),
        _ => Client::new(api_url, credential.bearer()),
    }
}

/// Fetches one page, or every page with `--all`. `fetch` takes the cursor.
fn paginate(page: &PageArgs, fetch: impl Fn(Option<&str>) -> Result<Value>) -> Result<Value> {
    let mut body = fetch(page.after.as_deref())?;
    if !page.all {
        return Ok(body);
    }
    while let Some(cursor) = body["next_cursor"].as_str().map(str::to_owned) {
        let mut next = fetch(Some(&cursor))?;
        let rows = next["data"]
            .as_array_mut()
            .map(std::mem::take)
            .unwrap_or_default();
        body["data"]
            .as_array_mut()
            .ok_or_else(|| Error::Decode("`data` is not a list".into()))?
            .extend(rows);
        body["next_cursor"] = next["next_cursor"].take();
    }
    Ok(body)
}

fn note_next_page(body: &Value) {
    if let Some(cursor) = body["next_cursor"].as_str() {
        output::note(&format!(
            "More results: add `--after {cursor}`, or `--all` for every page."
        ));
    }
}

/// Fails before any work when a prompt would have no one to answer it.
fn require_terminal(hint: &str) -> Result<()> {
    if io::stdin().is_terminal() {
        Ok(())
    } else {
        Err(Error::usage(format!(
            "no terminal to ask for confirmation; {hint}"
        )))
    }
}

/// Asks a yes/no question on the terminal; call [`require_terminal`] first.
fn confirm(question: &str) -> Result<bool> {
    eprint!("{question} [y/N] ");
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|error| Error::io("cannot read the answer", error))?;
    Ok(matches!(answer.trim(), "y" | "Y" | "yes" | "Yes"))
}
