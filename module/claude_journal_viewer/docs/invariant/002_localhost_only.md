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

- **Threshold**: Default bind address is `127.0.0.1` (measured by unit test asserting default and by code review of the `.serve` command)
- **Method**: Test in `serve_test.rs` asserts that `.serve` without `bind::` param uses `127.0.0.1:8411`

## Sources

- `src/cli/serve.rs` — default bind address constant
- `docs/cli/param/16_bind.md` — parameter documentation (Phase 2 deliverable; created alongside `src/cli/serve.rs`)
