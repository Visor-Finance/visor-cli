//! The read-only resources: one list (or summary) per command.

use serde_json::Value;

use super::{Context, note_next_page, paginate};
use crate::api::{self, Query};
use crate::cli::{
    CardsCommand, GoalArgs, GoalStatus, HiddenArgs, InstallmentArgs, InstallmentStatus, ListOnly, NoArgs,
    NotificationArgs, SpacesCommand,
};
use crate::error::Result;
use crate::models::{
    Account, Bill, Card, Category, Connection, Goal, InstallmentPlan, Investment, InvestmentSummary,
    Notification, Overview, RecurringExpense, Space, Tag,
};
use crate::output::{self, money, opt, opt_money, yes_no};
use crate::resolve;

/// Prints `body` as JSON, or as a table built from its `data` rows.
fn render<T: serde::de::DeserializeOwned>(
    ctx: &Context,
    body: &Value,
    header: &[&str],
    empty: &str,
    row: impl Fn(T) -> Vec<String>,
) -> Result<()> {
    if ctx.json() {
        return output::json(body);
    }
    let rows: Vec<T> = api::data(body.clone())?;
    let mut table = output::table(header);
    for item in rows {
        table.add_row(row(item));
    }
    output::print_table(&table, empty)
}

pub fn spaces(ctx: &Context, command: SpacesCommand) -> Result<()> {
    match command {
        SpacesCommand::List => {
            let body = ctx.client.spaces()?;
            let current = ctx.space_id().ok();
            render(
                ctx,
                &body,
                &["", "name", "type", "role", "id"],
                "No spaces found.",
                |space: Space| {
                    let marker = if current.as_deref() == Some(space.id.as_str()) {
                        "*"
                    } else {
                        ""
                    };
                    vec![marker.to_owned(), space.name, space.kind, space.role, space.id]
                },
            )
        }
        SpacesCommand::Use { space } => {
            let spaces = ctx.client.spaces_typed()?;
            let space = resolve::find(&spaces, &space, "space")?;
            let mut config = ctx.config.clone();
            config.default_space = Some(space.id.clone());
            config.save(&ctx.config_dir)?;
            output::note(&format!(
                "Commands now act on the space \"{}\" ({}).",
                space.name, space.id
            ));
            Ok(())
        }
    }
}

pub fn accounts(ctx: &Context, ListOnly::List(args): ListOnly<HiddenArgs>) -> Result<()> {
    let query = Query::new().flag("include_hidden", args.include_hidden);
    let body = ctx
        .client
        .space_collection(&ctx.space_id()?, "accounts", &query)?;
    render(
        ctx,
        &body,
        &["name", "type", "institution", "balance", "id"],
        "No accounts found.",
        |account: Account| {
            let mut name = account.name;
            if account.hidden {
                name.push_str(" (hidden)");
            }
            vec![
                name,
                account.kind,
                opt(account.provider_name.as_deref()),
                money(&account.balance, &account.currency_code),
                account.id,
            ]
        },
    )
}

pub fn categories(ctx: &Context, ListOnly::List(args): ListOnly<HiddenArgs>) -> Result<()> {
    let query = Query::new().flag("include_hidden", args.include_hidden);
    let body = ctx
        .client
        .space_collection(&ctx.space_id()?, "categories", &query)?;
    if ctx.json() {
        return output::json(&body);
    }

    // Children print under their parent, indented.
    let categories: Vec<Category> = api::data(body)?;
    let mut table = output::table(&["name", "type", "key", "id"]);
    let row = |category: &Category, indent: &str| {
        let mut name = format!("{indent}{}", category.name);
        if category.hidden {
            name.push_str(" (hidden)");
        }
        vec![
            name,
            opt(category.kind.as_deref()),
            opt(category.key.as_deref()),
            category.id.clone(),
        ]
    };
    for parent in categories.iter().filter(|c| c.parent_id.is_none()) {
        table.add_row(row(parent, ""));
        for child in categories
            .iter()
            .filter(|c| c.parent_id.as_deref() == Some(&parent.id))
        {
            table.add_row(row(child, "  "));
        }
    }
    let known = |id: &str| categories.iter().any(|c| c.id == id);
    for orphan in categories
        .iter()
        .filter(|c| c.parent_id.as_deref().is_some_and(|parent| !known(parent)))
    {
        table.add_row(row(orphan, ""));
    }
    output::print_table(&table, "No categories found.")
}

pub fn tags(ctx: &Context, ListOnly::List(NoArgs {}): ListOnly<NoArgs>) -> Result<()> {
    let body = ctx
        .client
        .space_collection(&ctx.space_id()?, "tags", &Query::new())?;
    render(
        ctx,
        &body,
        &["name", "color", "id"],
        "No tags found.",
        |tag: Tag| vec![tag.name, tag.color, tag.id],
    )
}

pub fn cards(ctx: &Context, command: CardsCommand) -> Result<()> {
    let space = ctx.space_id()?;
    match command {
        CardsCommand::List => {
            let body = ctx.client.space_collection(&space, "cards", &Query::new())?;
            render(
                ctx,
                &body,
                &["name", "used", "available", "limit", "closes", "due", "id"],
                "No credit cards found.",
                |card: Card| {
                    let currency = &card.currency_code;
                    vec![
                        card.name.clone(),
                        opt_money(card.used_limit.as_deref(), currency),
                        opt_money(card.available_limit.as_deref(), currency),
                        opt_money(card.credit_limit.as_deref(), currency),
                        opt(card.current_bill_close_date.as_deref()),
                        opt(card.current_bill_due_date.as_deref()),
                        card.id.clone(),
                    ]
                },
            )
        }
        CardsCommand::Bills { card, from, to } => {
            let query = Query::new().opt("from", from).opt("to", to);
            let body = ctx.client.card_bills(&space, &card, &query)?;
            render(
                ctx,
                &body,
                &["due", "closes", "status", "total", "paid", "minimum"],
                "No bills in this period.",
                |bill: Bill| {
                    let currency = &bill.currency_code;
                    let mut total = opt_money(bill.total_amount.as_deref(), currency);
                    if bill.amount_is_estimate {
                        total.push_str(" (estimate)");
                    }
                    vec![
                        opt(bill.due_date.as_deref()),
                        opt(bill.close_date.as_deref()),
                        bill.status.clone(),
                        total,
                        opt_money(bill.paid_amount.as_deref(), currency),
                        opt_money(bill.minimum_payment.as_deref(), currency),
                    ]
                },
            )
        }
    }
}

pub fn recurring(ctx: &Context, ListOnly::List(NoArgs {}): ListOnly<NoArgs>, collection: &str) -> Result<()> {
    let body = ctx
        .client
        .space_collection(&ctx.space_id()?, collection, &Query::new())?;
    render(
        ctx,
        &body,
        &[
            "name",
            "amount",
            "frequency",
            "next due",
            "last paid",
            "active",
            "id",
        ],
        "Nothing found.",
        |expense: RecurringExpense| {
            vec![
                expense.name,
                money(&expense.amount, &expense.currency_code),
                opt(expense.frequency.as_deref()),
                opt(expense.next_due_date.as_deref()),
                opt(expense.last_paid_date.as_deref()),
                yes_no(expense.active),
                expense.id,
            ]
        },
    )
}

pub fn installments(ctx: &Context, ListOnly::List(args): ListOnly<InstallmentArgs>) -> Result<()> {
    let status = args.status.map(|status| match status {
        InstallmentStatus::Active => "active",
        InstallmentStatus::Completed => "completed",
    });
    let query = Query::new().opt("status", status);
    let body = ctx
        .client
        .space_collection(&ctx.space_id()?, "installment-plans", &query)?;
    render(
        ctx,
        &body,
        &[
            "description",
            "installment",
            "paid",
            "remaining",
            "next due",
            "status",
            "id",
        ],
        "No installment plans found.",
        |plan: InstallmentPlan| {
            let currency = &plan.currency_code;
            vec![
                plan.description.clone(),
                money(&plan.installment_amount, currency),
                format!("{}/{}", plan.paid_installments, plan.total_installments),
                money(&plan.remaining_amount, currency),
                opt(plan.next_due_date.as_deref()),
                plan.status.clone(),
                plan.id.clone(),
            ]
        },
    )
}

pub fn goals(ctx: &Context, ListOnly::List(args): ListOnly<GoalArgs>) -> Result<()> {
    let status = args.status.map(|status| match status {
        GoalStatus::Active => "active",
        GoalStatus::Archived => "archived",
        GoalStatus::Completed => "completed",
    });
    let query = Query::new().opt("status", status);
    let body = ctx.client.space_collection(&ctx.space_id()?, "goals", &query)?;
    render(
        ctx,
        &body,
        &[
            "name",
            "saved",
            "target",
            "progress",
            "target date",
            "status",
            "id",
        ],
        "No goals found.",
        |goal: Goal| {
            let currency = &goal.currency_code;
            vec![
                goal.name.clone(),
                money(&goal.current_amount, currency),
                money(&goal.target_amount, currency),
                format!("{}%", goal.progress_percent),
                opt(goal.target_date.as_deref()),
                goal.status.clone(),
                goal.id.clone(),
            ]
        },
    )
}

pub fn investments(ctx: &Context, ListOnly::List(NoArgs {}): ListOnly<NoArgs>) -> Result<()> {
    let body = ctx
        .client
        .space_collection(&ctx.space_id()?, "investments", &Query::new())?;
    render(
        ctx,
        &body,
        &["name", "category", "institution", "balance", "id"],
        "No investments found.",
        |investment: Investment| {
            vec![
                investment.name,
                investment.category,
                opt(investment.provider_name.as_deref()),
                money(&investment.balance, &investment.currency_code),
                investment.id,
            ]
        },
    )?;
    if !ctx.json() {
        let summary: InvestmentSummary = crate::client::decode(body["summary"].clone())?;
        if summary.count > 0 {
            output::note(&format!(
                "Total: {} across {} investments",
                money(&summary.total_value, &summary.currency_code),
                summary.count
            ));
        }
    }
    Ok(())
}

pub fn connections(ctx: &Context, ListOnly::List(NoArgs {}): ListOnly<NoArgs>) -> Result<()> {
    let body = ctx
        .client
        .space_collection(&ctx.space_id()?, "connections", &Query::new())?;
    render(
        ctx,
        &body,
        &[
            "institution",
            "status",
            "last sync",
            "next sync",
            "accounts",
            "id",
        ],
        "No connections found.",
        |connection: Connection| {
            let mut status = connection.status;
            if connection.requires_reauth {
                status.push_str(" (reconnect in Visor)");
            } else if connection.paused {
                status.push_str(" (paused)");
            }
            vec![
                opt(connection.provider_name.as_deref()),
                status,
                output::local_time(connection.last_successful_sync_at.as_deref()),
                output::local_time(connection.next_auto_sync_at.as_deref()),
                connection.account_ids.len().to_string(),
                connection.id,
            ]
        },
    )
}

pub fn overview(ctx: &Context) -> Result<()> {
    let body = ctx
        .client
        .space_collection(&ctx.space_id()?, "overview", &Query::new())?;
    if ctx.json() {
        return output::json(&body);
    }
    let overview: Overview = api::data(body)?;
    let currency = overview.currency_code.as_str();
    let committed = &overview.projection.committed_outflows;
    output::print_fields(&[
        (
            "Period",
            format!("{} to {}", overview.period_start, overview.period_end),
        ),
        ("Bank balance", money(&overview.bank_balance, currency)),
        ("Credit card debt", money(&overview.credit_card_debt, currency)),
        ("Open bills", money(&overview.open_bills_total, currency)),
        ("Net worth", money(&overview.net_worth, currency)),
        ("Income", money(&overview.period.income, currency)),
        ("Expenses", money(&overview.period.expenses, currency)),
        ("Result", money(&overview.period.result, currency)),
        (
            "Expected income",
            money(&overview.projection.expected_income, currency),
        ),
        ("Committed bills", money(&committed.bills, currency)),
        ("Committed installments", money(&committed.installments, currency)),
        (
            "Committed subscriptions",
            money(&committed.subscriptions, currency),
        ),
        ("Scheduled", money(&committed.scheduled, currency)),
        (
            "Day-to-day estimate",
            money(&overview.projection.estimated_day_to_day, currency),
        ),
        (
            "Projected balance",
            money(&overview.projection.projected_balance, currency),
        ),
    ])?;

    if !overview.upcoming_bills.is_empty() {
        output::line("")?;
        let mut table = output::table(&["due", "upcoming bill", "amount"]);
        for bill in &overview.upcoming_bills {
            let mut amount = money(&bill.amount, currency);
            if bill.amount_is_estimate {
                amount.push_str(" (estimate)");
            }
            table.add_row(vec![bill.due_date.clone(), bill.description.clone(), amount]);
        }
        output::print_table(&table, "")?;
    }
    Ok(())
}

pub fn notifications(ctx: &Context, ListOnly::List(args): ListOnly<NotificationArgs>) -> Result<()> {
    let query = Query::new().flag("unread", args.unread);
    let body = paginate(&args.page, 50, "notifications", |after, size| {
        let mut query = query.as_slice().to_vec();
        query.push(("limit", size.to_string()));
        query.extend(after.map(|after| ("after", after.to_owned())));
        ctx.client.get_json("/v1/notifications", &query)
    })?;
    render(
        ctx,
        &body,
        &["date", "title", "message", "read"],
        "No notifications.",
        |notification: Notification| {
            vec![
                output::local_time(Some(&notification.created_at)),
                notification.title,
                notification.body,
                yes_no(notification.read),
            ]
        },
    )?;
    if !ctx.json() {
        note_next_page(&body);
    }
    Ok(())
}
