# Localhost Only

**Status**: Planned | **Since**: 1.3.0

## Description

The `.serve` web server binds to `127.0.0.1` by default. Journal data may contain sensitive information (prompts, API responses, credential names). Exposing the web viewer to the network requires explicit `bind::0.0.0.0` (or another non-loopback address) — the user must consciously choose to expose the data.

## Measurement

- **Threshold**: Default bind address is `127.0.0.1` (measured by unit test asserting default and by code review of the `.serve` command)
- **Method**: Test in `serve_test.rs` asserts that `.serve` without `bind::` param uses `127.0.0.1:8411`

## Sources

- `src/cli/serve.rs` — default bind address constant
- `docs/cli/param/16_bind.md` — parameter documentation
