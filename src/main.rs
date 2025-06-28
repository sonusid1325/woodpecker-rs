use anyhow::{Context, Result};
use dotenvy::dotenv;
use lettre::message::{Mailbox, header::ContentType};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use polars::prelude::*;
use std::env;
use std::fs::File;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env
    dotenv().ok();

    // load CSV file
    let file = File::open("data.csv")?;
    let df = CsvReader::new(file)
        .infer_schema(Some(100)) // scan first 100 rows for schema
        .has_header(true)
        .finish()?;

    // Load config with context
    let smtp_host = env::var("SMTP_HOST").context("Missing SMTP_HOST")?;
    let smtp_username = env::var("SMTP_USERNAME").context("Missing SMTP_USERNAME")?;
    let smtp_password = env::var("SMTP_PASSWORD").context("Missing SMTP_PASSWORD")?;
    let from_email = env::var("FROM_EMAIL").context("Missing FROM_EMAIL")?;
    let to_email = env::var("TO_EMAIL").context("Missing TO_EMAIL")?;
    let display_name = env::var("DISPLAY_NAME").unwrap_or("Sender".to_string());

    // Parse emails safely
    let from_mailbox: Mailbox = format!("{} <{}>", display_name, from_email)
        .parse()
        .context("Invalid FROM_EMAIL format")?;
    let to_mailbox: Mailbox = to_email.parse().context("Invalid TO_EMAIL format")?;

    println!("From: {:?}", from_mailbox);
    println!("To: {:?}", to_mailbox);

    // Prepare email
    let email = Message::builder()
        .from(from_mailbox)
        .to(to_mailbox)
        .subject(format!("Hello its {}", display_name))
        .header(ContentType::TEXT_PLAIN)
        .body(format!("Myself {}", display_name))
        .context("Failed to build the message")?;

    // SMTP CREDINTIALS
    let credintials = Credentials::new(smtp_username, smtp_password);

    // Mailer
    let mailer: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host)
            .context("Failed to create SMTP transport")?
            .credentials(credintials)
            .build();

    // match mailer.send(email).await {
    //     Ok(_) => println!("Email sent successfully"),
    //     Err(e) => eprintln!("Failed to send email: {}", e),
    // }
    println!("{}", df);

    Ok(())
}
