# Design Decisions: clr

Rationale for key design choices made during the `--flag value` CLI redesign (task 031).

### D2 — `--verbose` vs `--quiet` (supersedes `--verbosity`)

`--verbose` passes through to claude (it's a claude-native flag). Runner-internal diagnostic
output is controlled by `--quiet` (bool, default false) — when set, non-fatal CLR diagnostics
(gate-wait, retry progress, retry-exhaustion, keep-claudecode warning) are suppressed. Fatal
errors are always emitted regardless of `--quiet`. Command preview is handled exclusively by
`--trace` (not tied to any verbosity level). The former `--verbosity <0-5>` parameter was
removed (TSK-337) as an anti-pattern: a 0–5 numeric scale bundles independent output concerns
into one opaque integer, violating CLI composability.

### D3 — Print mode requires a message

Fail fast with a clear error if print mode is active without a message. Silent no-op
would be confusing; claude in print mode without input produces nothing useful. Print
mode is triggered by: message present (default) or explicit `-p`/`--print`. Either
way, a message is required.

### D4 — Positional args joined as message

Multiple positional arguments are joined with spaces: `clr Fix the bug` becomes
message `"Fix the bug"`. Standard CLI convention (like `git commit -m`). Eliminates the need
to quote simple messages.

### D5 — Unknown flags rejected

Explicit whitelist of known flags. Unknown flags produce an error with `--help` hint.
Prevents typos from being silently ignored and avoids accidental passthrough to claude.

### D6 — Duplicate value-flags: last wins

When a flag like `--model` appears twice, the last value wins. Matches curl/git convention.
Enables wrapper scripts to override defaults.

### D7 — Hand-rolled parser over clap/unilang

Hand-rolled parser. Zero external dependencies for CLI parsing. Exact control over
error messages and behavior. The flag surface (24 flags + 1 positional) is small enough
that a framework adds complexity without benefit.

### D9 — Session continuation by default

Behavioral specification: [invariant/001_default_flags.md](invariant/001_default_flags.md).

**Rationale:** `clr` adds value over the raw `claude` binary by managing
session continuity automatically. Most invocations are continuations of ongoing work.
Users who want a genuinely fresh start opt in explicitly with `--new-session`.
This also decouples `clr` from external session orchestration.

**Consequence:** removed `-c`/`--continue` from public flag list (redundant); added
`--new-session` (the only way to disable default continuation). Net: 11 flags → 11 flags.

**Fixed (BUG-214, 2026-05-28; reopened and re-fixed 2026-06-03):** The "most invocations are continuations" assumption was false on first use. When no prior session existed in storage, `-c` caused the claude binary to exit immediately with "No conversation found to continue". Fixed by adding `session_exists()` guard in `build_claude_command()`: `-c` is now injected only when session storage is non-empty. Initial fix used `$HOME/.claude/` (always non-empty — contains credentials and config). Re-fix uses `claude_storage_core::continuation::check_continuation()` which checks the correct project-specific path `$HOME/.claude/projects/{encoded(cwd)}/`.

### D8 — Three-layer docs/cli/ replaces 42-file structure

The previous `docs/cli/` contained 42 files documenting `param::value` syntax. Restored as
a proper three-layer reference (command/, param/, type/) with parameter groups,
dictionary, and user stories (originally workflow_scenario.md; migrated to user_story/ in a subsequent pass) — adapted to the new `--flag value` syntax. Extended to L4 in a
subsequent pass: tests/docs/cli/ added with per-command, per-param, per-type, and per-group test case coverage.

### D11 — Print by default when message given; `--interactive` to opt into TTY

When `[MESSAGE]` is provided, `clr` defaults to print mode (captured stdout via
`execute()` + `--print`). Interactive TTY passthrough requires explicit `--interactive`.

**Rationale:** The primary use of `clr "message"` is scripting and automation — piping
output, capturing into variables, chaining with other tools. Interactive TTY passthrough
is the minority case when a message is given. Defaulting to print mode avoids forcing
users to remember `-p` for every scripted invocation and aligns with shell expectations
(running a command with an argument should produce capturable output).

**Consequence:** `clr "Fix bug"` now behaves like `clr -p "Fix bug"` did before.
The `-p`/`--print` flag is kept as a backward-compatible explicit alias. The new
`--interactive` flag opts into TTY passthrough when a message is given. Bare `clr`
(no message) still opens the interactive REPL as before.

### D12 — Expose `--system-prompt` (replace) despite capability loss

Both `--system-prompt` (replace) and `--append-system-prompt` (extend) are exposed,
even though `--system-prompt` strips Claude Code's behavioral guardrails.

**Why expose the destructive option:** Specialized single-purpose agents need complete
control. A coding assistant locked to Rust, a JSON-only responder, a domain-specific
tool — these require a clean prompt slate, not additions on top of general-purpose
coding instructions. The flag is not a mistake; it's a deliberate escape hatch.

**What actually survives replacement:** Tool definitions (~12,000 tokens: Bash, Read,
Write, Edit, Glob, Grep, WebFetch, etc.) are injected into the assembled system prompt
before the replacement is applied. Tools remain fully operational. What is lost is
the behavioral layer: coding guidelines, git safety rules, CLAUDE.md-handling
instructions, output style. Claude gets raw tool access with no behavioral scaffolding.

**Recommendation in docs:** `--append-system-prompt` is documented as the default
recommendation. `--system-prompt` is documented as an explicit opt-in for full-control
scenarios, with the capability table in `command/01_run.md` (Notes) making the tradeoffs
visible.

**Note on CLI vs SDK distinction:** This behavior applies to the CLI `--system-prompt`
flag. The Agent SDK `systemPrompt:` parameter has different semantics — tools may not
be automatically preserved without using `preset: "claude_code"`. The CLI always
preserves tool definitions regardless of replacement.

### D10 — Binary named `clr`, crate named `claude_runner`

The installed binary is `clr`; the Rust crate/lib remains `claude_runner`.

**Rationale:** `clr` is short and fast to type — the tool is used interactively
many times per session (mirrors the `cm` convention of `claude_version`). The crate
name stays `claude_runner` so existing `use claude_runner::COMMANDS_YAML` consumers
are unaffected; only the `[[bin]] name` in `Cargo.toml` changes.

**Consequence:** `cargo install --path .` installs `clr`; `CARGO_BIN_EXE_clr` is
used in integration tests; all docs and help text show `clr`.

### D13 — Commands are bare words, not `--` flags

Behavioral specification: [invariant/003_command_naming.md](invariant/003_command_naming.md).

**Rationale:** Commands select a mode of operation (`run`, `isolated`, `refresh`,
`help`). Parameters modify that mode (`--model`, `--creds`, `--trace`). Mixing these
namespaces (e.g. `--help` as the only way to invoke help) breaks the lexical distinction
and confuses shell completers, subcommand typo detection, and user mental models.

**Consequence:** `help` is now invocable as `clr help` (bare word subcommand). `--help`
and `-h` remain as parameter aliases for POSIX compliance. `KNOWN_SUBCOMMANDS` includes
`"help"` alongside `"isolated"` and `"refresh"`.

### D15 — `render_summary()` gates on invariant field `type=="result"`, not optional fields

`render_summary()` uses `"type":"result"` as its primary gate condition. Optional fields
such as `session_id` are extracted with `.unwrap_or_default()` when absent.

**Rationale:** The `claude --output-format json` envelope schema varies by binary version.
At least one observed version emits a minimal 7-field envelope without `session_id`:
`{"type":"result","subtype":"success","is_error":false,"duration_ms":N,"duration_api_ms":N,"num_turns":N,"result":"..."}`.
Gating on any optional field causes `render_summary()` to return `None` for that variant,
silently restoring the raw-JSON fallback symptom that summary rendering was intended to fix
(BUG-310; structural recurrence of BUG-309 which gated on `"id"`).

**Pitfall:** BUG-309's fix replaced the `"id"` gate with `"session_id"` — same `?`-gate
mechanism, different optional field. This inherited the same structural fragility. Any
future change to the gate field must use a field guaranteed present in ALL CLR result
envelopes across all claude binary versions.

**Invariant field:** `"type":"result"` is present in every CLR result envelope observed
across all tested claude binary versions. It is the only reliable gate.

**Consequence:** `render_summary()` returns `None` only for non-CLR-result JSON
(envelope lacks `"type":"result"`) or non-JSON input — not for CLR envelopes that
omit optional fields like `session_id`, `usage`, or `total_cost_usd`.

### D14 — Dedicated `refresh` command vs reusing `isolated`

**Rationale:** `clr isolated` is designed for running real tasks in credential isolation.
Credential refresh is a distinct operational intent: no user task, no output, just token
renewal. Encoding the `["--print", "."]` invocation trick inside `isolated` forces users
to know the implementation detail. A dedicated `clr refresh` makes intent self-documenting.

**Consequence:** `clr refresh --creds <FILE>` wraps `run_isolated()` with fixed args
`["--print", "."]`. Default timeout is 45s (vs 30s for `isolated`) to accommodate slow
OAuth token exchange. Exit 0 means credentials were refreshed; exit 1 means error or
no refresh; exit 2 means timeout without refresh.
