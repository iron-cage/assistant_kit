# Feature: CLI Design

### Scope

- **Purpose**: Document the design rationale behind the `clr` CLI's `--flag value` syntax, flag surface, and parsing behavior.
- **Responsibility**: Describe why the flag-based syntax was chosen over alternatives, how flags and commands are parsed, and the reasoning behind individual flag-level design choices.
- **In Scope**: `--flag value` syntax rationale, hand-rolled parser rationale, flag precedence rules, command-vs-flag namespace separation, print/interactive mode defaults, system prompt flag exposure, binary/crate naming, `refresh` command rationale, `render_summary()` gate field choice.
- **Out of Scope**: Individual command reference (→ `../cli/command/`), individual parameter reference (→ `../cli/param/`), default flag values (→ `invariant/001_default_flags.md`), CLI reference surface — syntax tables, type definitions, parameter defaults (→ `../cli/`).

### Design

**Command syntax:** `clr` uses bare-word subcommands (`run`, `isolated`, `refresh`, `help`, `ps`, `kill`, `tools`, `ask`, `scope`) followed by `--flag value` pairs and an optional positional message. This mirrors POSIX CLI convention (like `git`) rather than a unilang-style `.command param::value` syntax.

**Parsing:** A hand-rolled parser (no external CLI framework) validates an explicit whitelist of known flags; unknown flags produce an error with a `--help` hint. Positional arguments are joined with spaces to form the message. Duplicate value-flags resolve last-wins (matches curl/git convention).

**Mode selection:** `clr` defaults to print mode when a message is present (captured stdout via `execute()` + `--print`); bare `clr` (no message) opens the interactive REPL. `--interactive` opts into TTY passthrough when a message is given; `-p`/`--print` remains as an explicit alias.

**Session handling:** Session continuation (`-c`) is injected by default when a prior session exists for the effective working directory; `--new-session` is the only way to disable it.

### Features

| File | Relationship |
|------|-------------|
| [feature/001_runner_tool.md](001_runner_tool.md) | Runner tool architecture implementing this CLI design |
| [feature/003_retry_hierarchy.md](003_retry_hierarchy.md) | Retry flag surface governed by these parsing rules |
| [feature/004_json_config.md](004_json_config.md) | JSON config precedence layered under the CLI flag parsing described here |

### Design Decisions

| ID | Decision | Category |
|----|----------|----------|
| D2 | `--verbose` vs `--quiet` (supersedes `--verbosity`) | Parameter Conventions |
| D3 | Print mode requires a message | Behavior |
| D4 | Positional args joined as message | Syntax |
| D5 | Unknown flags rejected | Parsing |
| D6 | Duplicate value-flags: last wins | Parameter Conventions |
| D7 | Hand-rolled parser over clap/unilang | Parsing |
| D9 | Session continuation by default | Behavior |
| D10 | Binary named `clr`, crate named `claude_runner` | Naming |
| D11 | Print by default when message given; `--interactive` to opt into TTY | Behavior |
| D12 | Expose `--system-prompt` (replace) despite capability loss | Parameter Conventions |
| D13 | Commands are bare words, not `--` flags | Syntax |
| D14 | Dedicated `refresh` command vs reusing `isolated` | Behavior |
| D15 | `render_summary()` gates on invariant field `type=="result"`, not optional fields | Pipeline |

Decisions by concern area: **Syntax**: D4, D13 | **Parsing**: D5, D7 | **Parameter Conventions**: D2, D6, D12 | **Behavior**: D3, D9, D11, D14 | **Naming**: D10 | **Pipeline**: D15

**D9 — Session continuation by default:** Behavioral specification: [invariant/001_default_flags.md](../invariant/001_default_flags.md).

`clr` adds value over the raw `claude` binary by managing session continuity automatically. Most invocations are continuations of ongoing work. Users who want a genuinely fresh start opt in explicitly with `--new-session`. This also decouples `clr` from external session orchestration. Consequence: `-c`/`--continue` was removed from the public flag list (redundant); `--new-session` was added as the only way to disable default continuation. Net: 11 flags → 11 flags.

**Fixed (BUG-214, 2026-05-28; reopened and re-fixed 2026-06-03):** The "most invocations are continuations" assumption was false on first use. When no prior session existed in storage, `-c` caused the claude binary to exit immediately with "No conversation found to continue". Fixed by adding `session_exists()` guard in `build_claude_command()`: `-c` is now injected only when session storage is non-empty. Initial fix used `$HOME/.claude/` (always non-empty — contains credentials and config). Re-fix uses `claude_storage_core::continuation::check_continuation()` which checks the correct project-specific path `$HOME/.claude/projects/{encoded(cwd)}/`.

**D12 — Expose `--system-prompt` (replace) despite capability loss:** Both `--system-prompt` (replace) and `--append-system-prompt` (extend) are exposed, even though `--system-prompt` strips Claude Code's behavioral guardrails.

Specialized single-purpose agents need complete control. A coding assistant locked to Rust, a JSON-only responder, a domain-specific tool — these require a clean prompt slate, not additions on top of general-purpose coding instructions. The flag is not a mistake; it's a deliberate escape hatch.

What actually survives replacement: Tool definitions (~12,000 tokens: Bash, Read, Write, Edit, Glob, Grep, WebFetch, etc.) are injected into the assembled system prompt before the replacement is applied. Tools remain fully operational. What is lost is the behavioral layer: coding guidelines, git safety rules, CLAUDE.md-handling instructions, output style. Claude gets raw tool access with no behavioral scaffolding.

`--append-system-prompt` is documented as the default recommendation. `--system-prompt` is documented as an explicit opt-in for full-control scenarios, with the capability table in `command/01_run.md` (Notes) making the tradeoffs visible.

**Note on CLI vs SDK distinction:** This behavior applies to the CLI `--system-prompt` flag. The Agent SDK `systemPrompt:` parameter has different semantics — tools may not be automatically preserved without using `preset: "claude_code"`. The CLI always preserves tool definitions regardless of replacement.

**D15 — `render_summary()` gates on invariant field `type=="result"`, not optional fields:** `render_summary()` uses `"type":"result"` as its primary gate condition. Optional fields such as `session_id` are extracted with `.unwrap_or_default()` when absent.

The `claude --output-format json` envelope schema varies by binary version. At least one observed version emits a minimal 7-field envelope without `session_id`: `{"type":"result","subtype":"success","is_error":false,"duration_ms":N,"duration_api_ms":N,"num_turns":N,"result":"..."}`. Gating on any optional field causes `render_summary()` to return `None` for that variant, silently restoring the raw-JSON fallback symptom that summary rendering was intended to fix (BUG-310; structural recurrence of BUG-309 which gated on `"id"`).

**Pitfall:** BUG-309's fix replaced the `"id"` gate with `"session_id"` — same `?`-gate mechanism, different optional field. This inherited the same structural fragility. Any future change to the gate field must use a field guaranteed present in ALL CLR result envelopes across all claude binary versions.

**Invariant field:** `"type":"result"` is present in every CLR result envelope observed across all tested claude binary versions. It is the only reliable gate. Consequence: `render_summary()` returns `None` only for non-CLR-result JSON (envelope lacks `"type":"result"`) or non-JSON input — not for CLR envelopes that omit optional fields like `session_id`, `usage`, or `total_cost_usd`.

### Sources

| File | Relationship |
|------|-------------|
| `../../src/cli/parse.rs` | Flag parsing, whitelist validation, last-wins resolution |
| `../../src/cli/builder.rs` | `build_claude_command()` — session continuation guard (D9) |
| `../../src/cli/summary.rs` | `render_summary()` — invariant field gate (D15) |
| `../../src/cli/credential.rs` | `run_isolated_command()`, `run_refresh_command()` (D14) |

### Provenance

| File | Notes |
|------|-------|
| [../001_design_decisions.md](../001_design_decisions.md) | Original informal design-rationale notes; retained as reference |

### Tests

| File | Relationship |
|------|-------------|
| `../../tests/cli_args_test.rs` | Flag parsing, whitelist rejection, last-wins duplicate resolution (D5–D7) |
| `../../tests/cli_args_ext_test.rs` | Extended flag coverage including session continuation guard (D9) |
| `../../tests/refresh_test.rs` | `clr refresh` command behavior (D14) |
| `../../tests/summary_unit_test.rs` | `render_summary()` invariant field gate (D15) |
