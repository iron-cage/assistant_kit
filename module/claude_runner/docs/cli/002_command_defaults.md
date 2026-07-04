# Command Defaults

### Scope

- **Purpose**: Cross-command parameter default matrix and isolated/refresh injection defaults.
- **Responsibility**: Document what each command injects by default; record resolved design issues and implemented changes for historical reference.
- **In Scope**: default injection behavior for all 4 commands; resolved design issues (I1–I7); implemented changes (S1–S7); isolated CLAUDE.md content.
- **Out of Scope**: Individual parameter docs (→ `param/`); implementation design (→ `../feature/`).

---

### Parameter Matrix

Rows are parameters or behaviors. Columns are the four commands. Key: ✅ = active, ⬜ = not injected, ➖ = not applicable.

| Parameter | run | ask | isolated | refresh |
|-----------|-----|-----|----------|---------|
| **mode** | print if message present; else interactive | print (always) | print (always) | print (always, message `"."`) |
| **message** | user-supplied positional | user-supplied positional | user-supplied positional (optional) | `"."` hardcoded |
| **model** | user-specified; none = claude binary default | user-specified; none = claude binary default | `"opus"` (`ISOLATED_DEFAULT_MODEL`) | `"claude-sonnet-5"` (`REFRESH_DEFAULT_MODEL`) |
| `--effort` | `max` (default; `--no-effort-max` opts out; `--effort <level>` overrides) | `max` (same) | `max` (injected) | `low` (injected) |
| `ultrathink` suffix | appended to message (unless `--no-ultrathink` or already present) | appended | ➖ not injected | ➖ not injected |
| `-c` (continue) | injected when session exists and not `--new-session` | injected when session exists | ➖ not injected | ➖ not injected |
| `--dangerously-skip-permissions` | ON (unless `--no-skip-permissions`) | ON (unless `--no-skip-permissions`) | ON when message present | ➖ not applicable (no tool use) |
| `--no-session-persistence` | opt-in via `--no-persist` | opt-in via `--no-persist` | always injected | always injected |
| CLAUDE.md (global) | `~/.claude/CLAUDE.md` from user HOME | `~/.claude/CLAUDE.md` from user HOME | written to `<temp_home>/.claude/CLAUDE.md` | written to `<temp_home>/.claude/CLAUDE.md` |
| `--chrome` | ON interactive / OFF print (BUG-304; `--no-chrome` opts out) | OFF (always print — BUG-304) | ON (ClaudeCommand default) | OFF (`--no-chrome` injected) |
| `env -u CLAUDECODE` | ON (unless `--keep-claudecode`) | ON (unless `--keep-claudecode`) | ON (ClaudeCommand default) | ON (ClaudeCommand default) |
| `CLAUDE_CODE_MAX_OUTPUT_TOKENS` | `200,000` | `200,000` | `200,000` | `200,000` |
| `CLAUDE_CODE_AUTO_COMPACT_WINDOW` | `200,000` (`--no-compact-window` opts out) | `200,000` (same) | `200,000` (same) | `200,000` (same) |
| `CLAUDE_CODE_AUTO_CONTINUE` | `true` | `true` | `true` | `true` |
| `CLAUDE_CODE_TELEMETRY` | `false` | `false` | `false` | `false` |
| `CLAUDE_CODE_BASH_TIMEOUT` | `3,600,000 ms` (1 h) | `3,600,000 ms` | `3,600,000 ms` | `3,600,000 ms` |
| `CLAUDE_CODE_BASH_MAX_TIMEOUT` | `7,200,000 ms` (2 h) | `7,200,000 ms` | `7,200,000 ms` | `7,200,000 ms` |
| `--timeout` | `3600 s` (print-mode) / `0` (interactive); `0` = unlimited | same | `30 s` default; `0` = unlimited (no watchdog) | `45 s` default; `0` = unlimited (no watchdog) |
| passthrough args (`--`) | ➖ not supported | ➖ not supported | ✅ collected verbatim after `--` | ➖ not supported |
| `--output-file` | ✅ supported | ✅ supported | ➖ not supported | ➖ not supported |
| `--expect` / `--expect-strategy` | ✅ supported | ✅ supported | ➖ not supported | ➖ not supported |
| `--max-sessions` | ✅ supported | ✅ supported | ➖ not supported | ➖ not supported |
| `--retry-on-transient` / `--transient-delay` (+ all retry params) | ✅ supported | ✅ supported | ➖ not supported | ➖ not supported |
| `--dir` / `--subdir` / `--session-dir` | ✅ supported | ✅ supported | ➖ not supported | ➖ not supported |
| `--system-prompt` / `--append-system-prompt` | ✅ supported | ✅ supported | via passthrough only | ➖ not supported |
| `--json-schema` / `--mcp-config` | ✅ supported | ✅ supported | via passthrough only | ➖ not supported |
| `--args-file` / `CLR_ARGS_FILE` / stdin JSON | ✅ supported | ✅ supported | ✅ supported | ✅ supported |

---

### Design Issues (all resolved — Plan 009)

| ID | Issue | Commands | Resolution |
|----|-------|----------|------------|
| I1 | `--effort` not injected | isolated, refresh | ✅ `--effort max` for isolated, `--effort low` for refresh (S1) |
| I7 | Model was hardcoded string for isolated | isolated | ✅ Changed to `ISOLATED_DEFAULT_MODEL = "opus"`; `REFRESH_DEFAULT_MODEL = "claude-sonnet-5"` added for refresh (S7) |
| I2 | `--timeout 0` = immediate kill | isolated, refresh | ✅ Fixed: `0` = unlimited (no watchdog), matching run/ask semantics (S2) |
| I3 | `--no-session-persistence` not injected | isolated, refresh | ✅ Always injected for both commands (S3) |
| I4 | `--chrome` injected for refresh | refresh | ✅ `--no-chrome` injected for refresh (S4) |
| I5 | `--dangerously-skip-permissions` not injected | isolated | ✅ Injected when message present (S5) |
| I6 | No CLAUDE.md in isolated temp HOME | isolated, refresh | ✅ Written to `<temp_home>/.claude/CLAUDE.md` before spawn (S6) |

---

### Implemented Changes (all applied — Plan 009)

| # | Change | Affected code | Implemented behavior |
|---|--------|---------------|----------------------|
| S1 | Inject `--effort max` for isolated; `--effort low` for refresh | `credential.rs::run_isolated_command()` — `effort: EffortLevel` param | ✅ Isolated passes `Max`, refresh passes `Low` |
| S7 | `ISOLATED_DEFAULT_MODEL` = `"opus"`; `REFRESH_DEFAULT_MODEL` = `"claude-sonnet-5"` | `isolated.rs` constants | ✅ `ISOLATED_DEFAULT_MODEL = "opus"`, `REFRESH_DEFAULT_MODEL = "claude-sonnet-5"` |
| S2 | Fix `--timeout 0` semantics | `isolated.rs::run_isolated()` — `Option<Instant>` deadline | ✅ `None` when `timeout_secs == 0` = no watchdog |
| S3 | Inject `--no-session-persistence` | `credential.rs::run_isolated_command()` — prepended to args vec | ✅ Always injected for both commands |
| S4 | Suppress `--chrome` for refresh | `credential.rs::run_isolated_command()` — `no_chrome: bool` param | ✅ `--no-chrome` prepended for refresh |
| S5 | Inject `--dangerously-skip-permissions` for isolated | `credential.rs::run_isolated_command()` — `skip_perms: bool` param | ✅ Injected when `message.is_some()` |
| S6 | Write CLAUDE.md to isolated temp HOME | `isolated.rs::run_isolated()` — write before spawn | ✅ Written to `claude_dir/CLAUDE.md` |

#### Injection Order Convention

Injected defaults (S1, S3, S5) are prepended **before** `--print`, message, and passthrough args in the args vec. Claude binary processes flags left-to-right with last-wins semantics, so passthrough overrides:

```sh
# Override injected --effort max with medium for a lighter isolated task:
clr isolated "summarize this file" -- --effort medium
```

---

### Isolated CLAUDE.md Content (S6)

Written to `<temp_home>/.claude/CLAUDE.md` by `run_isolated()` before subprocess spawn.

```markdown
# Isolated subprocess

Execute the given task immediately and exit.

- Do not ask clarifying questions — act on the message as given.
- Do not request human confirmation for any operation.
- Do not explain your reasoning or narrate your steps.
- Output only the direct result of the task; no preamble, no summary.
- If the input is a single character or whitespace only, reply with a single period.
```

#### Design Notes

- This CLAUDE.md lives at `<temp_home>/.claude/CLAUDE.md` — the global user config path within the isolated HOME. It replaces the user's personal `~/.claude/CLAUDE.md` for the subprocess.
- The project-level CLAUDE.md (if the working directory has one) is still read normally — this only controls global user instructions.
- Keeping the content minimal avoids surprising interactions with passthrough args or system prompt flags.

---

### Cross-References

| Type | Path | Responsibility |
|------|------|----------------|
| source | `src/cli/credential.rs` | `run_isolated_command()`, `run_refresh_command()` — entry points for S1–S5 |
| source | `module/claude_runner_core/src/isolated.rs` | `run_isolated()` — entry point for S2, S6 |
| source | `module/claude_runner_core/src/command/mod.rs` | `ClaudeCommand::new()` defaults (chrome, tokens, env vars) |
| param | `param/036_timeout.md` | `--timeout` semantics for run/ask |
| param | `param/022_no_persist.md` | `--no-persist` / `--no-session-persistence` for run/ask |

### Provenance

| File | Notes |
|------|-------|
| [../command_defaults.md](command_defaults.md) | Original un-migrated source; retained as reference |
