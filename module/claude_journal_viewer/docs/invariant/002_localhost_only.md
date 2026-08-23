# Localhost Only

**Status**: Planned | **Since**: 1.3.0

### Scope

- **Purpose**: Guarantee journal data is not exposed to the network without explicit user consent.
- **Responsibility**: Documents the default bind address for the `.serve` web server and how a user opts into wider exposure.
- **In Scope**: The `127.0.0.1` default and the explicit `bind::` override required to listen beyond loopback.
- **Out of Scope**: Web dashboard content/behavior (→ `docs/feature/002_web_viewing.md`), read-only file-access guarantees (→ `docs/invariant/001_read_only.md`).

## Description

The `.serve` web server binds to `127.0.0.1` by default. Journal data may contain sensitive information (prompts, API responses, credential names). Exposing the web viewer to the network requires explicit `bind::0.0.0.0` (or another non-loopback address) — the user must consciously choose to expose the data.

## Measurement

- **Threshold**: Bind address is `127.0.0.1` for every `.serve` invocation (measured by code review of `cmd_serve()`; there is no configurable path to widen it yet)
- **Method**: Grep assertion over `src/cli_main.rs` — the only argument reaching `tiny_http::Server::http()` is `format!( "127.0.0.1:{port}" )`, and the parameter map is never queried for a `bind` key. No dedicated test file exists yet; `serve_test.rs` is a Phase 2 deliverable alongside `bind::` itself, and the invariant stays `Status: Planned` until both land

## Sources

- `src/cli_main.rs` `cmd_serve()` — builds the bind address (`format!( "127.0.0.1:{port}" )`, line 168) and hands it to `tiny_http::Server::http()`
- `docs/cli/param/16_bind.md` — parameter documentation (Phase 2 deliverable; `bind::` is not yet wired into `cmd_serve()`, which still hardcodes the `127.0.0.1` loopback host)
