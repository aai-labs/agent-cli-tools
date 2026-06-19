use std::{
    net::TcpStream,
    time::{SystemTime, UNIX_EPOCH},
};

use lettre::{
    message::Mailbox, transport::smtp::authentication::Credentials, Message, SmtpTransport,
    Transport,
};
use rustls_connector::RustlsConnector;
use serde_json::{json, Value};

use crate::{cli::*, config::Context, error::AppError, services::shared::CtxProfile};

pub(crate) async fn messages(
    ctx: &Context,
    action: EmailMessagesAction,
) -> Result<Value, AppError> {
    match action {
        EmailMessagesAction::List(args) => list(ctx, &args),
        EmailMessagesAction::Get(args) => get(ctx, &args.id),
        EmailMessagesAction::Send(args) => send(ctx, args),
        EmailMessagesAction::Delete(args) => delete(ctx, &args.id),
    }
}

fn list(ctx: &Context, args: &EmailMessageList) -> Result<Value, AppError> {
    let mut criteria_parts: Vec<String> = Vec::new();
    if let Some(after) = &args.received_after {
        criteria_parts.push(format!(
            "SINCE {}",
            date_to_imap(after, "messages.list")?
        ));
    }
    if let Some(before) = &args.received_before {
        criteria_parts.push(format!(
            "BEFORE {}",
            date_to_imap(before, "messages.list")?
        ));
    }
    let query = if criteria_parts.is_empty() {
        "ALL".to_string()
    } else {
        criteria_parts.join(" ")
    };
    let mut session = imap_session(ctx, &folder(ctx, false))?;
    let mut ids = session
        .uid_search(query)
        .map_err(|err| AppError::internal("email", "messages.list", err.to_string()))?
        .into_iter()
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids.reverse();
    ids.truncate(args.limit as usize);
    let _ = session.logout();
    Ok(json!({
        "transport": "smtp_imap",
        "folder": folder(ctx, false),
        "ids": ids,
    }))
}

fn date_to_imap(s: &str, op: &'static str) -> Result<String, AppError> {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let date = s.split('T').next().unwrap_or(s);
    let parts: Vec<u32> = date
        .split('-')
        .map(|p| p.parse::<u32>())
        .collect::<Result<_, _>>()
        .map_err(|_| AppError::invalid_input("email", op, format!("invalid date: {s}")))?;
    if parts.len() != 3 || parts[1] < 1 || parts[1] > 12 {
        return Err(AppError::invalid_input(
            "email",
            op,
            format!("invalid date: {s}"),
        ));
    }
    Ok(format!(
        "{:02}-{}-{}",
        parts[2],
        MONTHS[(parts[1] - 1) as usize],
        parts[0]
    ))
}

fn get(ctx: &Context, id: &str) -> Result<Value, AppError> {
    let mut session = imap_session(ctx, &folder(ctx, false))?;
    let messages = session
        .uid_fetch(id, "RFC822")
        .map_err(|err| AppError::internal("email", "messages.get", err.to_string()))?;
    let message = messages.iter().next().ok_or_else(|| {
        AppError::not_found("email", "messages.get", format!("message {id} not found"))
    })?;
    let raw = message
        .body()
        .map(|body| String::from_utf8_lossy(body).to_string())
        .unwrap_or_default();
    let _ = session.logout();
    Ok(json!({
        "transport": "smtp_imap",
        "id": id,
        "raw": raw,
    }))
}

fn delete(ctx: &Context, id: &str) -> Result<Value, AppError> {
    let mut session = imap_session(ctx, &folder(ctx, false))?;
    session
        .uid_store(id, "+FLAGS.SILENT (\\Deleted)")
        .map_err(|err| AppError::internal("email", "messages.delete", err.to_string()))?;
    let expunged = session
        .expunge()
        .map_err(|err| AppError::internal("email", "messages.delete", err.to_string()))?;
    let _ = session.logout();
    Ok(json!({
        "transport": "smtp_imap",
        "id": id,
        "deleted": true,
        "expunged": expunged,
    }))
}

fn send(ctx: &Context, args: EmailSend) -> Result<Value, AppError> {
    let to = args.to.ok_or_else(|| {
        AppError::invalid_input("email", "messages.send", "--to is required for smtp_imap")
    })?;
    let subject = args.subject.unwrap_or_else(|| "(no subject)".to_string());
    let content = args.body.unwrap_or_default();
    let from = from_address(ctx)?;
    let message_id = format!("<aai-cli-{}@local>", unique_id());

    let email = Message::builder()
        .from(parse_mailbox(&from, "from")?)
        .to(parse_mailbox(&to, "to")?)
        .subject(subject)
        .message_id(Some(message_id.clone()))
        .body(content)
        .map_err(|err| AppError::invalid_input("email", "messages.send", err.to_string()))?;

    let smtp_host = ctx
        .profile()
        .smtp_host
        .as_deref()
        .unwrap_or("smtp.zoho.com");
    let smtp_port = ctx.profile().smtp_port.unwrap_or(465);
    let credentials = Credentials::new(username(ctx)?.to_string(), password(ctx)?);
    let mailer = if smtp_port == 465 {
        SmtpTransport::relay(smtp_host)
    } else {
        SmtpTransport::starttls_relay(smtp_host)
    }
    .map_err(|err| AppError::internal("email", "messages.send", err.to_string()))?
    .port(smtp_port)
    .credentials(credentials)
    .build();

    mailer
        .send(&email)
        .map_err(|err| AppError::internal("email", "messages.send", err.to_string()))?;

    append_to_sent(ctx, &email.formatted())?;
    let found_id = search_message_id(ctx, &message_id).ok().flatten();
    Ok(json!({
        "transport": "smtp_imap",
        "sent": true,
        "id": found_id,
        "message_id": message_id,
    }))
}

fn append_to_sent(ctx: &Context, raw: &[u8]) -> Result<(), AppError> {
    let sent_folder = folder(ctx, true);
    let mut session = imap_session(ctx, &sent_folder)?;
    session
        .append(&sent_folder, raw)
        .map_err(|err| AppError::internal("email", "messages.send", err.to_string()))?;
    let _ = session.logout();
    Ok(())
}

fn search_message_id(ctx: &Context, message_id: &str) -> Result<Option<u32>, AppError> {
    let mut session = imap_session(ctx, &folder(ctx, true))?;
    let query = format!("HEADER Message-ID {}", quote_imap(message_id));
    let mut ids = session
        .uid_search(query)
        .map_err(|err| AppError::internal("email", "messages.send", err.to_string()))?
        .into_iter()
        .collect::<Vec<_>>();
    ids.sort_unstable();
    let _ = session.logout();
    Ok(ids.pop())
}

fn imap_session(
    ctx: &Context,
    folder: &str,
) -> Result<imap::Session<rustls_connector::TlsStream<TcpStream>>, AppError> {
    let imap_host = ctx
        .profile()
        .imap_host
        .as_deref()
        .unwrap_or("imap.zoho.com");
    let imap_port = ctx.profile().imap_port.unwrap_or(993);
    let stream = TcpStream::connect((imap_host, imap_port))
        .map_err(|err| AppError::internal("email", "imap", err.to_string()))?;
    let tls = RustlsConnector::new_with_webpki_roots_certs();
    let tls_stream = tls
        .connect(imap_host, stream)
        .map_err(|err| AppError::internal("email", "imap", err.to_string()))?;
    let mut client = imap::Client::new(tls_stream);
    client
        .read_greeting()
        .map_err(|err| AppError::internal("email", "imap", err.to_string()))?;
    let password = password(ctx)?;
    let mut session = client
        .login(username(ctx)?, &password)
        .map_err(|(err, _)| AppError::auth("email", "imap", err.to_string()))?;
    session
        .select(folder)
        .map_err(|err| AppError::internal("email", "imap", err.to_string()))?;
    Ok(session)
}

fn username(ctx: &Context) -> Result<&str, AppError> {
    ctx.profile()
        .username
        .as_deref()
        .or(ctx.profile().email.as_deref())
        .ok_or_else(|| AppError::auth("email", "smtp_imap", "profile is missing username or email"))
}

fn password(ctx: &Context) -> Result<String, AppError> {
    let password = ctx
        .profile()
        .password
        .as_deref()
        .or(ctx.profile().api_token.as_deref())
        .or(ctx.profile().token.as_deref())
        .ok_or_else(|| AppError::auth("email", "smtp_imap", "profile is missing password"))?;
    Ok(normalize_app_password(ctx, password))
}

fn normalize_app_password(ctx: &Context, value: &str) -> String {
    if matches!(
        ctx.profile().auth_type.as_deref(),
        Some("app_password" | "app-password")
    ) {
        value.split_whitespace().collect()
    } else {
        value.to_string()
    }
}

fn from_address(ctx: &Context) -> Result<String, AppError> {
    ctx.profile()
        .from_address
        .as_deref()
        .or(ctx.profile().email.as_deref())
        .or(ctx.profile().username.as_deref())
        .map(ToString::to_string)
        .ok_or_else(|| {
            AppError::auth(
                "email",
                "messages.send",
                "profile is missing from_address, email, or username",
            )
        })
}

fn folder(ctx: &Context, sent: bool) -> String {
    if sent {
        ctx.profile()
            .sent_folder
            .clone()
            .unwrap_or_else(|| "Sent".to_string())
    } else {
        ctx.profile()
            .mail_folder
            .clone()
            .unwrap_or_else(|| "INBOX".to_string())
    }
}

fn parse_mailbox(value: &str, field: &'static str) -> Result<Mailbox, AppError> {
    value.parse().map_err(|err| {
        AppError::invalid_input(
            "email",
            "messages.send",
            format!("invalid {field} address: {err}"),
        )
    })
}

fn quote_imap(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn unique_id() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before epoch")
        .as_millis()
}
