# Localhost Only

**Status**: Implemented | **Since**: 1.3.0

### Scope

- **Purpose**: Guarantee journal data is not exposed to the network without explicit user consent.
- **Responsibility**: Documents the default bind address for the `.serve` web server and how a user opts into wider exposure.
- **In Scope**: The `127.0.0.1` default and the explicit `bind::` override required to listen beyond loopback.
- **Out of Scope**: Web dashboard content/behavior (→ `docs/feature/002_web_viewing.md`), read-only file-access guarantees (→ `docs/invariant/001_read_only.md`).

## Description

The `.serve` web server binds to `127.0.0.1` by default. Journal data may contain sensitive information (prompts, API responses, credential names). Exposing the web viewer to the network requires explicit `bind::0.0.0.0` (or another non-loopback address) — the user must consciously choose to expose the data.

Consent is made visible rather than silent. A non-loopback bind changes two
observable outputs: the startup line reports the address actually listened on
instead of `localhost`, and a warning goes to stderr naming the exposure. A
user who widens the bind by accident (a stray `bind::0.0.0.0` in a script)
therefore sees it in the first two lines of output, not only in a later
network scan.

The warning is written **before** the startup line. Any consumer that treats
the startup line as the "server is up" signal — a script, a supervisor, the
test harness — is then guaranteed to already have the warning in hand.
Emitting it afterwards would make the exposure notice racy for precisely the
readers that automate around this command.

## Measurement

- **Threshold**: Bind address is `127.0.0.1` for every `.serve` invocation that does not pass `bind::`; any other address is reached only through an explicit `bind::` value, and is accompanied by the stderr exposure warning
- **Method**: `tests/serve_test.rs` — `ft1_in1_serve_starts_on_loopback_and_prints_url` asserts the default startup line reports `localhost` and carries no exposure warning; `in2_non_loopback_bind_is_honored_and_warned` asserts `bind::0.0.0.0` reports its real address and does warn; `in3_bind_selects_the_interface` binds `127.0.0.2` and asserts `127.0.0.1` is *refused* on the same port — the case that can only pass when the parameter genuinely selects the interface rather than being ignored in favour of a hardcoded loopback

## Sources

- `src/cli_main.rs` `cmd_serve()` — reads `bind::` (default `127.0.0.1`), builds `format!( "{bind}:{port}" )`, classifies the result as loopback or not, and emits the startup line and exposure warning accordingly
- `docs/cli/param/16_bind.md` — parameter documentation
- `tests/serve_test.rs` — IN-1/IN-2/IN-3 enforcement
