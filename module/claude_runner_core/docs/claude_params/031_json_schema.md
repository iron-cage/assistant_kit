# json_schema

JSON Schema for structured output validation.

## Type

**CLI** — JSON string value

## Syntax

```
claude --print --output-format json --json-schema '<schema>'
```

## Default

None (no schema validation)

## Description

Provides a JSON Schema that Claude's output must conform to. When set, Claude Code validates and potentially coerces the response to match the schema structure.

Use cases:
- Generating structured data with guaranteed field presence
- Enforcing output types (e.g., always return an array of objects)
- API integrations where the consumer expects a specific JSON shape

The schema should be a valid JSON Schema object. Claude will attempt to produce output matching the schema.

## Builder API

Use `with_json_schema()` — Accepts a JSON Schema string for structured output.

```rust
use claude_runner_core::ClaudeCommand;

let schema = r#"{"type":"object","properties":{"answer":{"type":"string"}}}"#;
let cmd = ClaudeCommand::new()
  .with_json_schema( schema )
  .with_message( "Answer in JSON" );
```

## Examples

```bash
# Require structured output
claude --print \
  --output-format json \
  --json-schema '{"type":"object","properties":{"issues":{"type":"array","items":{"type":"string"}},"severity":{"type":"string","enum":["low","medium","high"]}},"required":["issues","severity"]}' \
  "Review src/auth.rs for security issues"

# Simple typed output
claude --print \
  --output-format json \
  --json-schema '{"type":"array","items":{"type":"string"}}' \
  "List 5 Rust crates for HTTP clients"
```

## Notes

- Works best with `--output-format json` (single response object)
- Claude makes best-effort to conform to the schema; validation is not strictly enforced in all versions
- Complex schemas with many required fields may produce less accurate results
