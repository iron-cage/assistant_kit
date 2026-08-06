# Invariant: render_summary() Gate Field

### Scope

- **Purpose**: Ensure `render_summary()` accepts any valid CLR result envelope regardless of which optional fields are present, and only rejects genuinely non-CLR input.
- **Responsibility**: State the compound gate (`subtype` presence OR `"type":"result"`), the required handling of optional fields with `.unwrap_or_default()`, and the precise conditions under which `render_summary()` is permitted to return `None`.
- **In Scope**: `render_summary()` gate field selection, optional-field handling (`session_id`, `usage`, `total_cost_usd`, and all other non-`type` fields), `None`-return preconditions, structural fragility anti-pattern (gating on optional fields).
- **Out of Scope**: Which fields appear in the rendered output (→ `cli/param/071_summary_fields.md`), when `render_summary()` is called (→ `cli/param/070_output_style.md`), the full exit code contract (→ `006_exit_codes.md`).

### Invariant Statement

`render_summary(json, fields)` MUST return `Some(rendered)` for any JSON string that is a valid CLR result envelope — either the old SDK format (`"type":"result"` at top level, no `"subtype"`) or the new SDK format (`"subtype"` present at top level, regardless of any `"type"` value in nested fields).

`render_summary()` MUST return `None` only when:
1. The input is not valid JSON, or
2. The input JSON passes neither arm of the compound gate: `"subtype"` is absent AND `"type"` is not `"result"` (non-CLR-result input such as stream chunks with `"type":"message"` but no `"subtype"`, or JSON lacking both fields).

Missing optional fields (`session_id`, `usage`, `total_cost_usd`, and any other field not shown in a minimal CLR envelope) MUST NOT cause `render_summary()` to return `None`. All optional fields must be extracted with `.unwrap_or_default()` or equivalent safe fallback — never with the `?` propagation operator.

| Condition | Return value | Rationale |
|-----------|-------------|-----------|
| JSON has `"subtype"` at top level (new SDK) | `Some(rendered)` | New SDK result envelope — `"subtype"` is exclusive to top-level CLR result objects |
| JSON has `"type":"result"` at top level (old SDK, no `"subtype"`) | `Some(rendered)` | Old SDK result envelope — always render |
| JSON has `"type":"result"` but lacks `session_id` | `Some(rendered)` | `session_id` is optional in some claude binary versions |
| JSON has `"type":"result"` but lacks `usage` | `Some(rendered)` | `usage` may be absent in minimal envelopes |
| JSON has `"type":"message"` and no `"subtype"` (stream chunk) | `None` | Not a CLR result — fall back to raw |
| JSON lacks both `"subtype"` and `"type":"result"` | `None` | Not a CLR result — fall back to raw |
| Input is not JSON | `None` | Unparseable — fall back to raw |

**Compound gate (BUG-436 fix):** The `None`-vs-`Some` return decision requires `subtype.is_some() || msg_type == "result"`. The old single-field gate (`"type":"result"` only) fails for new Claude SDK envelopes where `usage.iterations[].type = "message"` appears earlier in the serialized JSON — `extract_str` (depth-unaware `s.find()`) returns `"message"` for the nested field, causing `render_summary()` to incorrectly return `None`. The `"subtype"` field is emitted only at the top level of CLR result envelopes.

**Anti-pattern:** Gating on optional fields (e.g. `session_id`) using Rust's `?` operator on an `Option` restores the raw-JSON fallback symptom for any CLR binary version that omits that field. This is the structural root of BUG-309 (field name `"id"` absent) and BUG-310 (field name `"session_id"` absent from 7-field minimal envelopes). See D15 in `../feature/006_cli_design.md`.

### Enforcement Mechanism

In `src/cli/summary.rs`, `render_summary()` must apply the gate as follows:

```rust
// Fix(BUG-310): gate on invariant field, not optional session_id.
// Root cause: extract_str(json,"session_id")? returned None for 7-field envelopes
//   where session_id is absent — restoring BUG-309 raw-JSON symptom.
//
// Fix(BUG-436): compound gate — accepts old SDK ("type":"result", no subtype) and new SDK
//   ("subtype" present, no top-level "type"). Non-result stream chunks have neither and
//   are rejected. "type":"result"-only gate fails for new SDK envelopes where
//   usage.iterations[].type = "message" appears first (extract_str depth-unaware find()).
// Pitfall: extract_str uses s.find() — gate only on fields exclusive to the top level.
let subtype  = extract_str( json, "subtype" );
let msg_type = extract_str( json, "type" ).unwrap_or_default();
if subtype.is_none() && msg_type != "result" { return None; }
// Fix(BUG-440): clear msg_type for new SDK path so "type:" display line is not wrong.
let msg_type = if subtype.is_some() && msg_type != "result" { String::new() } else { msg_type };
let session_id = extract_str( json, "session_id" ).unwrap_or_default();
```

All subsequent field extractions in `render_summary()` must use `.unwrap_or_default()`, `.unwrap_or(0)`, `.unwrap_or(false)`, or equivalent safe fallbacks. No `?` operator may be used on field extractions after the `msg_type` gate check.

### Violation Consequences

If `render_summary()` gates on an optional field using `?`:
- All CLR result envelopes from any claude binary version that omits that field produce raw JSON output on stdout instead of the expected key:val summary
- `--output-style summary` (the default) silently degrades for those envelopes — users see unformatted JSON
- EC-14 (`ec14_render_summary_minimal_envelope_no_session_id` in `output_style_test.rs`) fails
- The bug is invisible in CI if the fake claude fixture always includes the gated field — it manifests only with real binary output from a different version

### Features

| File | Relationship |
|------|--------------|
| [feature/001_runner_tool.md](../feature/001_runner_tool.md) | Defines the print-mode execution path and `render_summary()` role within it |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/009_session_mismatch_detection.md](009_session_mismatch_detection.md) | `extract_session_id()` and `extract_structured_output()` inherit the compound gate defined by this invariant (BUG-437/BUG-438 fixes) |

### Sources

| File | Relationship |
|------|--------------|
| `../../src/cli/summary.rs` | `render_summary()` implementation — compound gate at ~line 330; `extract_session_id()` compound gate at ~line 131; `extract_structured_output()` compound gate at ~line 152 |
| `../../src/cli/execution.rs` | Call site: `render_summary(&out, cli.summary_fields.as_deref()).unwrap_or(out)` |

### Tests

| File | Relationship |
|------|--------------|
| `../../tests/output_style_test.rs` | EC-14: `ec14_render_summary_minimal_envelope_no_session_id` — minimal 7-field CLR envelope; asserts `stdout.contains("---")` (renders `Some`) |

### Provenance

| Source | Notes |
|--------|-------|
| TSK-236 | Verified task implementing the gate fix and EC-14 test |
| BUG-310 | Root bug: `extract_str(json,"session_id")?` in `render_summary()` returns `None` for minimal envelopes lacking `session_id` |
| BUG-309 | Prior structural instance: gate was on `"id"` (changed to `"session_id"` in TSK-233) — same `?`-gate anti-pattern |
| BUG-436 | Compound gate fix for `render_summary()`: single `"type":"result"` gate failed for new SDK envelopes where `usage.iterations[].type = "message"` appears first — depth-unaware `extract_str` matched the nested field; `render_summary()` incorrectly returned `None` |
| BUG-437 | Same compound gate fix for `extract_session_id()` — BUG-320 session mismatch detection silently disabled for all new SDK envelopes |
| BUG-438 | Same compound gate fix for `extract_structured_output()` — `--json-schema` output silently fell back to raw JSON for new SDK envelopes |
| BUG-440 | `msg_type` display fix for new SDK path: clear `msg_type` when `subtype` is present but no top-level `"type":"result"` found, so the rendered `type:` field is not populated with a wrong nested value |
| D15 | Design decision in `docs/feature/006_cli_design.md` documenting the invariant-field gate rationale |
