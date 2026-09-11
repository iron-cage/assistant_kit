# Decision: render_summary() Gate Field

**ID:** D15 · **Category:** Pipeline · **Status:** ✅ Adopted

### Scope

- **Purpose**: Record why `render_summary()` gates on the invariant field `"type":"result"` rather than any optional field, and the two-bug history that established this as the only correct choice.
- **Responsibility**: Rationale for the invariant-field gate, the BUG-309/BUG-310 pattern it closes, and the pitfall a future change to the gate field must avoid repeating.
- **In Scope**: Why `"type":"result"` is the gate; why optional fields must never gate; the structural-recurrence pattern between BUG-309 and BUG-310.
- **Out of Scope**: The full invariant specification and anti-pattern table (→ [`../invariant/008_render_summary_gate.md`](../invariant/008_render_summary_gate.md)).

### Decision

`render_summary()` uses `"type":"result"` as its primary gate condition. Optional fields such as `session_id` are extracted with `.unwrap_or_default()` when absent.

### Rationale

The `claude --output-format json` envelope schema varies by binary version. At least one observed version emits a minimal 7-field envelope without `session_id`:

```json
{"type":"result","subtype":"success","is_error":false,"duration_ms":N,"duration_api_ms":N,"num_turns":N,"result":"..."}
```

Gating on any optional field causes `render_summary()` to return `None` for that variant, silently restoring the raw-JSON fallback symptom that summary rendering was intended to fix in the first place. `"type":"result"` is present in every CLR result envelope observed across all tested claude binary versions — it is the only field confirmed to survive every schema variation, which is what makes it the only reliable gate.

### History

**BUG-309** gated on `"id"` using Rust's `?` operator on an `Option`. When a binary version omitted `"id"`, the gate silently failed and rendering fell back to raw JSON.

**BUG-310** is a structural recurrence: the BUG-309 fix replaced the `"id"` gate with a `"session_id"` gate — same `?`-gate mechanism, different optional field. This inherited the identical structural fragility, because the fix changed *which* optional field was tested without changing the underlying pattern of testing an optional field at all.

### Pitfall

Any future change to the gate field must use a field guaranteed present in **all** CLR result envelopes across all claude binary versions — not merely the one currently being debugged. Swapping one optional field for another optional field reproduces BUG-309/BUG-310 under a new field name; the fix that actually closes the pattern is gating on a field the schema treats as invariant, never on one treated as optional anywhere in the observed version range.

### Consequence

`render_summary()` returns `None` only for non-CLR-result JSON (envelope lacks `"type":"result"`) or non-JSON input — not for CLR envelopes that omit optional fields like `session_id`, `usage`, or `total_cost_usd`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Decision collection index |
| invariant | [`../invariant/008_render_summary_gate.md`](../invariant/008_render_summary_gate.md) | Full invariant specification and anti-pattern table |
| cli | [`../cli/param/070_output_style.md`](../cli/param/070_output_style.md) | Output style behavior gated on this same invariant field |
| source | `../../src/cli/summary.rs` | `render_summary()` — the invariant field gate |
| test | `../../tests/summary_unit_test.rs` | `render_summary()` invariant field gate coverage |
