use std::collections::HashMap;

use serde_json::{Map, Value, json};

use super::{Context, confirm, note_next_page, paginate};
use crate::api::{self, Query};
use crate::cli::{
    Direction, TransactionAddArgs, TransactionEditArgs, TransactionListArgs, TransactionsCommand,
};
use crate::error::{Error, Result};
use crate::input;
use crate::models::{Account, Category, Transaction};
use crate::output;
use crate::resolve::{self, Named};

pub fn run(ctx: &Context, command: TransactionsCommand) -> Result<()> {
    let space = ctx.space_id()?;
    match command {
        TransactionsCommand::List(args) => list(ctx, &space, &args),
        TransactionsCommand::Get { id } => get(ctx, &space, &id),
        TransactionsCommand::Add(args) => add(ctx, &space, args),
        TransactionsCommand::Edit(args) => edit(ctx, &space, args),
        TransactionsCommand::Delete { id, yes } => delete(ctx, &space, &id, yes),
    }
}

/// Turns a name or ID into an ID, fetching the list only for a name.
fn id_of<T: Named>(query: &str, what: &str, fetch: impl FnOnce() -> Result<Vec<T>>) -> Result<String> {
    if input::is_id(query) {
        return input::id(query).map_err(Error::Usage);
    }
    let items = fetch()?;
    resolve::find(&items, query, what).map(|item| item.id().to_owned())
}

fn list(ctx: &Context, space: &str, args: &TransactionListArgs) -> Result<()> {
    let client = &ctx.client;
    let account = args
        .account
        .as_deref()
        .map(|account| id_of(account, "account", || client.accounts_typed(space)))
        .transpose()?;
    let category = args
        .category
        .as_deref()
        .map(|category| id_of(category, "category", || client.categories_typed(space)))
        .transpose()?;
    let tag = args
        .tag
        .as_deref()
        .map(|tag| id_of(tag, "tag", || client.tags_typed(space)))
        .transpose()?;

    let query = Query::new()
        .opt("from_date", args.from.as_deref())
        .opt("to_date", args.to.as_deref())
        .opt("account_id", account)
        .opt("category_id", category)
        .opt("tag_id", tag)
        .opt("type", args.kind.map(Direction::as_str))
        .opt("ignored", args.ignored)
        .opt("search", args.search.as_deref())
        .opt("limit", args.page.limit);

    let body = paginate(&args.page, |after| {
        let mut query = query.as_slice().to_vec();
        query.extend(after.map(|after| ("after", after.to_owned())));
        client.get_json(&format!("/v1/spaces/{space}/transactions"), &query)
    })?;

    if ctx.json() {
        return output::json(&body);
    }

    let transactions: Vec<Transaction> = api::data(body.clone())?;
    let names = Names::fetch(ctx, space)?;
    let mut table = output::table(&["date", "description", "category", "account", "amount", "id"]);
    for tx in &transactions {
        table.add_row(vec![
            output::local_date(&tx.transacted_at),
            describe(tx),
            names.category(tx.category_id.as_deref()),
            names.account(&tx.account_id),
            output::signed(&tx.amount, &tx.kind, &tx.currency_code),
            tx.id.clone(),
        ]);
    }
    output::print_table(&table, "No transactions found.")?;
    note_next_page(&body);
    Ok(())
}

fn describe(tx: &Transaction) -> String {
    let mut text = tx.description.clone();
    if tx.status == "pending" {
        text.push_str(" (pending)");
    }
    if tx.ignored {
        text.push_str(" (ignored)");
    }
    text
}

/// Account and category names for showing transactions.
struct Names {
    accounts: HashMap<String, String>,
    categories: HashMap<String, String>,
}

impl Names {
    fn fetch(ctx: &Context, space: &str) -> Result<Self> {
        let accounts: Vec<Account> = ctx.client.accounts_typed(space)?;
        let categories: Vec<Category> = ctx.client.categories_typed(space)?;
        Ok(Self {
            accounts: accounts.into_iter().map(|a| (a.id, a.name)).collect(),
            categories: categories.into_iter().map(|c| (c.id, c.name)).collect(),
        })
    }

    fn account(&self, id: &str) -> String {
        self.accounts.get(id).cloned().unwrap_or_else(|| id.to_owned())
    }

    fn category(&self, id: Option<&str>) -> String {
        id.map_or_else(
            || "-".to_owned(),
            |id| self.categories.get(id).cloned().unwrap_or_else(|| id.to_owned()),
        )
    }
}

fn print_transaction(ctx: &Context, space: &str, body: &Value) -> Result<()> {
    if ctx.json() {
        return output::json(body);
    }
    let tx: Transaction = api::data(body.clone())?;
    let names = Names::fetch(ctx, space)?;
    output::print_fields(&[
        ("Description", tx.description.clone()),
        ("Amount", output::signed(&tx.amount, &tx.kind, &tx.currency_code)),
        ("Date", output::local_time(Some(&tx.transacted_at))),
        ("Category", names.category(tx.category_id.as_deref())),
        ("Account", names.account(&tx.account_id)),
        ("Merchant", output::opt(tx.merchant_name.as_deref())),
        ("Notes", output::opt(tx.notes.as_deref())),
        (
            "Tags",
            if tx.tag_ids.is_empty() {
                "-".to_owned()
            } else {
                tx.tag_ids.join(", ")
            },
        ),
        ("Status", tx.status.clone()),
        ("Ignored", output::yes_no(tx.ignored)),
        ("Source", tx.source.clone()),
        ("ID", tx.id),
    ])
}

fn get(ctx: &Context, space: &str, id: &str) -> Result<()> {
    let body = ctx.client.transaction(space, id)?;
    print_transaction(ctx, space, &body)
}

fn tag_ids(ctx: &Context, space: &str, tags: &[String]) -> Result<Vec<String>> {
    if tags.iter().all(|tag| input::is_id(tag)) {
        return tags
            .iter()
            .map(|tag| input::id(tag).map_err(Error::Usage))
            .collect();
    }
    let known = ctx.client.tags_typed(space)?;
    tags.iter()
        .map(|tag| resolve::find(&known, tag, "tag").map(|tag| tag.id.clone()))
        .collect()
}

fn add(ctx: &Context, space: &str, args: TransactionAddArgs) -> Result<()> {
    let client = &ctx.client;
    let account = id_of(&args.account, "account", || client.accounts_typed(space))?;
    let category = id_of(&args.category, "category", || client.categories_typed(space))?;

    let mut body = json!({
        "account_id": account,
        "category_id": category,
        "amount": args.amount,
        "type": args.kind.as_str(),
        "description": args.description,
        "transacted_at": args.at,
    });
    let fields = body.as_object_mut().expect("body is an object");
    if let Some(merchant) = args.merchant {
        fields.insert("merchant_name".into(), merchant.into());
    }
    if let Some(notes) = args.notes {
        fields.insert("notes".into(), notes.into());
    }
    if !args.tags.is_empty() {
        fields.insert("tag_ids".into(), tag_ids(ctx, space, &args.tags)?.into());
    }
    if args.ignored {
        fields.insert("ignored".into(), true.into());
    }

    let key = args
        .idempotency_key
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let reply = client.create_transaction(space, &body, &key)?;

    let replayed = reply
        .headers
        .get("idempotent-replayed")
        .is_some_and(|value| value == "true");
    if replayed {
        output::note(&format!(
            "Already recorded with idempotency key {key}; nothing new was recorded."
        ));
    } else {
        output::note(&format!("Recorded. Idempotency key: {key}"));
    }
    print_transaction(ctx, space, &reply.body)
}

fn edit(ctx: &Context, space: &str, args: TransactionEditArgs) -> Result<()> {
    let mut fields = Map::new();
    if let Some(amount) = args.amount {
        fields.insert("amount".into(), amount.into());
    }
    if let Some(description) = args.description {
        fields.insert("description".into(), description.into());
    }
    if let Some(category) = args.category {
        let id = id_of(&category, "category", || ctx.client.categories_typed(space))?;
        fields.insert("category_id".into(), id.into());
    }
    if let Some(kind) = args.kind {
        fields.insert("type".into(), kind.as_str().into());
    }
    if let Some(at) = args.at {
        fields.insert("transacted_at".into(), at.into());
    }
    if let Some(notes) = args.notes {
        fields.insert("notes".into(), notes.into());
    }
    if args.clear_tags {
        fields.insert("tag_ids".into(), json!([]));
    } else if !args.tags.is_empty() {
        fields.insert("tag_ids".into(), tag_ids(ctx, space, &args.tags)?.into());
    }
    if let Some(ignored) = args.ignored {
        fields.insert("ignored".into(), ignored.into());
    }
    if fields.is_empty() {
        return Err(Error::usage(
            "nothing to change; pass at least one field, such as --category",
        ));
    }

    let body = ctx
        .client
        .update_transaction(space, &args.id, &Value::Object(fields))?;
    output::note("Updated.");
    print_transaction(ctx, space, &body)
}

fn delete(ctx: &Context, space: &str, id: &str, yes: bool) -> Result<()> {
    if !yes {
        super::require_terminal("pass --yes to delete without confirming")?;
        let tx: Transaction = api::data(ctx.client.transaction(space, id)?)?;
        let question = format!(
            "Delete \"{}\" ({}, {})?",
            tx.description,
            output::signed(&tx.amount, &tx.kind, &tx.currency_code),
            output::local_date(&tx.transacted_at)
        );
        if !confirm(&question)? {
            output::note("Nothing deleted.");
            return Ok(());
        }
    }
    ctx.client.delete_transaction(space, id)?;
    output::note(&format!("Deleted transaction {id}."));
    Ok(())
}
