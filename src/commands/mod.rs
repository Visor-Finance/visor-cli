//! Command handlers. Each one fetches, then prints the API body for `--json`
//! or a table otherwise.

mod auth;
mod doctor;
mod pix;
mod resources;
pub(crate) mod status;
mod transactions;
mod update;

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
    if cli.global.debug {
        crate::debug::enable();
    }
    let format = if cli.global.json {
        Format::Json
    } else {
        Format::Table
    };

    let Some(command) = cli.command else {
        // Like `resend` with no command: the banner, then the help.
        crate::banner::print();
        return Cli::command()
            .print_help()
            .map_err(|error| Error::io("cannot print the help", error));
    };
    let command = match command {
        Command::Completions { shell } => {
            clap_complete::generate(shell, &mut Cli::command(), "visor", &mut io::stdout());
            return Ok(());
        }
        Command::Auth(command) => {
            let config_dir = config::config_dir()?;
            let store = Store::from_env(&config_dir)?;
            return auth::run(command, &store, &Config::load(&config_dir)?, &cli.global, format);
        }
        Command::Whoami => {
            let config_dir = config::config_dir()?;
            let store = Store::from_env(&config_dir)?;
            return auth::status(&store, &Config::load(&config_dir)?, &cli.global, format);
        }
        Command::Doctor => {
            crate::activity::start("Running checks…");
            return doctor::run(&cli.global, format);
        }
        Command::Update(args) => return update::run(&args, format),
        Command::Open { page, print } => {
            // Like `gh browse`: print the URL when asked or when no browser opens.
            if print || !crate::browser::open(page.url()) {
                output::line(page.url())?;
            } else {
                output::note(&format!("Opening {} in your browser.", page.url()));
            }
            return Ok(());
        }
        command => command,
    };

    crate::activity::start("Loading…");
    let mut ctx = Context::new(
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
        Command::Pix(args) => pix::run(&mut ctx, args),
        Command::Auth(_)
        | Command::Whoami
        | Command::Doctor
        | Command::Update(_)
        | Command::Open { .. }
        | Command::Completions { .. } => {
            unreachable!("handled above")
        }
    }
}

/// What every authenticated command needs.
pub struct Context {
    pub client: Client,
    pub format: Format,
    pub config_dir: PathBuf,
    pub config: Config,
    space: Option<String>,
    api_url: String,
    auth_url: String,
    credential: Credential,
    source: Source,
}

impl Context {
    fn new(api_url: &str, auth_url: &str, space: Option<String>, format: Format) -> Result<Self> {
        let config_dir = config::config_dir()?;
        let config = Config::load(&config_dir)?;
        let store = Store::from_env(&config_dir)?;
        let (credential, source) = credentials::resolve(&store)?.ok_or(Error::NotLoggedIn)?;

        Ok(Self {
            client: client_for(api_url, auth_url, credential.clone(), &source)?,
            format,
            config_dir,
            config,
            space: space.filter(|space| !space.trim().is_empty()),
            api_url: api_url.to_owned(),
            auth_url: auth_url.to_owned(),
            credential,
            source,
        })
    }

    /// Switches to a new login, such as one with more access, for the rest
    /// of the command.
    fn use_credential(&mut self, credential: Credential) -> Result<()> {
        self.client = client_for(&self.api_url, &self.auth_url, credential.clone(), &self.source)?;
        self.credential = credential;
        Ok(())
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

/// Fetches pages until `--limit` results (default 50) or, with `--all`,
/// until the last page, like `gh`'s `--limit`. `fetch` takes the cursor
/// and the page size; `max_page` is the API's largest page. Each page
/// counts toward the rate limit, so a long fetch shows its progress.
fn paginate(
    page: &PageArgs,
    max_page: usize,
    noun: &str,
    fetch: impl Fn(Option<&str>, usize) -> Result<Value>,
) -> Result<Value> {
    let max = page.max();
    let size = |fetched: usize| max.map_or(max_page, |max| (max - fetched).min(max_page));

    let mut body = fetch(page.after.as_deref(), size(0))?;
    let mut fetched = body["data"].as_array().map_or(0, Vec::len);
    while let Some(cursor) = body["next_cursor"].as_str().map(str::to_owned) {
        if max.is_some_and(|max| fetched >= max) {
            break;
        }
        crate::activity::set(&format!("{fetched} {noun} so far…"));
        let mut next = fetch(Some(&cursor), size(fetched))?;
        let rows = next["data"]
            .as_array_mut()
            .map(std::mem::take)
            .unwrap_or_default();
        fetched += rows.len();
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
            "More results: raise `--limit`, use `--all` for every one, or continue with `--after {cursor}`."
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
    crate::activity::stop();
    eprint!("{question} [y/N] ");
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|error| Error::io("cannot read the answer", error))?;
    Ok(matches!(answer.trim(), "y" | "Y" | "yes" | "Yes"))
}
