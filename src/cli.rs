//! Command-line arguments.

use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

use crate::client::DEFAULT_API_URL;
use crate::input;
use crate::oauth::DEFAULT_AUTH_URL;

const AFTER_HELP: &str = "\
Authentication:
  Run `visor auth login` to log in through the browser (Premium plan). On a
  machine without a browser, use a personal API token from
  https://dashboard.visorfinance.app/settings/connected_apps with
  `visor auth login --with-token`. VISOR_TOKEN overrides the stored login.

Exit codes:
  0 success, 1 failure, 2 invalid usage, 3 not authenticated or not allowed,
  4 not found, 5 request rejected, 6 rate limited or Visor unavailable

Docs: https://docs.visorfinance.app";

#[derive(Debug, Parser)]
#[command(name = "visor", version, about = "Your Visor finances from the terminal", after_help = AFTER_HELP)]
pub struct Cli {
    #[command(flatten)]
    pub global: Global,

    /// Without a command, the CLI shows the banner and this help
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Args)]
pub struct Global {
    /// Print the API response body as JSON instead of a table
    #[arg(long, global = true)]
    pub json: bool,

    /// Space to act on, by name or ID [default: the one set with `visor spaces use`]
    #[arg(long, short = 's', global = true, env = "VISOR_SPACE", value_name = "SPACE")]
    pub space: Option<String>,

    /// API base URL
    #[arg(long, global = true, env = "VISOR_API_URL", default_value = DEFAULT_API_URL, hide = true)]
    pub api_url: String,

    /// Base URL of the browser login (OAuth authorization server)
    #[arg(long, global = true, env = "VISOR_AUTH_URL", default_value = DEFAULT_AUTH_URL, hide = true)]
    pub auth_url: String,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Log in, log out, and check the stored login
    #[command(subcommand)]
    Auth(AuthCommand),

    /// Show who is logged in, how, and the space commands act on
    Whoami,

    /// Check the CLI version, the login, and the connection to Visor
    Doctor,

    /// Show the user the token belongs to
    Me,

    /// List spaces and choose the default one
    #[command(subcommand)]
    Spaces(SpacesCommand),

    /// List, record, edit, and delete transactions
    #[command(subcommand, visible_alias = "tx")]
    Transactions(TransactionsCommand),

    /// List accounts
    #[command(subcommand)]
    Accounts(ListOnly<HiddenArgs>),

    /// List categories
    #[command(subcommand)]
    Categories(ListOnly<HiddenArgs>),

    /// List tags
    #[command(subcommand)]
    Tags(ListOnly<NoArgs>),

    /// List credit cards and their bills
    #[command(subcommand)]
    Cards(CardsCommand),

    /// List subscriptions
    #[command(subcommand)]
    Subscriptions(ListOnly<NoArgs>),

    /// List fixed bills, such as rent or utilities
    #[command(subcommand, name = "fixed-bills")]
    FixedBills(ListOnly<NoArgs>),

    /// List installment plans
    #[command(subcommand)]
    Installments(ListOnly<InstallmentArgs>),

    /// List goals
    #[command(subcommand)]
    Goals(ListOnly<GoalArgs>),

    /// List investments
    #[command(subcommand)]
    Investments(ListOnly<NoArgs>),

    /// List Open Finance connections and their sync status
    #[command(subcommand)]
    Connections(ListOnly<NoArgs>),

    /// Show balances, the month so far, and the projection for a space
    Overview,

    /// List notifications
    #[command(subcommand)]
    Notifications(ListOnly<NotificationArgs>),

    /// Print a shell completion script
    #[command(after_help = "Example: visor completions zsh > \"${fpath[1]}/_visor\"")]
    Completions {
        /// Shell to generate for
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Debug, Clone, Copy, Subcommand)]
pub enum AuthCommand {
    /// Log in through the browser, or with a personal API token
    #[command(after_help = "\
Without options, the CLI opens Visor in your browser. Approve the login there
and choose what the CLI can do. The login refreshes itself; revoke it in
Settings > Connected apps (Visor CLI).

On a machine without a browser, use a personal API token instead. Create it at
https://dashboard.visorfinance.app/settings/connected_apps.

Examples:
  visor auth login
  visor auth login --with-token < token.txt")]
    Login {
        /// Use a personal API token, read from standard input or a hidden prompt
        #[arg(long)]
        with_token: bool,

        /// Print the login page URL instead of opening a browser
        #[arg(long, conflicts_with = "with_token")]
        no_browser: bool,
    },

    /// Remove the stored login
    Logout,

    /// Show which token is in use and whom it belongs to
    Status,
}

#[derive(Debug, Subcommand)]
pub enum SpacesCommand {
    /// List the spaces you can access
    #[command(visible_alias = "ls")]
    List,

    /// Set the space other commands act on
    Use {
        /// Space name or ID
        space: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ListOnly<A: Args> {
    /// List them
    #[command(visible_alias = "ls")]
    List(A),
}

#[derive(Debug, Args)]
pub struct NoArgs {}

#[derive(Debug, Args)]
pub struct HiddenArgs {
    /// Include the ones hidden in Visor
    #[arg(long)]
    pub include_hidden: bool,
}

#[derive(Debug, Args)]
pub struct GoalArgs {
    /// Only goals with this status
    #[arg(long, value_enum)]
    pub status: Option<GoalStatus>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum GoalStatus {
    Active,
    Archived,
    Completed,
}

#[derive(Debug, Args)]
pub struct InstallmentArgs {
    /// Only plans with this status
    #[arg(long, value_enum)]
    pub status: Option<InstallmentStatus>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum InstallmentStatus {
    Active,
    Completed,
}

#[derive(Debug, Args)]
pub struct NotificationArgs {
    /// Only unread notifications
    #[arg(long)]
    pub unread: bool,

    #[command(flatten)]
    pub page: PageArgs,
}

#[derive(Debug, Args)]
pub struct PageArgs {
    /// Results per page (transactions: up to 100, notifications: up to 50)
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..=100))]
    pub limit: Option<u16>,

    /// Cursor from a previous page
    #[arg(long, conflicts_with = "all")]
    pub after: Option<String>,

    /// Fetch every page
    #[arg(long)]
    pub all: bool,
}

#[derive(Debug, Subcommand)]
pub enum CardsCommand {
    /// List credit cards with their limits and current bill
    #[command(visible_alias = "ls")]
    List,

    /// List the bills of a card
    Bills {
        /// Card ID, from `visor cards list`
        #[arg(value_parser = input::id)]
        card: String,

        /// First due date to include, YYYY-MM-DD [default: start of this month]
        #[arg(long, value_parser = input::date)]
        from: Option<String>,

        /// Last due date to include, YYYY-MM-DD [default: end of this month]
        #[arg(long, value_parser = input::date)]
        to: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Direction {
    /// Money out
    Debit,
    /// Money in
    Credit,
}

impl Direction {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Debit => "debit",
            Self::Credit => "credit",
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum TransactionsCommand {
    /// List transactions, newest first
    #[command(
        visible_alias = "ls",
        after_help = "\
Examples:
  visor tx list --from 2026-10-01 --category Mercado
  visor tx list --from 2026-01-01 --to 2026-12-31 --all --json > 2026.json"
    )]
    List(TransactionListArgs),

    /// Show one transaction
    Get {
        /// Transaction ID
        #[arg(value_parser = input::id)]
        id: String,
    },

    /// Record a manual transaction
    #[command(after_help = "\
Accounts, categories, and tags accept a name or an ID.

Example:
  visor tx add --amount 18,90 --description Padaria --category Mercado --account Nubank")]
    Add(TransactionAddArgs),

    /// Edit a transaction; bank-owned fields of a connected one cannot change
    Edit(TransactionEditArgs),

    /// Delete a transaction
    #[command(visible_alias = "rm")]
    Delete {
        /// Transaction ID
        #[arg(value_parser = input::id)]
        id: String,

        /// Do not ask for confirmation
        #[arg(long, short = 'y')]
        yes: bool,
    },
}

#[derive(Debug, Args)]
pub struct TransactionListArgs {
    /// First day to include, YYYY-MM-DD (Brasília time)
    #[arg(long, value_parser = input::date)]
    pub from: Option<String>,

    /// Last day to include, YYYY-MM-DD (Brasília time) [default: today]
    #[arg(long, value_parser = input::date)]
    pub to: Option<String>,

    /// Account name or ID
    #[arg(long)]
    pub account: Option<String>,

    /// Category name, key, or ID
    #[arg(long)]
    pub category: Option<String>,

    /// Tag name or ID
    #[arg(long)]
    pub tag: Option<String>,

    /// Only debits or only credits
    #[arg(long = "type", value_name = "TYPE", value_enum)]
    pub kind: Option<Direction>,

    /// Only ignored (true) or only counted (false) transactions
    #[arg(long)]
    pub ignored: Option<bool>,

    /// Text in the description, merchant, or notes
    #[arg(long)]
    pub search: Option<String>,

    #[command(flatten)]
    pub page: PageArgs,
}

#[derive(Debug, Args)]
pub struct TransactionAddArgs {
    /// Positive amount, such as 42.90 or 42,90
    #[arg(long, value_parser = input::amount, allow_negative_numbers = true)]
    pub amount: String,

    /// What it was, as it should appear in Visor
    #[arg(long, short = 'd')]
    pub description: String,

    /// Category name, key, or ID
    #[arg(long, short = 'c')]
    pub category: String,

    /// Account name or ID; the transaction takes its currency
    #[arg(long, short = 'a')]
    pub account: String,

    /// Debit for money out, credit for money in
    #[arg(long = "type", value_name = "TYPE", value_enum, default_value_t = Direction::Debit)]
    pub kind: Direction,

    /// When it happened: YYYY-MM-DD, an RFC 3339 time, or `now`
    #[arg(long, value_parser = input::datetime, default_value = "now")]
    pub at: String,

    /// Merchant name
    #[arg(long)]
    pub merchant: Option<String>,

    /// Free-form notes
    #[arg(long)]
    pub notes: Option<String>,

    /// Tag name or ID; repeat for several
    #[arg(long = "tag")]
    pub tags: Vec<String>,

    /// Leave it out of totals
    #[arg(long)]
    pub ignored: bool,

    /// Reuse a key to retry without recording twice [default: a new UUID]
    #[arg(long, value_parser = input::idempotency_key)]
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Args)]
pub struct TransactionEditArgs {
    /// Transaction ID
    #[arg(value_parser = input::id)]
    pub id: String,

    /// Manual transactions only
    #[arg(long, value_parser = input::amount, allow_negative_numbers = true)]
    pub amount: Option<String>,

    /// Manual transactions only
    #[arg(long, short = 'd')]
    pub description: Option<String>,

    /// Category name, key, or ID
    #[arg(long, short = 'c')]
    pub category: Option<String>,

    /// Manual transactions only
    #[arg(long = "type", value_name = "TYPE", value_enum)]
    pub kind: Option<Direction>,

    /// Manual transactions only: YYYY-MM-DD, an RFC 3339 time, or `now`
    #[arg(long, value_parser = input::datetime)]
    pub at: Option<String>,

    /// New notes; an empty string removes them
    #[arg(long)]
    pub notes: Option<String>,

    /// Replace every tag; repeat for several
    #[arg(long = "tag", conflicts_with = "clear_tags")]
    pub tags: Vec<String>,

    /// Remove every tag
    #[arg(long)]
    pub clear_tags: bool,

    /// Leave it out of totals (true) or count it again (false)
    #[arg(long)]
    pub ignored: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn command_definition_is_valid() {
        Cli::command().debug_assert();
    }
}
