# BUG-249 — `clr ask help` Hits Session Gate Instead of Showing Help

## Execution State

- **State:** Fixed
- **Fixed:** 2026-06-07

## Symptom

`clr ask help` blocks on the session gate instead of displaying help text.
When ≥ `--max-sessions` (default 10) Claude processes are running, the command
hangs for 30 s then exits 1 with a timeout message.

```bash
timeout 5 bash -c "PATH=/nonexistent clr ask help" 2>&1
# Actual (bug):
#   Info: 12 claude session(s) running (limit 10); waiting 30s for a free slot...
#   [exit 124 — timeout]
# Expected:
#   Usage: clr ask [OPTIONS] [MESSAGE]
#   [help text]
#   [exit 0]
```

## Impact

- **Who**: Any user running `clr ask help` while ≥10 Claude sessions are active.
- **Conditions**: `clr ask help` with positional "help" token (not `--help`/`-h`).
- **Severity**: Minor — workaround is `clr ask --help`; but `clr run help` works
  correctly (BUG-215 fix applied), so `ask` is inconsistent with `run`.
- **Silent**: No — emits session-gate wait message.

## How Discovered

Manual testing (Test & Fix Loop, group B — ask command variants). Spec
`tests/docs/cli/command/05_ask.md` IT-8 states "clr ask help → dispatches to
help, exit 0". Confirmed failing by running `timeout 5 bash -c "PATH=/nonexistent $CLR ask help"`.

## MRE

```bash
CLR=$(cargo build --bin clr 2>/dev/null; echo target/debug/clr)
# Ensure ≥10 claude sessions running (or stub with PATH=/nonexistent to expose the gate)
timeout 5 bash -c "PATH=/nonexistent $CLR ask help" 2>&1
echo "exit: $?"
# Expected: help text, exit 0
# Actual: session-gate message, exit 124 (timeout) or exit 1
```

## Root Cause

### Root Cause

`dispatch_ask()` (`src/cli/mod.rs`) intercepts `--help`/`-h` flags but does NOT
handle the positional "help" token. The call `dispatch_run(&tokens[1..])` receives
`["help"]`, which `parse_args` interprets as `message = Some("help")`. The
assembled command is then passed to `run_built_command()`, which calls
`wait_for_session_slot()` before subprocess launch — blocking when the session
limit is reached.

By contrast, `clr run help` works because `run_cli()` in `src/lib.rs` re-checks
for the "help" token immediately after stripping the "run" prefix (BUG-215 fix).
No equivalent check exists in `dispatch_ask`.

```rust
// Current (broken):
pub(super) fn dispatch_ask(tokens: &[String]) -> ! {
  if tokens.iter().skip(1).any(|t| t == "--help" || t == "-h") {
    print_ask_help();
  }
  dispatch_run(&tokens[1..]);  // "help" treated as message
}
```

### Why Not Caught

IT-8 in `tests/docs/cli/command/05_ask.md` specifies the behavior but no
automated test existed for `clr ask help` (only `clr ask --help` was implicitly
covered by the help-flag intercept). The session gate blocks only when ≥10
sessions are active, making the bug intermittent in normal test environments.

### Fix Location

`src/cli/mod.rs` — `dispatch_ask()` — add positional "help" check before
`dispatch_run`, mirroring the BUG-215 pattern from `run_cli()`:

```rust
// Fix(BUG-249): 'clr ask help' must show ask help, not treat "help" as a message.
// Root cause: dispatch_ask only intercepted --help/-h flags, not the positional
//   "help" token, so "help" flowed into dispatch_run as a message and hit the gate.
// Pitfall: mirrors BUG-215 fix in run_cli() for 'clr run help'; both subcommands
//   need the positional check; future subcommands must include it too.
if tokens.get(1).map(String::as_str) == Some("help") {
  print_ask_help();
}
```

### Prevention

Every subcommand dispatcher that delegates to `dispatch_run` must intercept
the positional "help" token. IT-8-style tests (positional "help" without `--`)
must be included in the test matrix for every subcommand, not just `--help`/`-h`.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|------------|-------|---------|----------|
| H1 | `dispatch_ask` does not check positional "help" | ✅ Root Cause | Only checks `--help`/`-h` flags | `src/cli/mod.rs:549-556` |
| H2 | "help" token flows into `dispatch_run` as message | ✅ Confirmed | `parse_args(["help"])` → `message = Some("help")` → `run_built_command` → gate | `src/cli/parse.rs`, `src/cli/mod.rs` |
| H3 | `clr run help` works via different path | ✅ Confirmed | `run_cli()` re-checks "help" after stripping "run" (BUG-215 fix) | `src/lib.rs` |

## Evidence Table

| Location | What It Shows |
|----------|---------------|
| `src/cli/mod.rs:549-556` | `dispatch_ask` — only `--help`/`-h` intercepted; no positional check |
| `src/cli/mod.rs:~230` | `wait_for_session_slot` called unconditionally in `run_built_command` |
| `src/lib.rs` | `run_cli()` re-checks "help" after stripping "run" — BUG-215 fix |
| `tests/docs/cli/command/05_ask.md:98-103` | IT-8 specifies `clr ask help → exit 0` |

## History

| Date | Event | Note |
|------|-------|------|
| 2026-06-07 | filed | Source: manual testing group B (ask variants); confirmed by timeout test |
| 2026-06-07 | fixed | `src/cli/mod.rs:561-564` — `Fix(BUG-249)`: positional "help" intercepted in `dispatch_ask` before delegating |
| 2026-06-07 | verified | `ask_command_test.rs::t11_ask_positional_help_shows_help` passes; `clr ask help` exits 0 with help text |
