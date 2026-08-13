# User-defined `clippy`-like structured code annotations.

(n) a marginal note or explanatory comment made by a scholiast.

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

See more information about usage `scholium::mark` in code in the [official documentation](https://docs.rs/scholium).

## Basic usage

Tool do not attempt any analysis and reports what a user already defined in code.

* List project-local reports ids and their titles with `cargo scholium list`
* Explain project-local reports ids with `cargo scholium explain <report_id>`
* Collect and report `scholium::mark` attributes from code with `cargo scholium report`

*Example output:*

```
 INFO Future implementation
  --> src/file.rs:90:18
  = reason: user defined reason
  = info: Implement in the future
  = help: for further information run `cargo scholium explain group::future_imp`
```

## Configuration

### Known reports

Known report ids are located in project under `.config/scholium` folder.
All definitions are located in TOML files and look like below. Filename without
  extension is treated as a group identifier.

*Example report id definition:*

Configuration below will define `group::future_imp` if put in
  `.config/scholium/group.toml` inside a project.

```toml
display-name = "Group display name"

[report.future_imp]
severity = "info" # other values: trace, debug, warning, error or suppress
display-name = "Future implementation"
info = "Implement in the future" # information
documentation = """
Non-implemented feature.

## Recommended action

Implement requested feature.
""" # documentation is shown for explain.
```

### Profiles

Profiles are pre-defined configuration setup, could be used during an execution.
Profiles are defined in `.config/scholium.toml` file and may have any name.

To specify profile, use `--profile` in CLI or environment variable `CARGO_SCHOLIUM_PROFILE`


*Example profile definition:*

```toml
[profile.example]
format = "human" # Human readable output
detail = "compact" # write compact details
unknown = "trace" # unknown report ids will be reported at `trace` level (by default).
output = "target/.scholium.log" # write whole output to this file. Default is stdout.

[profile.example.overrides]
# all reports for group `third_party` will be reported with `info` level (by default).
"third_party" = "info"
```

## Implementation details

### Attribute matching

At the moment this tool looks for full path `scholium::mark` and doesn't support
  neither import, nor `cfg_attr`.

### Overrides

Overrides is a table `prefix` to `severity` overriding default values defined
  for known report definitions.
  The most specific prefix always wins.

*Note:* Unknown reports are not affected by overrides.

Prefix is full group with an optional partial id and can be in forms as shown below:

* `gr`: matches any report id starting with `gr::` (not `group::`)
* `group`: matches any report id starting with `group::`
* `group::`: matches any report id starting with `group::`
* `group::i`: matches any report id starting with `group::i`
* `group::identifier` matches any report id starting with `group::identifier`

### Output filtering

All output is routed through [`tracing`](https://docs.rs/tracing) crate and
  can be configured via environment variable `CARGO_SCHOLIUM_LOG`.

Following targets are used:

* `parse_error`: For all syntax errors met
* `malformed_attribute`: When `scholium::mark` attribute met, but format is not recognized.
* `unknown_report`: When unknown report id met and unknown severity wasn't set
* `report`: All reports
* all other logs are written with default logging target they created.

## MSRV

Minimal MSRV is `1.86`, while latest Rust version is recommended (due to `cargo-platform` transient dependency).
