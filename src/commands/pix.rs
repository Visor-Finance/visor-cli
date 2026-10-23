//! `visor pix`: start a Pix, show the page where you confirm it (on this
//! computer or as a QR code for your phone), and follow it until it ends.

use std::fmt::Write as _;
use std::thread;
use std::time::{Duration, Instant};

use qrcode::QrCode;
use qrcode::render::unicode::Dense1x2;
use serde_json::{Value, json};

use super::Context;
use crate::api;
use crate::cli::{PixArgs, PixCommand, PixSendArgs};
use crate::credentials::{Credential, Source};
use crate::error::{Error, PIX_SCOPE, Result, TOKENS_URL};
use crate::models::PixPayment;
use crate::oauth;
use crate::output::{self, Paint};
use crate::picker::{self, Choice};

/// The API allows 60 requests a minute; one every 5 s leaves room for the rest.
const POLL_INTERVAL: Duration = Duration::from_secs(5);
/// The confirmation page expires well before this.
const WAIT_LIMIT: Duration = Duration::from_mins(15);

pub fn run(ctx: &mut Context, args: PixArgs) -> Result<()> {
    match args.command {
        Some(PixCommand::Status { id }) => status(ctx, &id),
        None => send(ctx, args.send),
    }
}

/// Where the user confirms the Pix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Device {
    /// Open the page in the browser here.
    Computer,
    /// Show the page as a QR code.
    Phone,
    /// Only print the link: no terminal, or `--json`.
    Link,
}

fn send(ctx: &mut Context, args: PixSendArgs) -> Result<()> {
    let (Some(key), Some(amount)) = (args.key, args.amount) else {
        return Err(Error::usage(
            "pass the Pix key and the amount, such as `visor pix maria@example.com 20,00`",
        ));
    };
    let mut body = json!({"pix_key": key, "amount": amount});
    let fields = body.as_object_mut().expect("body is an object");
    if let Some(description) = args.description {
        fields.insert("description".into(), description.into());
    }
    if let Some(bank) = args.bank {
        fields.insert("bank".into(), bank.into());
    }
    if let Some(key_type) = args.key_type {
        fields.insert("key_type".into(), key_type.as_str().into());
    }
    let idempotency_key = args
        .idempotency_key
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    crate::activity::set("Starting the Pix…");
    let mut allowed_pix = false;
    let reply = loop {
        // Each retry sends the same key: the API started nothing for an error.
        match ctx.client.create_pix_payment(&body, &idempotency_key) {
            Ok(reply) => break reply,
            Err(error) if !allowed_pix && lacks_pix_scope(&error) && can_allow_pix(ctx) => {
                allowed_pix = true;
                allow_pix(ctx)?;
            }
            Err(error) => match pick_missing(ctx, &error, &body)? {
                Some((field, value)) => {
                    body[field] = value.into();
                }
                None => return Err(error),
            },
        }
        crate::activity::start("Starting the Pix…");
    };
    let pix: PixPayment = api::data(reply.body.clone())?;
    if reply
        .headers
        .get("idempotent-replayed")
        .is_some_and(|value| value == "true")
    {
        output::note(&format!(
            "This Pix was already started with idempotency key {idempotency_key}; nothing new was started."
        ));
    }
    output::note(&summary(&pix));

    let Some(url) = pix.start_url.clone().filter(|_| !pix.is_final()) else {
        return finish(ctx, &reply.body);
    };
    let device = if args.open {
        Device::Computer
    } else if args.qr {
        Device::Phone
    } else if args.no_wait || ctx.json() || !output::interactive() {
        Device::Link
    } else {
        ask_device(&pix)?
    };

    if args.no_wait {
        if device != Device::Link {
            show(device, &url);
        }
        if ctx.json() {
            return output::json(&reply.body);
        }
        output::print_fields(&[("ID", pix.id.clone()), ("Link", url)])?;
        output::note(&format!("Follow it with `visor pix status {}`.", pix.id));
        return Ok(());
    }

    show(device, &url);
    let body = wait(ctx, &pix)?;
    finish(ctx, &body)
}

/// `Pix of R$ 20,00 to ***.456.789-** from Nubank`, and what comes first.
fn summary(pix: &PixPayment) -> String {
    let mut text = format!(
        "Pix of {} to {}",
        output::brl(&pix.amount),
        pix.recipient.as_deref().unwrap_or(&pix.pix_key_masked)
    );
    if let Some(bank) = &pix.bank {
        let _ = write!(text, " from {}", bank.name);
        if !bank.linked && !pix.is_final() {
            let _ = write!(text, ". First you link {} to Visor, once", bank.name);
        }
    }
    text
}

/// Asks where to confirm. The phone comes first while the bank is not
/// linked yet: linking happens in the bank's app.
fn ask_device(pix: &PixPayment) -> Result<Device> {
    let mut options = [
        (
            Device::Phone,
            Choice::new("On your phone").detail("Shows a QR code; scan it with the phone's camera."),
        ),
        (
            Device::Computer,
            Choice::new("On this computer").detail("Opens the page in your browser here."),
        ),
    ];
    if pix.bank.as_ref().is_some_and(|bank| bank.linked) {
        options.reverse();
    }
    let choices: Vec<Choice> = options.iter().map(|(_, choice)| choice.clone()).collect();
    let picked = picker::choose("Where do you want to confirm the Pix?", &choices)?;
    Ok(options[picked].0)
}

/// When the API answers with the choices it needs (the banks, or whether a
/// key is a CPF or a phone), asks the user to pick one on a terminal.
/// Returns the body field to set, or `None` to fail with the error, which
/// lists the choices and the flag that picks one.
fn pick_missing(ctx: &Context, error: &Error, body: &Value) -> Result<Option<(&'static str, String)>> {
    let Error::Api(api) = error else {
        return Ok(None);
    };
    if ctx.json() || !output::interactive() {
        return Ok(None);
    }
    let (field, question, options): (&str, String, Vec<(String, String)>) = match api.code.as_str() {
        code @ ("BANK_REQUIRED" | "BANK_NOT_FOUND") => {
            let banks: Vec<(String, String)> = api.details["banks"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(|bank| (bank.to_owned(), bank.to_owned()))
                .collect();
            let question = if code == "BANK_NOT_FOUND" {
                "Pix through Visor does not work from that bank. Which bank does the Pix come from?"
            } else {
                "Which bank does the Pix come from?"
            };
            ("bank", question.to_owned(), banks)
        }
        "AMBIGUOUS_PIX_KEY" => (
            "key_type",
            format!(
                "Is {} a CPF or a phone number?",
                body["pix_key"].as_str().unwrap_or("the key")
            ),
            vec![
                ("CPF".to_owned(), "cpf".to_owned()),
                ("Phone number".to_owned(), "phone".to_owned()),
            ],
        ),
        _ => return Ok(None),
    };
    // A choice the API already refused would only ask again.
    let options: Vec<(String, String)> = options
        .into_iter()
        .filter(|(_, value)| body[field].as_str() != Some(value))
        .collect();
    if options.is_empty() {
        return Ok(None);
    }
    let choices: Vec<Choice> = options
        .iter()
        .map(|(label, _)| Choice::new(label.clone()))
        .collect();
    let picked = picker::choose(&question, &choices)?;
    Ok(Some((field, options[picked].1.clone())))
}

fn show(device: Device, url: &str) {
    match device {
        Device::Computer if crate::browser::open(url) => {
            output::note(&format!("Opening the Pix page in your browser:\n\n    {url}\n"));
        }
        Device::Phone => match qr(url) {
            Some(code) => output::note(&format!(
                "Scan this code with your phone's camera to continue the Pix:\n\n{code}\nOr open: {url}\n"
            )),
            None => output::note(&format!(
                "Open this page on your phone to continue the Pix:\n\n    {url}\n"
            )),
        },
        Device::Computer | Device::Link => {
            output::note(&format!("Open this page to continue the Pix:\n\n    {url}\n"));
        }
    }
}

/// The QR code of `url` in Unicode half blocks: light modules are drawn, so
/// it scans on a dark terminal, with a quiet zone around it.
fn qr(url: &str) -> Option<String> {
    let code = QrCode::new(url.as_bytes()).ok()?;
    Some(
        code.render::<Dense1x2>()
            .dark_color(Dense1x2::Light)
            .light_color(Dense1x2::Dark)
            .quiet_zone(true)
            .build(),
    )
}

/// Reads the Pix every few seconds until it ends. The page may move on (the
/// bank linked, the payment created); it leads the user to the next step
/// itself, so nothing opens again.
fn wait(ctx: &Context, started: &PixPayment) -> Result<Value> {
    output::note(&Paint::stderr().dim(&format!(
        "Ctrl+C stops waiting, not the Pix. Check it later with `visor pix status {}`.",
        started.id
    )));
    crate::activity::start(waiting_message(started));
    let deadline = Instant::now() + WAIT_LIMIT;
    loop {
        let body = ctx.client.pix_payment(&started.id)?;
        let pix: PixPayment = api::data(body.clone())?;
        if pix.is_final() {
            crate::activity::stop();
            return Ok(body);
        }
        crate::activity::set(waiting_message(&pix));
        if Instant::now() + POLL_INTERVAL > deadline {
            return Err(Error::PixWaitTimeout {
                id: pix.id,
                minutes: WAIT_LIMIT.as_secs() / 60,
            });
        }
        thread::sleep(POLL_INTERVAL);
    }
}

fn waiting_message(pix: &PixPayment) -> &'static str {
    match pix.waiting_for.as_deref() {
        Some("bank_authorization") => "Waiting for you to link the bank…",
        Some("payment_confirmation") => "Waiting for you to confirm the Pix…",
        _ => "Waiting for the Pix…",
    }
}

/// Prints the Pix, and fails with exit code 5 unless it was paid.
fn finish(ctx: &Context, body: &Value) -> Result<()> {
    let pix: PixPayment = api::data(body.clone())?;
    if ctx.json() {
        output::json(body)?;
    } else {
        if pix.status == "completed" {
            output::note(&format!("{} Pix completed", Paint::stderr().ok()));
        }
        output::print_fields(&fields(&pix))?;
    }
    match pix.status.as_str() {
        "completed" => Ok(()),
        status => Err(Error::PixNotPaid {
            status: status.to_owned(),
            message: not_paid_message(&pix),
        }),
    }
}

fn not_paid_message(pix: &PixPayment) -> String {
    match (pix.status.as_str(), &pix.failure) {
        (_, Some(failure)) => format!("the Pix was not paid: {} ({})", failure.message, failure.code),
        ("canceled", None) => "the Pix was canceled; nothing was paid".to_owned(),
        ("expired", None) => {
            "the Pix expired before it was confirmed; nothing was paid. Run the command again to start a new one"
                .to_owned()
        }
        (status, None) => format!("the Pix ended as {status}; nothing was paid"),
    }
}

fn status(ctx: &Context, id: &str) -> Result<()> {
    let body = ctx.client.pix_payment(id)?;
    if ctx.json() {
        return output::json(&body);
    }
    let pix: PixPayment = api::data(body)?;
    output::print_fields(&fields(&pix))
}

fn fields(pix: &PixPayment) -> Vec<(&'static str, String)> {
    let mut fields = vec![
        ("Status", pix.status.clone()),
        ("Amount", output::brl(&pix.amount)),
        ("To", output::opt(pix.recipient.as_deref())),
        ("Pix key", pix.pix_key_masked.clone()),
        (
            "Bank",
            output::opt(pix.bank.as_ref().map(|bank| bank.name.as_str())),
        ),
    ];
    if let Some(description) = &pix.description {
        fields.push(("Description", description.clone()));
    }
    if let Some(end_to_end_id) = &pix.end_to_end_id {
        fields.push(("Pix ID", end_to_end_id.clone()));
    }
    if pix.paid_at.is_some() {
        fields.push(("Paid at", output::local_time(pix.paid_at.as_deref())));
    }
    if let Some(failure) = &pix.failure {
        fields.push(("Failure", failure.message.clone()));
    }
    if let Some(url) = pix.start_url.as_ref().filter(|_| !pix.is_final()) {
        fields.push(("Continue at", url.clone()));
    }
    fields.push(("Started", output::local_time(Some(&pix.created_at))));
    fields.push(("ID", pix.id.clone()));
    fields
}

fn lacks_pix_scope(error: &Error) -> bool {
    matches!(error, Error::Api(api)
        if api.code == "INSUFFICIENT_SCOPE" && api.details["required_scope"] == PIX_SCOPE)
}

/// The CLI asks for Pix access itself only when someone can approve it in
/// the browser and the login is one it stores; `VISOR_TOKEN` stays as is.
fn can_allow_pix(ctx: &Context) -> bool {
    !ctx.json() && output::interactive() && matches!(ctx.source, Source::Store(_))
}

/// Runs the browser login again asking for Pix on top of the access the
/// login has, stores the new login, and ends the old one.
fn allow_pix(ctx: &mut Context) -> Result<()> {
    let Source::Store(store) = ctx.source.clone() else {
        unreachable!("checked by can_allow_pix");
    };
    let scope = match &ctx.credential {
        Credential::OAuth { scope, .. } => with_scope(scope, PIX_SCOPE),
        Credential::PersonalToken { .. } => with_scope("pfm:read", PIX_SCOPE),
    };
    output::note("The Visor CLI cannot start a Pix yet. Opening the browser so you can allow it…\n");
    let credential = oauth::login(&ctx.auth_url, true, Some(&scope))?;
    if let Credential::OAuth { scope, .. } = &credential
        && !scope.is_empty()
        && !scope.split_whitespace().any(|granted| granted == PIX_SCOPE)
    {
        return Err(Error::Login(
            "the new login does not allow Pix. Run `visor pix` again and turn on Pix on the Visor page"
                .to_owned(),
        ));
    }

    // A refresh may have replaced the login since the command started; the
    // store has the latest one.
    let previous = store.load().ok().flatten();
    store.save(&credential)?;
    match previous {
        Some(Credential::OAuth { refresh_token, .. }) => {
            if let Err(error) = oauth::revoke(&ctx.auth_url, &refresh_token) {
                output::note(&format!(
                    "{} Could not end the previous login ({error}). Revoke it at {TOKENS_URL}.",
                    Paint::stderr().warn()
                ));
            }
        }
        Some(Credential::PersonalToken { .. }) => output::note(&format!(
            "The CLI now uses this login. The personal API token still works until you revoke it at {TOKENS_URL}."
        )),
        None => {}
    }
    ctx.use_credential(credential)?;
    output::note(&format!(
        "{} The Visor CLI can start Pix now",
        Paint::stderr().ok()
    ));
    Ok(())
}

/// `scope` plus `extra`, once.
fn with_scope(scope: &str, extra: &str) -> String {
    let mut scopes: Vec<&str> = scope.split_whitespace().collect();
    if !scopes.contains(&extra) {
        scopes.push(extra);
    }
    scopes.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PixPaymentBank;

    fn pix(status: &str, bank_linked: bool) -> PixPayment {
        PixPayment {
            id: "7b2d5e91-3c4a-4f68-9a0b-1c2d3e4f5a6b".into(),
            status: status.into(),
            waiting_for: Some("bank_authorization".into()),
            amount: "20.00".into(),
            currency: "BRL".into(),
            pix_key_masked: "***.456.789-**".into(),
            recipient: None,
            description: None,
            bank: Some(PixPaymentBank {
                name: "Nubank".into(),
                linked: bank_linked,
            }),
            start_url: Some("https://dashboard.visorfinance.app/pix/payments/continue?ref=x".into()),
            end_to_end_id: None,
            failure: None,
            created_at: "2026-10-22T15:31:02Z".into(),
            paid_at: None,
        }
    }

    #[test]
    fn adds_a_scope_once() {
        assert_eq!(
            with_scope("pfm:read pfm:transactions:write", PIX_SCOPE),
            "pfm:read pfm:transactions:write pix:payments:write"
        );
        assert_eq!(
            with_scope("pfm:read pix:payments:write", PIX_SCOPE),
            "pfm:read pix:payments:write"
        );
    }

    #[test]
    fn summarizes_the_pix_and_the_bank_link() {
        assert_eq!(
            summary(&pix("pending", true)),
            "Pix of R$ 20,00 to ***.456.789-** from Nubank"
        );
        assert!(summary(&pix("pending", false)).ends_with("First you link Nubank to Visor, once"));
    }

    #[test]
    fn draws_a_qr_code_with_a_quiet_zone() {
        let code = qr("https://dashboard.visorfinance.app/pix/payments/continue?ref=SFMyNTY").unwrap();
        let first = code.lines().next().unwrap();

        assert!(code.lines().count() > 10);
        assert!(first.chars().all(|c| c == '█'), "{first}");
    }

    #[test]
    fn explains_why_a_pix_was_not_paid() {
        assert!(not_paid_message(&pix("expired", true)).contains("start a new one"));
        let mut failed = pix("failed", true);
        failed.failure = Some(crate::models::PixPaymentFailure {
            code: "SALDO_INSUFICIENTE".into(),
            message: "Saldo insuficiente".into(),
        });
        assert_eq!(
            not_paid_message(&failed),
            "the Pix was not paid: Saldo insuficiente (SALDO_INSUFICIENTE)"
        );
    }
}
