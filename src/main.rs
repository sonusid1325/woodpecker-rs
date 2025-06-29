use anyhow::{Context, Result};
use colored::*;
use dotenvy::dotenv;
use lettre::message::{Mailbox, header::ContentType};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use polars::prelude::*;
use std::env;
use std::fs::File;
use std::io::Read;

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env
    dotenv().ok();

    // load CSV file
    let file = File::open("data.csv")?;
    let df = CsvReader::new(file)
        .infer_schema(Some(100))
        .has_header(true)
        .finish()?;

    // Load template
    let mut template_file = File::open("mail.txt").context("Cannot open main.txt")?;
    let mut template_content = String::new();
    template_file.read_to_string(&mut template_content)?;

    // Load config with context
    let smtp_host = env::var("SMTP_HOST").context("Missing SMTP_HOST")?;
    let smtp_username = env::var("SMTP_USERNAME").context("Missing SMTP_USERNAME")?;
    let smtp_password = env::var("SMTP_PASSWORD").context("Missing SMTP_PASSWORD")?;
    let from_email = env::var("FROM_EMAIL").context("Missing FROM_EMAIL")?;
    let display_name = env::var("DISPLAY_NAME").unwrap_or("Sender".to_string());

    // Parse emails safely
    let from_mailbox: Mailbox = format!("{} <{}>", display_name, from_email)
        .parse()
        .context("Invalid FROM_EMAIL format")?;

    println!(
        "From: {} \n Name: {}",
        from_mailbox.email.to_string().blue(),
        from_mailbox.name.clone().expect("No Name").blue()
    );

    let name_str = df.column("name")?.str()?;
    let email_str = df.column("email")?.str()?;
    let company_str = df.column("company")?.str()?;

    for (name, email_opt) in name_str
        .into_iter()
        .zip(email_str.into_iter().zip(company_str.into_iter()))
    {
        let name = name.unwrap_or("Friend");
        let email = email_opt.0.unwrap_or("");
        let company = email_opt.1.unwrap_or("");

        println!("Name: {}, Email: {}", name, email);

        let to_mailbox: Mailbox = email.parse().context("Invalid TO_EMAIL format")?;

        println!("To: {}", to_mailbox.email.to_string().purple());

        let email_body = template_content
            .replace("{name}", name)
            .replace("{company_name}", company)
            .replace("{sender_email}", &from_mailbox.email.to_string());

        let email = Message::builder()
            .from(from_mailbox.clone())
            .to(to_mailbox)
            .subject(format!("Hello its {}", display_name))
            .header(ContentType::TEXT_PLAIN)
            .body(email_body)
            .context("Failed to build the message")?;

        // SMTP CREDINTIALS
        let credintials = Credentials::new(smtp_username.clone(), smtp_password.clone());

        // Mailer
        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::relay(&smtp_host)
                .context("Failed to create SMTP transport")?
                .credentials(credintials)
                .build();

        match mailer.send(email).await {
            Ok(_) => println!("{}", "Email sent successfully".green()),
            Err(e) => eprintln!("Failed to send email: {}", e),
        }
    }

    Ok(())
}
