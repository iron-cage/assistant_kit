# Invariant: render_summary() Gate Field

### Scope

- **Purpose**: Ensure `render_summary()` accepts any valid CLR result envelope regardless of which optional fields are present, and only rejects genuinely non-CLR input.
- **Responsibility**: State the mandatory gate field (`"type":"result"`), the required handling of optional fields with `.unwrap_or_default()`, and the precise conditions under which `render_summary()` is permitted to return `None`.
- **In Scope**: `render_summary()` gate field selection, optional-field handling (`session_id`, `usage`, `total_cost_usd`, and all other non-`type` fields), `None`-return preconditions, structural fragility anti-pattern (gating on optional fields).
- **Out of Scope**: Which fields appear in the rendered output (→ `cli/param/071_summary_fields.md`), when `render_summary()` is called (→ `cli/param/070_output_style.md`), the full exit code contract (→ `006_exit_codes.md`).

### Invariant Statement

`render_summary(json, fields)` MUST return `Some(rendered)` for any JSON string that contains `"type":"result"`, regardless of which other fields are present or absent.

`render_summary()` MUST return `None` only when:
1. The input is not valid JSON, or
2. The input JSON does not contain `"type":"result"` (non-CLR-result input such as `"type":"message"` or JSON lacking a `type` field entirely).

Missing optional fields (`session_id`, `usage`, `total_cost_usd`, and any other field not shown in a minimal CLR envelope) MUST NOT cause `render_summary()` to return `None`. All optional fields must be extracted with `.unwrap_or_default()` or equivalent safe fallback — never with the `?` propagation operator.

| Condition | Return value | Rationale |
|-----------|-------------|-----------|
| JSON contains `"type":"result"` | `Some(rendered)` | Valid CLR result envelope — always render |
| JSON contains `"type":"result"` but lacks `session_id` | `Some(rendered)` | `session_id` is optional in some claude binary versions |
| JSON contains `"type":"result"` but lacks `usage` | `Some(rendered)` | `usage` may be absent in minimal envelopes |
| JSON does not contain `"type":"result"` | `None` | Not a CLR result — fall back to raw |
| Input is not JSON | `None` | Unparseable — fall back to raw |

**Invariant field:** `"type":"result"` is the ONLY field permitted to gate the `None`-vs-`Some` return decision. It is present in every CLR result envelope across all observed claude binary versions.

**Anti-pattern:** Gating on optional fields (e.g. `session_id`) using Rust's `?` operator on an `Option` restores the raw-JSON fallback symptom for any CLR binary version that omits that field. This is the structural root of BUG-309 (field name `"id"` absent) and BUG-310 (field name `"session_id"` absent from 7-field minimal envelopes). See D15 in `../001_design_decisions.md`.

### Enforcement Mechanism

In `src/cli/summary.rs`, `render_summary()` must apply the gate as follows:

```rust
// Fix(BUG-310): gate on invariant field, not optional session_id.
// Root cause: extract_str(json,"session_id")? returned None for 7-field envelopes
//   where session_id is absent — restoring BUG-309 raw-JSON symptom.
// Pitfall: any ? on an optional CLR field silently breaks all envelopes missing that field.
let msg_type = extract_str( json, "type" )?;
if msg_type != "result" { return None; }
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

### Sources

| File | Relationship |
|------|--------------|
| `../../src/cli/summary.rs` | `render_summary()` implementation — gate field selection at ~line 189 |
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
| D15 | Design decision in `docs/001_design_decisions.md` documenting the invariant-field gate rationale |
