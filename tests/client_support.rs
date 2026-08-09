//! Email-client support report, generated from real fixtures.
//!
//! This is *not* code coverage. It renders each fixture, posts it to a local
//! Mailpit instance, and asks Mailpit's `html-check` endpoint which email
//! clients support the HTML and CSS actually emitted. The data comes from
//! caniemail.com.
//!
//! Ignored by default, because it needs a container:
//!
//! ```text
//! docker compose up -d
//! cargo test --test client_support -- --ignored --nocapture
//! # writes target/client-support.md
//! ```
//!
//! The Mailpit image is pinned in `docker-compose.yml` because it bundles the
//! caniemail dataset these scores come from. A different image version moves
//! the numbers without any change to this crate.
//!
//! # A blind spot you must not forget
//!
//! `html-check` parses the DOM, and a downlevel-hidden conditional comment
//! (`<!--[if mso]> ... <![endif]-->`) is just a comment to a parser. So the
//! VML branch (the entire reason the Outlook path exists) is **never
//! analysed**, and any "Outlook: partial" verdict you see refers to the
//! `!mso` fallback that Outlook never renders. Treat this report as evidence
//! about non-Outlook clients only. Outlook still needs Litmus or a real client.

use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::TcpStream;

use ferromail::components::{Button, Document};
use ferromail::markup::{Color, Element, Node, Tag, Url};
use ferromail::render::render;
use lettre::message::{MultiPart, SinglePart, header};
use lettre::{Message, SmtpTransport, Transport};
use serde_json::Value;

const SMTP_HOST: &str = "localhost";
const SMTP_PORT: u16 = 1025;
const API_HOST: &str = "localhost";
const API_PORT: u16 = 8025;

/// Total unsupported share, above which the report is treated as a regression.
/// Coarse on purpose: the caniemail dataset moves, so a tight threshold would
/// fail for reasons that have nothing to do with this crate.
const MAX_UNSUPPORTED_PCT: f64 = 20.0;

struct Fixture {
    name: &'static str,
    html: String,
}

fn url(raw: &str) -> Url {
    Url::parse(raw).expect("fixture url is valid")
}

fn fixtures() -> Vec<Fixture> {
    let button = || {
        Button::new(url("https://example.com/confirm"), "Confirm your email")
            .background(Color::hex("#2563eb").expect("valid"))
            .size(240, 48)
            .radius(6)
    };

    vec![
        Fixture {
            name: "document-empty",
            html: render(&[Document::new("Empty").build()]),
        },
        Fixture {
            name: "document-with-button",
            html: render(&[Document::new("Confirm").children(button().build()).build()]),
        },
        Fixture {
            name: "button-bare",
            html: render(&button().build()),
        },
        Fixture {
            name: "table-layout",
            html: render(&[Document::new("Table")
                .child(Node::Element(
                    Element::new(Tag::Table).child(Node::Element(
                        Element::new(Tag::Tr)
                            .child(Node::Element(Element::new(Tag::Td).text("cell"))),
                    )),
                ))
                .build()]),
        },
    ]
}

/// Minimal HTTP/1.0 GET. Localhost only, and `Connection: close` gives a clean
/// EOF, so this needs no HTTP crate.
fn get(path: &str) -> Result<String, std::io::Error> {
    let mut stream = TcpStream::connect((API_HOST, API_PORT))?;
    write!(
        stream,
        "GET {path} HTTP/1.0\r\nHost: {API_HOST}:{API_PORT}\r\nConnection: close\r\n\r\n"
    )?;

    let mut raw = String::new();
    stream.read_to_string(&mut raw)?;

    Ok(raw
        .split_once("\r\n\r\n")
        .map(|(_, body)| body.to_owned())
        .unwrap_or(raw))
}

fn send(fixture: &Fixture) -> String {
    let subject = format!("ferromail-coverage-{}", fixture.name);

    let email = Message::builder()
        .from("ferromail <dev@example.com>".parse().expect("valid"))
        .to("coverage <coverage@example.com>".parse().expect("valid"))
        .subject(subject.clone())
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_PLAIN)
                        .body(String::from("plain text alternative")),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(fixture.html.clone()),
                ),
        )
        .expect("fixture builds");

    SmtpTransport::builder_dangerous(SMTP_HOST)
        .port(SMTP_PORT)
        .build()
        .send(&email)
        .unwrap_or_else(|e| panic!("mailpit unreachable on {SMTP_HOST}:{SMTP_PORT}: {e}"));

    let found = get(&format!("/api/v1/search?query=subject:{subject}")).expect("search request");
    let found: Value = serde_json::from_str(&found).expect("search json");

    found["messages"][0]["ID"]
        .as_str()
        .unwrap_or_else(|| panic!("no message found for subject {subject}"))
        .to_owned()
}

struct Report {
    name: &'static str,
    bytes: usize,
    supported: f64,
    partial: f64,
    unsupported: f64,
    /// Feature title -> clients with no support at all.
    failures: Vec<(String, Vec<String>)>,
}

fn check(fixture: &Fixture, id: &str) -> Report {
    let body = get(&format!("/api/v1/message/{id}/html-check")).expect("html-check request");
    let v: Value = serde_json::from_str(&body).expect("html-check json");

    let pct = |k: &str| v["Total"][k].as_f64().unwrap_or_default();

    let mut failures = Vec::new();
    for w in v["Warnings"].as_array().into_iter().flatten() {
        let mut no: Vec<String> = w["Results"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|r| r["Support"].as_str() == Some("no"))
            .filter_map(|r| r["Name"].as_str().map(str::to_owned))
            .collect();

        if no.is_empty() {
            continue;
        }

        no.sort();
        no.dedup();
        failures.push((w["Title"].as_str().unwrap_or("?").to_owned(), no));
    }

    Report {
        name: fixture.name,
        bytes: fixture.html.len(),
        supported: pct("Supported"),
        partial: pct("Partial"),
        unsupported: pct("Unsupported"),
        failures,
    }
}

fn markdown(reports: &[Report]) -> String {
    let mut out = String::new();
    out.push_str("# Email-client support\n\n");
    out.push_str(
        "Generated by `cargo test --test client_support -- --ignored`, \
         from Mailpit's `html-check` (caniemail.com data).\n\n",
    );
    out.push_str("> **Conditional comments are invisible to this report.**\n");
    out.push_str(
        "> The `mso` branch is a downlevel-hidden HTML comment, so the VML\n\
         > that exists for Outlook is never analysed, and any Outlook verdict\n\
         > below refers to the `!mso` fallback Outlook does not render. This\n\
         > is evidence about non-Outlook clients only.\n\n",
    );
    out.push_str("| fixture | bytes | supported | partial | unsupported |\n");
    out.push_str("|---|--:|--:|--:|--:|\n");

    // Writing to a String is infallible; the Results are only there because
    // `write!` is generic over `fmt::Write`.
    for r in reports {
        writeln!(
            out,
            "| {} | {} | {:.1}% | {:.1}% | {:.1}% |",
            r.name, r.bytes, r.supported, r.partial, r.unsupported
        )
        .expect("writing to a String cannot fail");
    }

    for r in reports {
        if r.failures.is_empty() {
            continue;
        }
        writeln!(out, "\n## {}: unsupported\n", r.name).expect("infallible");
        for (feature, clients) in &r.failures {
            writeln!(out, "- **{}**: {}", feature, clients.join(", ")).expect("infallible");
        }
    }

    out
}

#[test]
#[ignore = "needs a local mailpit: docker start mailpit"]
fn client_support_report() {
    let reports: Vec<Report> = fixtures()
        .iter()
        .map(|f| {
            let id = send(f);
            check(f, &id)
        })
        .collect();

    let md = markdown(&reports);
    std::fs::create_dir_all("target").expect("target dir");
    std::fs::write("target/client-support.md", &md).expect("write report");

    println!("{md}");
    println!("wrote target/client-support.md");

    let worst = reports
        .iter()
        .max_by(|a, b| a.unsupported.total_cmp(&b.unsupported))
        .expect("at least one fixture");

    assert!(
        worst.unsupported <= MAX_UNSUPPORTED_PCT,
        "{} is {:.1}% unsupported, over the {:.1}% budget",
        worst.name,
        worst.unsupported,
        MAX_UNSUPPORTED_PCT
    );
}
