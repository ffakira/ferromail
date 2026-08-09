# Security

## Reporting

Report privately through GitHub's
[security advisory form](https://github.com/ffakira/ferromail/security/advisories/new)
rather than a public issue.

Expect a first response within a week. This is a spare-time project, so treat
that as best effort rather than a commitment.

## Supported versions

While the version is `0.0.x`, only the latest release gets fixes. There are no
backports.

## What ferromail is for

One property, stated as narrowly as it is meant:

> A string that came from outside the program cannot become markup.

"Outside the program" means data: an order number, a customer name, a link
target, a colour from a database. Every route such a value can take into the
output is gated by a type whose only constructor performs the check.

| route | gate |
|---|---|
| text | escaped at render |
| attribute values | escaped at render |
| `href`, `src` | `Url`, allowlisting `http`, `https`, `mailto`, `tel`. There is no `AttrName::Href`, so `url_attr` is the only route |
| `style` values | `StyleValue`, rejecting CSS functions, comments and backslash escapes |
| `class` | `ClassName`, rejecting quotes and angle brackets |
| `<style>` blocks | `Stylesheet`, typed to the selector, so `</style>` cannot occur |
| conditional comments | `Condition`, so a condition cannot close the comment early |

The crate forbids `unsafe` (`unsafe_code = "forbid"`) and has no runtime
dependencies, so its supply chain is itself.

## What ferromail is not for

These are design decisions, not gaps to be reported.

**It does not sanitize HTML.** ferromail *constructs* safe markup. It has no
parser and cannot take an arbitrary HTML string and make it safe. If you need
that, you need a sanitizer, and you need it before the value reaches ferromail.

**`RawHtml::trusted` is a deliberate hole.** It is the one way to put unescaped
markup into a document, and it is named so the claim shows up in review.
Passing user input to it is an injection vulnerability. That is the caller's
decision, not a crate defect.

**It does not defend against the template author.** The threat model is
untrusted *data*, not untrusted *code*. Anyone who can write Rust against this
API can call `RawHtml::trusted`, and no library can prevent that.

**It does not send email.** No SMTP, no headers, no SPF, DKIM or DMARC. Those
belong to whatever you hand the rendered string to.

**Rendering correctness is not a security property.** ferromail will happily
render a convincing phishing email or a tracking pixel if you build one. What
you say is yours.

## Panics

No panic in the library is reachable from caller-supplied data. Every `expect`
in non-test code is on a value the crate itself owns:

- `components::prop` and `components::decl` are `pub(crate)` and called only
  with string literals written in this repository.
- `Button::new` builds its defaults from literals.
- `Color::style_value` cannot fail, because `Color::hex` is the only
  constructor and hex digits are all legal in a declaration value.
- `MediaQuery::write` writes to a `String`, which is infallible.

A panic reachable from data a caller supplied **is** a vulnerability. Please
report it.

## Known unverified areas

Stated plainly because a crate that sounds this confident invites assumptions
it has not earned.

**Nothing has been checked in a real email client.** The Outlook path exists on
reasoning, not evidence. The generated client-support report scores markup
against the caniemail dataset, and conditional comments are invisible to that
check, so the VML branch is not covered by it either.

**The validators are stricter than the spec on purpose.** No vendor prefixes,
no CSS functions, no quoted font names. If something legitimate is
unrepresentable, that is a feature request, not a vulnerability.

## In scope

- Caller-supplied data reaching the output unescaped without `RawHtml::trusted`
- A value passing `Url::parse`, `StyleValue::parse`, `ClassName::new`,
  `Property::new` or `Color::hex` that can still close its attribute, element
  or declaration
- Anything letting an attribute, tag, comment or `<style>` block be closed from
  inside a value
- A panic reachable from caller-supplied data

## Out of scope

- Anything requiring `RawHtml::trusted`
- Rendering wrong in a mail client, which is a bug, so open an issue
- A validator rejecting legitimate CSS, which is a feature request
- Content-level concerns such as phishing, tracking or deliverability
