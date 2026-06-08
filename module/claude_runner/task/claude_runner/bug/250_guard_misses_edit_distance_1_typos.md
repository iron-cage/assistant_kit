# BUG-250 — `guard_unknown_subcommand` Misses Edit-Distance-1 Typos

## Execution State

- **State:** Fixed
- **Fixed:** 2026-06-07

## Symptom

`clr assk "hello"` (one-character mid-word insertion of `ask`) falls through
`guard_unknown_subcommand` silently and treats `"assk"` as the Claude message
instead of printing `"Did you mean 'ask'?"` and exiting 1.

```bash
CLR=target/debug/clr
# Actual (bug):
$CLR assk --dry-run
# → env block + assembled command with message "assk\n\nultrathink"
# → exit 0

# Expected:
# Error: unknown subcommand: assk. Did you mean 'ask'?
# Run with --help for usage.
# → exit 1
```

## Impact

- **Who**: Any user who makes a one-character insertion or substitution typo in a
  subcommand name for short subcommands (`ask`, `run`, `help`).
- **Conditions**: First CLI token has edit distance exactly 1 from a known
  subcommand but is neither a prefix nor a superstring of it.
- **Severity**: Minor-to-Moderate — no crash; the typo is silently forwarded to
  Claude as the message, producing a meaningless Claude response with no indication
  of the typo.
- **Silent**: Yes — no error is emitted; the binary exits 0 (or blocks at the
  session gate).

## How Discovered

Manual testing (Test & Fix Loop, NC-5 corner cases for `guard_unknown_subcommand`).
Testing `clr assk "hello"` revealed the command blocked on the session gate rather
than rejecting the token.  Confirmed with `clr assk --dry-run` which exited 0 and
assembled a Claude command with `"assk"` as the message.

## MRE

```bash
CLR=$(cargo build --bin clr 2>/dev/null; echo target/debug/clr)
$CLR assk --dry-run 2>&1
echo "exit: $?"
# Expected: "Error: unknown subcommand: assk. Did you mean 'ask'?", exit 1
# Actual:   assembled Claude command with message="assk\n\nultrathink", exit 0
```

## Root Cause

### Root Cause

`guard_unknown_subcommand` in `src/cli/mod.rs` only uses two `starts_with` checks:

```rust
if first != sub
  && ( sub.starts_with( first.as_str() ) || first.starts_with( sub ) )
```

For `first = "assk"` and `sub = "ask"`:
- `"ask".starts_with("assk")` → false (shorter cannot start with longer)
- `"assk".starts_with("ask")` → false (`"assk"[0..3] == "ass"`, not `"ask"`)

Neither direction matches, so the guard silently falls through to `dispatch_run`,
which calls `parse_args(["assk", "--dry-run"])` and treats `"assk"` as the message.

### Why Not Caught

The BUG-225 reproducer (`isolated_test.rs`) only tests prefix truncations
(`"isol"`, `"isolate"`) against the long `"isolated"` subcommand.  Short
subcommand names (`"ask"`, `"run"`) were never tested with mid-word character
insertions or substitutions, so the `starts_with`-only gap was never observed.

### Fix Location

`src/cli/mod.rs` — `guard_unknown_subcommand` — add `edit_distance_le1` call:

```rust
// Fix(BUG-250): extend guard to catch one-character insertion/substitution typos.
// Root cause: prefix/superstring checks only caught truncations and extensions;
//   mid-word insertions (e.g. "assk" for "ask") bypassed the guard and fell through
//   to dispatch_run, treating the typo silently as the message argument to Claude.
// Pitfall: edit_distance_le1 is additive — starts_with checks remain for extensions
//   (length difference > 1); edit_distance_le1 fills the one-char gap neither
//   starts_with direction catches.
if first != sub
  && ( sub.starts_with( first.as_str() ) || first.starts_with( sub ) || edit_distance_le1( first, sub ) )
```

Add a private `edit_distance_le1(a: &str, b: &str) -> bool` helper (Levenshtein ≤ 1)
after the `guard_unknown_subcommand` function.

### Prevention

Every subcommand must have a test covering at least one edit-distance-1 typo (mid-word
insertion or substitution), not just prefix/truncation variants.  The BUG-225 test
matrix should be extended to include a mid-word insertion case for each short subcommand.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|------------|-------|---------|----------|
| H1 | Guard only uses `starts_with`; no edit-distance check | ✅ Root Cause | `src/cli/mod.rs` guard condition confirmed | `src/cli/mod.rs:61-63` |
| H2 | "assk".starts_with("ask") is false | ✅ Confirmed | `"assk"[0..3] == "ass"` ≠ `"ask"` | Rust string semantics |
| H3 | "assk" falls through to `dispatch_run` | ✅ Confirmed | `clr assk --dry-run` exits 0 with "assk" as message | MRE output |

## Evidence Table

| Location | What It Shows |
|----------|---------------|
| `src/cli/mod.rs:61-63` | Guard condition: only `starts_with` checks; no edit-distance |
| MRE output | `clr assk --dry-run` exits 0; "assk" appears in assembled command |
| `tests/isolated_test.rs:562-613` | BUG-225 reproducer: only tests prefix truncations, not mid-word insertions |

## History

| Date | Event | Note |
|------|-------|------|
| 2026-06-07 | filed | Source: manual testing NC-5; `clr assk --dry-run` exits 0 silently |
| 2026-06-07 | fixed | `src/cli/mod.rs` — added `edit_distance_le1` helper + extended guard condition |
| 2026-06-07 | verified | `ask_command_test.rs::t12_ask_edit_distance_typo_caught_by_guard` passes; `clr assk --dry-run` exits 1 with "Did you mean 'ask'?" |
