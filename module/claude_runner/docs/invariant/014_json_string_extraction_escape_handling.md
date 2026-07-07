# Invariant: JSON String Extraction Escape Handling

### Scope

- **Purpose**: Ensure every hand-rolled JSON string-value extraction site in the crate correctly bounds a string value in the presence of escaped quotes, and fails closed (returns `None`) rather than returning a silently truncated or partial value when the terminator is never found.
- **Responsibility**: State the escape-tracking requirement for locating a JSON string value's closing quote, and the fail-closed requirement for the case where the scan exhausts its input without finding one. Both requirements apply independently — a site can violate either without violating the other (BUG-394 violates the first at 3 sites while a 4th, `extract_str`, satisfies it; BUG-395 violates the second at that same 4th site while satisfying the first).
- **In Scope**: `try_jsonl_task()` (`ps.rs`), `parse_json_str()` (`ps.rs`), the inline `model_name` extraction in `render_summary()` (`summary.rs`), `extract_str()` (`summary.rs`) — all four JSON string-value terminator scans in the crate.
- **Out of Scope**: `extract_json_value()` (`summary.rs`) — already correctly escape-aware and fail-closed (`chars.next()?` propagates `None` on exhaustion); included here only as the known-correct comparison baseline, not as a site under this invariant's enforcement. JSON writing/escaping on the write side (→ `gate.rs::json_escape_str()`, BUG-384 — a separate, already-hardened concern). `render_summary()`'s overall gate-field selection (→ `008_render_summary_gate.md`).

### Invariant Statement

Every JSON string-value terminator scan in this crate MUST satisfy both of the following, independently:

1. **Escape-aware bounding:** The scan for the value's closing `"` MUST track whether the immediately-preceding character was an unescaped backslash, and MUST NOT treat an escaped `\"` as the terminator. A bare `rest.find('"')` (or an equivalent two-call first/next pattern) that does not track escape state is a violation.
2. **Fail-closed on exhaustion:** If the scan exhausts its input without finding an unescaped closing `"`, the function MUST return `None` (or the crate's equivalent absent-value signal), not a partial/truncated success value. Falling through a scan loop to a trailing `Some(...)` after the loop is a violation unless that fallthrough is itself gated by an explicit terminator check.

| Site | Escape-aware (req. 1) | Fail-closed on exhaustion (req. 2) | Status before fix |
|------|------------------------|--------------------------------------|--------------------|
| `try_jsonl_task()` (`ps.rs:850`) | Required | Required (already `?`-propagates via `Option` chain) | ❌ req. 1 (BUG-394 site 1) |
| `parse_json_str()` (`ps.rs:863`) | Required | Required (already `?`-propagates via `Option` chain) | ❌ req. 1 (BUG-394 site 2) |
| `render_summary()` inline `model_name` (`summary.rs:330,332`) | Required | Required (already `?`-propagates via `Option` chain) | ❌ req. 1 (BUG-394 site 3) |
| `extract_str()` (`summary.rs:22-54`) | ✅ already correct | Required | ❌ req. 2 (BUG-395) |

**Correct reference implementation** (already present in this file as the known-good baseline — `extract_json_value()`'s string-value branch, `summary.rs:154-166`):

```rust
'"' =>
{
  // JSON string — walk to the closing unescaped quote.
  let mut chars   = rest.char_indices().skip( 1 );
  let mut escaped = false;
  loop
  {
    let (i, c) = chars.next()?;      // req. 2: `?` propagates None on exhaustion
    if escaped { escaped = false; continue; }
    if c == '\\' { escaped = true; continue; }  // req. 1: escape tracking
    if c == '"' { return Some( rest[ ..= i ].to_owned() ); }
  }
}
```

### Rationale — Why One Invariant Covers Two Independent Requirements

BUG-394 and BUG-395 were filed and verified as two separate bug reports because they have distinct symptoms, distinct affected call sites, and distinct severities of consequence — but they are the same structural defect class: a hand-rolled scan for a JSON string's closing quote that does not correctly define "found the terminator" (BUG-394: an escaped quote is mistaken for the terminator) or "did not find the terminator" (BUG-395: exhaustion is mistaken for success). Consolidating them into one invariant, rather than two, follows this crate's own dedup-oriented bug history (BUG-295/296/297, BUG-385/386/391) of treating "the same hand-rolled parsing gap recurring at multiple call sites" as one Gap Class rather than N unrelated defects — see `014`'s Provenance table for both bugs' cross-reference.

### Enforcement Mechanism

**Requirement 1 (escape-aware bounding) — shared helper, all 3 BUG-394 sites:**

```rust
// Candidate shared helper — mirrors extract_json_value's already-correct pattern.
fn find_unescaped_quote( s : &str ) -> Option< usize >
{
  let mut escaped = false;
  for ( i, c ) in s.char_indices()
  {
    if escaped { escaped = false; continue; }
    if c == '\\' { escaped = true; continue; }
    if c == '"' { return Some( i ); }
  }
  None
}
```

`ps.rs:850`'s `try_jsonl_task()` and `ps.rs:863`'s `parse_json_str()` must replace `rest.find( '"' )` with `find_unescaped_quote( rest )`. `summary.rs:330,332`'s two-call `model_name` extraction (`s.find('"')` then `inner.find('"')`) must apply the same substitution to both calls. Note: replacing only the terminator search bounds the value correctly but does not unescape the captured slice's own contents — whether a given caller additionally needs the captured value unescaped (as `extract_str` does) or only correctly bounded (sufficient for `parse_json_str`'s and `try_jsonl_task`'s truncate-to-35-chars display use) is a per-call-site decision, not part of this invariant.

**Requirement 2 (fail-closed on exhaustion) — `extract_str()`:**

```rust
// src/cli/summary.rs, end of extract_str()'s char-by-char loop:
    if c == '"'  { return Some( out ); }
    out.push( c );
  }
  None   // Fix(BUG-395): loop exhausted without an unescaped closing quote — the
         // value is unterminated/malformed, not a legitimately short string.
}
```

Every OTHER malformed-input path in `extract_str()` already correctly returns `None` (absent key: `s.find(&needle)?`; `null` value: explicit check; non-string value: explicit check) — only the "input ends before the string is properly closed" case is missing an explicit `None`.

### Violation Consequences

**If requirement 1 is violated (escape-unaware scan reintroduced at any of the 3 sites):**
- `try_jsonl_task()`: any active session whose most recent Form-A human message contains an escaped `"` produces a garbled, prematurely-truncated Task-column preview in `clr ps`'s active-sessions table (e.g. `He said \"hi\" ...` truncates to `He said \`).
- `parse_json_str()`: any waiting invocation whose working directory contains a literal `"` byte produces a garbled, truncated CWD in `clr ps`'s "Queued CLR Processes" table — this is the unpaired **read** side of a round-trip whose **write** side (`gate.rs::json_escape_str()`) was already hardened by BUG-384; a regression here reopens exactly the display gap BUG-384's escaping made reachable.
- `render_summary()`'s `model_name` field: garbled model-name display if a `modelUsage` object key ever contains an escaped quote (lower practical likelihood — model identifiers are internally-controlled).
- All three failures are silent — `.find('"')` returning `Some` at the wrong offset is indistinguishable from a legitimately short string; no error, panic, or `None` fallback signals the truncation.

**If requirement 2 is violated (fail-closed fallback removed from `extract_str()`):**
- `extract_session_id()` (`?`-propagated, no `.unwrap_or_default()` guard) returns `Some(<truncated-uuid>)` instead of `None` when a truncation lands inside the `session_id` field — `execution.rs`'s BUG-320 mismatch comparison then almost certainly fires (a truncated partial UUID essentially never coincidentally equals the full expected UUID), producing a **false-positive** `"[Runner] warning: session mismatch ... (BUG-320 detected)"` that misdiagnoses a truncated envelope as session drift.
- `render_summary()`'s `"result"` field (`.unwrap_or_default()`-bounded, so failure here is display-only, not control-flow) displays partial, misleadingly-plausible-looking text instead of an empty string.
- Both failures are silent — no error, panic, or log line signals that the source data was actually truncated; every OTHER malformed-input path in `extract_str()` remains correctly `None`-returning, isolating any regression to exactly this one fallthrough.

### Sources

| File | Notes |
|------|-------|
| `../../src/cli/ps.rs` | `try_jsonl_task()` (site 1, req. 1), `parse_json_str()` (site 2, req. 1) |
| `../../src/cli/summary.rs` | `render_summary()` inline `model_name` extraction (site 3, req. 1); `extract_str()` (req. 2); `extract_json_value()` (known-correct baseline for both requirements, out of scope) |

### Tests

| File | Notes |
|------|-------|
| `../../tests/ps_command_test.rs` | Existing `it_16_task_column_form_a` harness (site 1) — requires an escaped-quote content fixture to exercise req. 1 |
| `../../tests/concurrency_gate_test.rs` | T07/T13 (BUG-384) exercise the write side only; requires a `clr ps`-invoking variant to exercise `parse_json_str()`'s read side (site 2, req. 1) |
| `../../tests/summary_unit_test.rs` | Existing unit tests for `summary.rs`'s extraction helpers; requires an escaped-quote `modelUsage` key fixture (site 3, req. 1) and an unterminated-string fixture for `extract_str` (req. 2) |

### Provenance

| Source | Notes |
|--------|-------|
| BUG-394 | Root bug for requirement 1: 3 sites (`ps.rs:850`, `ps.rs:863`, `summary.rs:330,332`) each hand-roll `.find('"')` with no escape tracking. Verified 2026-07-07; fix not yet applied — this invariant documents the required (not yet implemented) behavior. |
| BUG-395 | Root bug for requirement 2: `extract_str()` (`summary.rs:22-54`) falls through to `Some(<truncated>)` instead of `None` when its scan exhausts without finding a closing quote. Verified 2026-07-07; fix not yet applied — this invariant documents the required (not yet implemented) behavior. |
| BUG-384 | Prior, closed bug hardening the write side (`gate.rs::json_escape_str()`) of the exact round-trip BUG-394 site 2 is the unpaired read side of. |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/008_render_summary_gate.md](008_render_summary_gate.md) | Governs `render_summary()`'s overall gate-field selection; this invariant governs one specific inline extraction (`model_name`) within the same function, orthogonal to the gate decision |
| [invariant/009_session_mismatch_detection.md](009_session_mismatch_detection.md) | `extract_session_id()` calls `extract_str()`; a requirement-2 violation in `extract_str()` produces the false-positive mismatch warning this invariant's Violation Consequences section describes |

### Features

| File | Relationship |
|------|--------------|
| [feature/001_runner_tool.md](../feature/001_runner_tool.md) | Defines the print-mode execution path whose result envelope these extraction sites parse |
