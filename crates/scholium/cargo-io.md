(n) a marginal note or explanatory comment made by a scholiast.

User-defined `clippy`-like structured self-reported annotations.

## Introduction

There's multiple situations, when automatic reporting is not possible.
In these cases, developers tend to use various TODO comments like `TODO`, `REVIEW`,
`HACKME`, `XXX` and so on.

There's methods to extract and highlight existing TODOs, but all of them work
differently, as a comment doesn't tell exactly, where TODO should be finished
and in which cases it should be continued.

On the other hand `scholium::mark` annotation replaces all kinds of TODOs with
Rust-native way, allowing developers to categorize them with better precision,
write better reasoning and make them easy to extract, report and maintain in the long run.
Thanks to Rust literal string boundaries, it's possible to write long multiline reasons,
for example allowing to attach even code snippets if needed.

Annotation is made to be attached to an exact place and it provides only documentation
value without altering an item attached in any way. This means if the annotation
when be attached to a function, module, crate (or a statement), it documents this
particular item and nothing else.

## Usage

*Currently supported syntax:*

* An optional severity (see [`schollium_core::Severity`] enum for reference)
* One or more report identifiers (Rust non-absolute path with exactly 2-segments).
* None of identifiers may start with underscore character (`_`) due to Rust semantics.
* Optional `see_also` messages, value is a string literal which won't be interpreted.
* Single mandatory `reason` message for an entry, describing this annotation instance.

*Important Notes:*

* `see_also` and `reason` keys might be mixed, but `reason` must present.
* Full name `scholium::mark` is a canonical way to define this attribute, expected by reference implementation.

*Notable differences with `cargo clippy`:*
* Annotation is designed for developers themselves to report problems (opt-in)
  even without any analysis automation.
* Severity is the same as logging severity.
* An optional severity allows to override default severity level defined for
  specific report item.
* Category is fully user-defined and split into a group and category.
  This way it's simple to organize categories and describe them in files.
  User is responsible to manage these categories.
* Reason field is mandatory to help further readers with better understanding
  of code and problem marked.
* Annotation must be placed in outer placed before
  [custom inner attributes](https://github.com/rust-lang/rust/issues/54726)
  feature is there.
  It will be referred as `54726` in example below.


## Usage example

```rust
//! Utility functions for starship launching.
// wait for 54726 for this to work
// #![scholium::mark(doc::misleading, reason = "Module purpose was changed")]

/// Add two numbers.
#[scholium::mark(
    doc::missing_examples,
    reason = "Function documentation doesn't contain examples"
)]
#[scholium::mark(warning, test::missing, reason = "Function missing tests")]
#[scholium::mark(
    info,
    implementation::extend,
    doc::possible_extensions,
    reason = "Extend functionality to use any type implementing `+` operation"
)]
#[scholium::mark(
    third_party::missing_rust_feature,
    reason = "place scholium::mark in inner context",
    see_also = "https://github.com/rust-lang/rust/issues/54726"
)]
fn add(a: u32, b: u32) -> u32 {
   // wait for 54726 for this to work
   // #[scholium::mark(
   //       trace,
   //       int:check-missing,
   //       reason = "Use better addition if overflow is the case"
   // )]
   a + b
}
```

For the example above following information will be reported by a companion tool:

* (after 54726) module (exact) with category `doc::misleading`, reason `Module purpose was changed` with default severity.
* function `add` with category `doc::missing_examples`, reason `Function documentation doesn't contain examples` with default severity.
* function `add` with category `test::missing`, reason `Function missing tests` with `warning` severity.
* function `add` with category `implementation::extend`, reason `Extend functionality to use any type implementing `+` operation` with `info` severity.
* function `add` with category `doc::possible_extensions`, reason `Extend functionality to use any type implementing `+` operation` with `info` severity.
* function `add` with category `third_party::missing_rust_feature`, see-also "https://github.com/rust-lang/rust/issues/54726" and reason "place scholium::mark in inner context", with default severity.
* (after 54726) sentence `a+b` with category `int:check-missing`, reason `Use better addition if overflow is the case` with `debug` severity.

## MSRV

MSRV is `1.71`
