# ferromail

[![crates.io](https://img.shields.io/crates/v/ferromail.svg)](https://crates.io/crates/ferromail)
[![docs.rs](https://img.shields.io/docsrs/ferromail)](https://docs.rs/ferromail)
[![CI](https://github.com/ffakira/ferromail/actions/workflows/ci.yml/badge.svg)](https://github.com/ffakira/ferromail/actions/workflows/ci.yml)

Type-safe HTML email component builder for Rust.

Email HTML is its own dialect: table layouts, inline styles, Outlook conditional
comments. ferromail builds a markup tree you can't accidentally break, then
renders it to something mail clients actually display.

No runtime dependencies. Requires Rust 1.85.

## Status

Proof of concept, and the API will change.

What works today: the markup tree and renderer, the `html!` macro, a typed
`Stylesheet` for media queries, placeholders filled at render time, and five
components. `Document` emits the doctype and declares the VML namespaces,
`Container` is a centered fluid column, `Row` and `Column` lay out side by
side and stack on mobile, and `Button` has a VML branch for Outlook with a
table fallback for everyone else.

Missing before the component set is complete: `Text`, `Divider`, `Image` and
a preheader. Text in particular is unstyled today, so clients apply their own
defaults.

What is not established: **none of the output has been checked in a real email
client.** The generated client-support report scores the markup against the
caniemail dataset, but conditional comments are invisible to that check, so the
Outlook path in particular rests on reasoning rather than evidence. Treat the
Outlook support as untested.

The version published on crates.io predates most of the above.

## Example

```rust
use ferromail::components::{Button, Container, Document, Row};
use ferromail::html;
use ferromail::markup::{Color, Url, VarName};
use ferromail::render::{render_with, Bindings};

let order = Url::parse("https://example.com/orders/42?ref=a&b=2").expect("valid");

let body = Container::new()
    .background(Color::hex("#f4f4f5").expect("valid"))
    .children(html! {
        @(Row::single(html! {
            h1 { "Thanks, " {{ name }} }
            p { "We will email you when it ships." }
        }).build())
        @[Button::new(order, "View order")
            .background(Color::hex("#2563eb").expect("valid"))
            .build()]
    })
    .build();

let doc = Document::new("Thanks for your order").child(body).build();

let vars = Bindings::new().set(VarName::new("name").expect("valid"), "Ada & co");

println!("{}", render_with(&doc, &vars).expect("every name bound"));
```

Text is escaped on the way out, so `"View order <script>"` renders as
`View order &lt;script&gt;` rather than markup.

## The html! macro

`html!` builds a `Vec<Node>`. It is syntax only: every form below expands to the
builder call you would otherwise write by hand, so it opens no path the API does
not already allow.

| form | meaning |
|---|---|
| `tag { .. }` | an element with children |
| `tag;` | an element with none, which is also the form void tags take |
| `name=(expr)` | an attribute, parentheses always required |
| `"literal"` | escaped text |
| `(expr)` | escaped text from anything `Into<String>` |
| `{{ name }}` | a placeholder, filled by `render_with` |
| `@(expr)` | splices one node, so a component composes into a tree |
| `@[expr]` | splices many, which is what `Button::build` and `Document::build` return |

```rust
use ferromail::html;
use ferromail::markup::{ClassName, Url};

let url = Url::parse("https://example.com/logo.png").expect("valid");
let name = "Ada & <Lovelace>";

let nodes = html! {
    div class=(ClassName::new("stack").expect("valid")) {
        img src=(url) alt=("Logo") width=(120);
        p { "Hello, " (name) "!" }
    }
};
```

Attributes route by name. `href` and `src` go to `url_attr` and take a parsed
`Url`, `class` takes a `ClassName`, and everything else becomes an `AttrValue`
and is escaped on render.

Placeholders are substituted inside the escaping path, so a binding holding
markup becomes text rather than markup. They work in text positions only:
`href` and `style` values are validated when they are built, and a hole would
defeat that.

```rust
use ferromail::html;
use ferromail::markup::VarName;
use ferromail::render::{render_with, Bindings};

let nodes = html! { p { "Hi, " {{ name }} } };
let vars = Bindings::new().set(VarName::new("name").expect("valid"), "Ada & co");

assert_eq!(render_with(&nodes, &vars).expect("bound"), "<p>Hi, Ada &amp; co</p>");
```

The tag and attribute tables are closed, so a name outside them is a compile
error that says which one and what to do about it, rather than a macro matcher
failure:

```text
error: html!: unknown attribute `onclick`. Event handlers such as onclick have
no variant by design and never will. For anything else, add a variant to
markup::AttrName and an arm to __html_attr_name!. Note href and src are not
here: they take a parsed Url through url_attr.
```

## What is enforced

Untrusted strings cannot become markup. Every value that reaches an attribute
goes through a type that rejects anything able to close it, and the checks live
in the type rather than in a convention the caller has to remember:

- `Url` accepts `http`, `https`, `mailto` and `tel` only. There is no
  `AttrName::Href`, so the only route to `href` and `src` is `url_attr`, which
  demands a parsed `Url`. `javascript:` has no path in.
- `StyleValue` rejects CSS functions, comments and backslash escapes, so
  `expression()` and its encoded variants cannot appear in a `style` attribute.
- `Color::hex` is the only way to build a colour, which is what lets
  `Button::build` return nodes instead of a `Result`.
- `Stylesheet` is typed down to the selector, so `</style>` cannot be written
  from inside a `<style>` block.
- `RawHtml::trusted` is the single unescaped entry point, and it is named so it
  shows up in review.
- Placeholder values are substituted inside the escaping path, so `{{ name }}`
  holding `<script>` becomes text. A placeholder is not a hole in the escaper,
  which is why they exist only in text positions: `href` and `style` values are
  validated when built, and a hole would defeat that.

Outlook conditional comments are typed rather than strings, so `!mso` emits the
downlevel-revealed syntax instead of silently hiding its content.

The `html!` macro is syntax only. It expands to the same builder calls, so it
opens no path the API does not already allow.

## Not done yet

`Text`, `Divider`, `Image` and a preheader, then Tailwind-style utilities.

Media queries exist but are an enhancement: Gmail's app strips `<style>` for
non-Gmail accounts and Outlook ignores media queries, so `Row::stack` does
nothing there and the columns stay side by side. The fluid layout has to stand
on its own, which is what `Container` is for.

## Development

```sh
cargo test
docker compose up -d                                       # local SMTP catcher
cargo run --example send                                   # preview at :8025
cargo test --test client_support -- --ignored --nocapture  # client-support report
```

See [docs/PHILOSOPHY.md](https://github.com/ffakira/ferromail/blob/main/docs/PHILOSOPHY.md)
for why the crate is shaped this way.

## License

MIT or Apache-2.0, at your option.
