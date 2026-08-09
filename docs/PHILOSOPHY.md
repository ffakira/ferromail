# Philosophy

ferromail exists to make one class of bug impossible: a string that came from
outside the program becoming markup inside the email.

Everything else in this document follows from that.

## Validation is a type, not a habit

A function that takes `&str` and promises to be careful is a habit. Habits
survive until someone adds a second call site at 2am.

So validation lives in a type. `Url`, `Color`, `ClassName`, `Property` and
`StyleValue` each hold a string that has already been checked, and each has
exactly one constructor. Once you hold one, the check has happened. The
renderer can write `StyleValue` into a `style` attribute without escaping it,
not because the renderer is careful but because the value could not have been
constructed otherwise.

This is also why `Color` exists rather than passing hex strings around.
`Color::hex` is the only way to make one, so `Color::style_value` is
infallible, so `Button::build` returns `Vec<Node>` instead of a `Result`. One
type moved a runtime error into the type system and simplified two layers
above it.

## Validate at the bottom, compose above

`markup` validates. `render` serialises. `components` compose. Higher layers
never re-implement a check, and never accept a raw string where a validated
type would do.

When that rule is followed, an audit is small. To know whether hostile input
can reach a `href`, you read `Url::parse` and confirm that `UrlAttr` is the
only route to the attribute. You do not read every component.

`UrlAttr` being a separate enum from `AttrName` is this rule made structural.
There is no `AttrName::Href`, so no code path exists that puts an unparsed
string into an `href`.

## Allowlists, never blocklists

Blocking `javascript:` invites `vbscript:`, then `data:text/html`, then
whatever the next one is. The blocklist is always one entry behind an
attack and requires a code change every time the world changes.

`Url::parse` accepts `http`, `https`, `mailto` and `tel`. Nothing else. A new
scheme is rejected by default and adding one is a deliberate act.

The same shape appears in `Tag` and `AttrName`, which are closed enums with no
`Other(String)` escape hatch. If a tag is not in the enum it cannot be
rendered.

## Normalise before you inspect

Checks that read a string must first remove the things that let a string lie
about itself.

`Url::parse` strips control characters before it looks at the scheme, because
a tab inside `java<tab>script:` is discarded by some clients and the string
they end up parsing is not the one we checked.

`StyleValue` rejects backslash outright for the same reason. CSS lets you
write `\65 xpression(...)` and mean `expression(...)`, so any check on the
function name loses to an encoded one. No legitimate email style value
contains a backslash, so banning the character removes the whole class rather
than playing the escaping game.

## Reject more than the spec allows

`StyleValue` accepts no CSS functions at all. Not `calc`, not `rgba`, not
`var`. That is stricter than CSS and stricter than most email clients.

Two reasons. The narrow one is safety: `expression()` was IE's scripting
vector and `url()` still reaches the network, so a rule of "no parentheses"
is one line and admits no argument. The broader one is honesty. Outlook's
Word engine ignores `calc` and `var`, so accepting that means the type says
"fine" about values that silently render wrong in the client the crate exists
to survive.

`url()` is still reachable, through `StyleValue::url`, which takes an
already-parsed `Url`. The exception is a named door rather than a hole.

Strict defaults are cheap to relax later. A permissive default cannot be
tightened without breaking everyone.

## Make the dangerous thing loud

There is exactly one way to put unescaped markup into a document, and it is
called `RawHtml::trusted`. Not `new`, not `from`. The call site reads as a
claim the caller is making, and it greps in one command.

`Node::Conditional` was originally a `String` condition. It looked safe and
was not: a condition containing `]>` closes the comment early and everything
after it renders live. It is now a `Condition` enum, which also let the
renderer emit the downlevel-revealed form for `!mso` that a string could
never have signalled.

If something can only be used correctly, it should be a type. If it must be
used carefully, it should be named so that carelessness is visible in review.

## Panics are allowed only over crate-owned input

`components::prop` and `decl` call `expect`. That is sound because they are
`pub(crate)` and called only with string literals written in this repository.
A panic there is a typo, caught by our own tests, and no consumer input can
reach it.

The moment such a helper becomes public, or starts receiving caller data, it
must return a `Result` instead. This is the boundary that keeps `expect` from
becoming a denial of service.


## Client compatibility is correctness

An email that renders wrong in Outlook is broken, not imperfect. So decisions
that look like style are treated as correctness:

- The button's fallback is a table with the colour on a `td`, because several
webmail clients drop background and padding on inline elements. Outlook
desktop never sees it: it gets a VML shape from the `mso` branch instead
- Colours are written twice, as a `bgcolor` attribute and a declaration,
because some clients honour the attribute and ignore the CSS, and others do
the reverse
- Layout tables carry `role="presentation"`, so a screen reader does not
announce a button as a data table
- Style declarations are joined without spaces because Gmail clips messages
over 102KB and hides everything after the cut

Where a client quirk is encoded, the reason is written next to it. The quirk
outlives everyone's memory of why.

## What ferromail is not

It is not a general HTML library. It renders the narrow, ancient dialect that
email clients accept, and refuses much of what a browser would happily eat.

It is not a templating engine. There is no string interpolation into markup,
because that is the mechanism this crate exists to remove.

It has no runtime dependencies, and adding one needs a better reason than
convenience.
