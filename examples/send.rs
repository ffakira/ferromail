//! Renders a sample email, writes it to `target/preview.html`, and posts it to
//! a local SMTP catcher.
//!
//! ```text
//! docker compose up -d
//! cargo run --example send
//! # open http://localhost:8025
//! ```
//!
//! What this proves: that the rendered HTML survives real MIME assembly and
//! quoted-printable encoding. `render` emits no newlines, so a whole document
//! is one very long line, and SMTP caps lines at 1000 characters, so the encoder
//! has to fold it. That is the failure this loop is here to catch.
//!
//! What it does NOT prove: how Outlook renders anything. Mailpit's viewer is a
//! browser, and Outlook on Windows uses Word's engine. The VML branch is
//! invisible here. Use Litmus or a real client for that.

use std::fs;

use ferromail::components::{Button, Document};
use ferromail::markup::{Color, Url};
use ferromail::render::render;
use lettre::message::{MultiPart, SinglePart, header};
use lettre::{Message, SmtpTransport, Transport};

const SMTP: &str = "localhost";
const SMTP_PORT: u16 = 1025;

fn sample() -> String {
    let href = Url::parse("https://example.com/confirm?token=abc123&next=/welcome")
        .expect("sample url is valid");

    let button = Button::new(href, "Confirm your email")
        .background(Color::hex("#2563eb").expect("valid"))
        .color(Color::hex("#ffffff").expect("valid"))
        .size(240, 48)
        .radius(6);

    render(
        &Document::new("Confirm your email")
            .children(button.build())
            .build(),
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let html = sample();

    println!(
        "rendered {} bytes on {} line(s)",
        html.len(),
        html.lines().count()
    );

    // Cheapest feedback loop of all: no SMTP, just look at it.
    fs::create_dir_all("target")?;
    fs::write("target/preview.html", &html)?;
    println!("wrote target/preview.html");

    // A text/plain alternative is not optional in practice: clients that
    // cannot render HTML, and some spam filters, expect one.
    let plain = "Confirm your email: https://example.com/confirm?token=abc123&next=/welcome";

    let email = Message::builder()
        .from("ferromail <dev@example.com>".parse()?)
        .to("inbox <inbox@example.com>".parse()?)
        .subject("Confirm your email")
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(plain.to_owned()),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(html),
                ),
        )?;

    let mailer = SmtpTransport::builder_dangerous(SMTP)
        .port(SMTP_PORT)
        .build();

    match mailer.send(&email) {
        Ok(_) => println!("sent to {SMTP}:{SMTP_PORT}, open http://localhost:8025"),
        Err(e) => {
            eprintln!("could not send to {SMTP}:{SMTP_PORT}: {e}");
            eprintln!("is mailpit running? docker compose up -d");
            return Err(e.into());
        }
    }

    Ok(())
}
