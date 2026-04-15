# Design Decisions: clr

Rationale for key design choices made during the `--flag value` CLI redesign (task 031).

## D1 — Interactive REPL when no message; print by default when message given

Running `clr` without a message opens an interactive REPL via `execute_interactive()`.
When a message is provided, `clr` defaults to print mode (`execute()` + `--print`) —
capturing stdout for pipeline/scripting use. Use `--interactive` to opt into TTY
passthrough when a message is given.

## D2 — `--verbose` vs `--verbosity`

`--verbose` passes through to claude (it's a claude-native flag). `--verbosity <0-5>` controls
the runner's own output gating level. At `--verbosity 4` (verbose detail), the runner prints
a command preview to stderr before execution.

## D3 — Print mode requires a message

Fail fast with a clear error if print mode is active without a message. Silent no-op
would be confusing; claude in print mode without input produces nothing useful. Print
mode is triggered by: message present (default) or explicit `-p`/`--print`. Either
way, a message is required.

## D4 — Positional args joined as message

Multiple positional arguments are joined with spaces: `clr Fix the bug` becomes
message `"Fix the bug"`. Standard CLI convention (like `git commit -m`). Eliminates the need
to quote simple messages.

## D5 — Unknown flags rejected

Explicit whitelist of known flags. Unknown flags produce an error with `--help` hint.
Prevents typos from being silently ignored and avoids accidental passthrough to claude.

## D6 — Duplicate value-flags: last wins

When a flag like `--model` appears twice, the last value wins. Matches curl/git convention.
Enables wrapper scripts to override defaults.

## D7 — Hand-rolled parser over clap/unilang

Hand-rolled parser. Zero external dependencies for CLI parsing. Exact control over
error messages and behavior. The flag surface (15 flags + 1 positional) is small enough
that a framework adds complexity without benefit.

## D9 — Session continuation by default

Every invocation passes `-c` to claude unless `--new-session` is given. The
explicit `-c`/`--continue` flag is removed from the public CLI surface — it is
now internalized as the default.

**Rationale:** `clr` adds value over the raw `claude` binary by managing
session continuity automatically. Most invocations are continuations of ongoing work.
Users who want a genuinely fresh start opt in explicitly with `--new-session`.
This also decouples `clr` from external session orchestration (formerly
`dream_agent`'s responsibility).

**Consequence:** removed `-c`/`--continue` from public flag list (redundant); added
`--new-session` (the only way to disable default continuation). Net: 11 flags → 11 flags.

## D8 — Three-layer docs/cli/ replaces 42-file structure

The previous `docs/cli/` contained 42 files documenting `param::value` syntax. Restored as
a proper three-layer reference (commands.md, params.md, types.md) with parameter groups,
dictionary, and workflows — adapted to the new `--flag value` syntax. Extended to L5 in a
subsequent pass: testing/ added with per-command and per-param test case coverage.

## D11 — Print by default when message given; `--interactive` to opt into TTY

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

## D12 — Expose `--system-prompt` (replace) despite capability loss

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
scenarios, with the capability table in `parameter_interactions.md` making the tradeoffs
visible.

**Note on CLI vs SDK distinction:** This behavior applies to the CLI `--system-prompt`
flag. The Agent SDK `systemPrompt:` parameter has different semantics — tools may not
be automatically preserved without using `preset: "claude_code"`. The CLI always
preserves tool definitions regardless of replacement.

## D10 — Binary named `clr`, crate named `claude_runner`

The installed binary is `clr`; the Rust crate/lib remains `claude_runner`.

**Rationale:** `clr` is short and fast to type — the tool is used interactively
many times per session (mirrors the `cm` convention of `claude_version`). The crate
name stays `claude_runner` so existing `use claude_runner::COMMANDS_YAML` consumers
are unaffected; only the `[[bin]] name` in `Cargo.toml` changes.

**Consequence:** `cargo install --path .` installs `clr`; `CARGO_BIN_EXE_clr` is
used in integration tests; all docs and help text show `clr`.
