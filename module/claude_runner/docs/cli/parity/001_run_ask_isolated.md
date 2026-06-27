# Parity: run / ask / isolated

### Scope

- **Purpose**: Comprehensive behavioral parity comparison of `clr run`, `clr ask`, and `clr isolated`.
- **Responsibility**: Document every behavioral dimension where these three commands differ or agree.
- **In Scope**: Param surface, auto-injections, execution modes, session/credential handling, retry system, output control, exit codes.
- **Out of Scope**: `clr refresh` (-> `002_isolated_refresh.md`); implementation internals (-> `docs/feature/001_runner_tool.md`).

---

### Comparison Matrix

| Dimension | `run` | `ask` | `isolated` |
|-----------|-------|-------|------------|
| **--- Identity ---** | | | |
| Purpose | General-purpose Claude execution | Semantic alias signalling "question" intent | Credential-isolated one-shot execution |
| Relation | Canonical default command | Pure alias for `run` (identical code path) | Distinct command |
| Is default subcommand? | Yes (`clr MSG` = `clr run MSG`) | No | No |
| Syntax | `clr [OPTIONS] [MESSAGE]` / `clr run …` | `clr ask [OPTIONS] [MESSAGE]` | `clr isolated [--creds F] [--timeout N] [MESSAGE]` |
| Behavioral difference from `run` | — | None | Significant (see below) |
| **--- Param Surface ---** | | | |
| Full param set (67+ params) | Yes | Yes (identical) | No — minimal set only |
| Param count | All 67+ | All 67+ | 12: `MESSAGE`, `--creds`, `--timeout`, `--trace`, `--dry-run`, `--dir`, `--add-dir`, `--file`, `--expect`, `--expect-strategy`, `--journal`, `--journal-dir` |
| Passthrough override (`-- <args>`) | No | No | Yes (e.g., `-- --effort medium`) |
| **--- Session ---** | | | |
| Session continuation (`-c`) | Yes — auto (last session in dir) | Yes — auto | No (temp HOME has no history) |
| `--new-session` suppresses `-c` | Yes | Yes | N/A |
| Session persistence | On (default) | On (default) | Always off (`--no-session-persistence` injected) |
| `--no-persist` opt-out | Yes | Yes | No (always no-persist) |
| HOME directory | Real `$HOME` | Real `$HOME` | Fresh temp dir (deleted after subprocess exits) |
| Temp HOME contents | — | — | `.claude/.credentials.json` + minimal `CLAUDE.md` |
| **--- Auto-Injections ---** | | | |
| `env -u CLAUDECODE claude …` | Yes (suppressed by `--keep-claudecode`) | Yes | Yes |
| `-c` (continue session) | Yes (suppressed by `--new-session`) | Yes | No |
| `--dangerously-skip-permissions` | Yes — **always** | Yes — **always** | Yes — only when MESSAGE present; No in no-message REPL mode |
| `--effort max` | Yes (suppressed by `--no-effort-max`) | Yes | Yes (no suppression flag available) |
| `--chrome` | Yes in interactive / No in print (auto-suppressed; `--no-chrome` opt-out) | No (always print) | Yes (ClaudeCommand default; no suppression flag) |
| Ultrathink suffix on MESSAGE | Yes (suppressed by `--no-ultrathink`) | Yes | No |
| `--no-session-persistence` | Via `--no-persist` flag | Via `--no-persist` | Always injected |
| Default model injection | No (uses claude binary default) | No | Yes — `claude-opus-4-6` (`ISOLATED_DEFAULT_MODEL`) |
| Minimal `CLAUDE.md` written to HOME | No | No | Yes (instructs: execute immediately, no clarifying questions, no confirmation) |
| `CLAUDE_CODE_MAX_OUTPUT_TOKENS` | `200,000` | `200,000` | `200,000` |
| **--- Execution Modes ---** | | | |
| No-message -> REPL | Yes — interactive | Yes — interactive | Yes — interactive (no `--dangerously-skip-permissions`) |
| With-message -> print (auto) | Yes | Yes | Yes (+ `--dangerously-skip-permissions`) |
| `--interactive` with message opt-in | Yes | Yes | No (flag not available) |
| `--dry-run` preview | Yes | Yes | Yes |
| `--trace` | Yes | Yes | Yes |
| **--- Timeout ---** | | | |
| Default timeout (print mode) | `3600 s` (`DEFAULT_PRINT_TIMEOUT_SECS`) | `3600 s` | `30 s` |
| Default timeout (interactive / no-message) | `0` (unlimited) | `0` (unlimited) | `0` (unlimited when no message) |
| `--timeout 0` = unlimited | Yes | Yes | Yes |
| Exit code on timeout | `4` (CLR watchdog) | `4` | `2` — unless creds refreshed before timeout -> `0` |
| Creds-refreshed-before-timeout path | No | No | Yes — exits `0` (not `2`) if OAuth refresh completed before deadline |
| **--- Retry System ---** | | | |
| 3-tier retry system | Yes — full | Yes — full | No — none |
| 8 error classes | Yes (Transient/Account/Auth/Service/Process/Validation/Runner/Unknown) | Yes | No |
| `--retry-override` (Tier 1) | Yes | Yes | No |
| Per-class `--retry-on-*` (Tier 2) | Yes (8 params) | Yes | No |
| `--retry-default` / `--retry-default-delay` (Tier 3) | Yes (default: 2 / 30s) | Yes | No |
| **--- Concurrency Gate ---** | | | |
| `--max-sessions` gate | Yes (default: 30) | Yes | No |
| Blocks when gate hit | Yes (waits for slots) | Yes | No |
| **--- Credentials ---** | | | |
| `--creds` flag | No | No | Yes (default: `~/.claude/.credentials.json`) |
| Credential isolation (temp HOME) | No | No | Yes |
| Credential writeback on OAuth refresh | No | No | Yes (written back to `--creds` in-place) |
| Temp HOME cleanup | No | No | Yes (unconditional delete after subprocess exits) |
| **--- Model & Effort ---** | | | |
| `--model` flag | Yes | Yes | No (override via passthrough `-- --model`) |
| Default model | claude binary default | claude binary default | `claude-opus-4-6` (hardcoded) |
| `--fallback-model` | Yes | Yes | No |
| `--effort` flag | Yes | Yes | No (override via passthrough `-- --effort`) |
| Default effort | `max` (injected) | `max` (injected) | `max` (always injected) |
| `--no-effort-max` suppress | Yes | Yes | No |
| **--- Output ---** | | | |
| `--output-style` (summary/raw) | Yes | Yes | No |
| `--summary-fields` | Yes | Yes | No |
| `--output-file` (tee to file) | Yes | Yes | No |
| `--output-format` (text/json/stream-json) | Yes | Yes | No |
| `--strip-fences` | Yes | Yes | No |
| **--- Validation ---** | | | |
| `--expect` / `--expect-strategy` | Yes | Yes | Yes (`fail` + `default:<V>` only; `retry` not supported → exit 1) |
| **--- Input ---** | | | |
| `MESSAGE` positional | Yes | Yes | Yes (optional) |
| `--file` (pipe file as stdin) | Yes | Yes | Yes |
| `--json-schema` | Yes | Yes | No (override via passthrough) |
| `--mcp-config` | Yes | Yes | No (override via passthrough) |
| **--- Directory ---** | | | |
| `--dir` (working directory) | Yes (default: cwd) | Yes | Yes (validated before spawn; `CLR_DIR` env fallback) |
| `--subdir` | Yes | Yes | No |
| `--add-dir` | Yes | Yes | Yes (repeatable; `CLR_ADD_DIR` env fallback) |
| `--session-dir` | Yes | Yes | No |
| **--- System Prompt ---** | | | |
| `--system-prompt` | Yes | Yes | No (override via passthrough) |
| `--append-system-prompt` | Yes | Yes | No |
| **--- Tools & Budget ---** | | | |
| `--allowed-tools` / `--disallowed-tools` | Yes | Yes | No |
| `--max-budget-usd` | Yes | Yes | No |
| `--max-turns` | Yes | Yes | No |
| **--- Verbosity ---** | | | |
| `--verbose` | Yes | Yes | No |
| `--verbosity` (gate level) | Yes (default: 3) | Yes | No |
| **--- Journal ---** | | | |
| `--journal` / `--journal-dir` | Yes | Yes | Yes |
| **--- Exit Codes ---** | | | |
| `0` success | Yes | Yes | Yes |
| `1` error (parse/spawn/I/O) | Yes | Yes | Yes |
| `2` rate-limit or transient retries exhausted | Yes | Yes | Yes — means **timeout** here (not rate-limit) |
| `3` expect mismatch | Yes | Yes | Yes (`--expect` mismatch with `fail` strategy) |
| `4` CLR watchdog timeout | Yes | Yes | No (isolated timeout -> exit `2`) |
| `N` subprocess passthrough | Yes | Yes | Yes |
| `128+signal` (POSIX) | Yes | Yes | Yes (POSIX passthrough — same semantics as `N`) |
| **--- Param Groups ---** | | | |
| Claude-Native Flags | Yes | Yes | No |
| Runner Control | Yes | Yes | No |
| System Prompt | Yes | Yes | No |
| Credential Operations | No | No | Yes |

---

### Key Takeaways

- `run` vs `ask` — zero behavioral difference; `ask` is a pure documentation signal for "this is a question".
- `isolated` shares 9 params with `run`/`ask` (`MESSAGE`, `--trace`, `--dry-run`, `--dir`, `--add-dir`, `--file`, `--expect`, `--expect-strategy`, `--journal*`); everything else is stripped down or hardcoded.
- The defining `isolated`-specific behaviors: temp HOME lifecycle, credential writeback, timeout exits as `2` (not `4`), and `--dangerously-skip-permissions` conditional on MESSAGE presence.
- `isolated` parameter overrides are only possible via `-- <passthrough-args>` syntax; no native flags for model/effort/tools.
- Both `run` and `ask` suppress `--chrome` automatically in print mode (BUG-304 fix); `isolated` never suppresses it.

---

### Planned Gap Closures

The following gaps between `isolated` and `run`/`ask` are tracked as implementation tasks. Each gap was qualified as **Actual Gap** (not by design) in the gap analysis.

| Task | Gap | Dimension | `isolated` After Closure |
|------|-----|-----------|--------------------------|
| TSK-328 ✅ | `--dry-run` not available | Execution Modes | Preview injected command without spawning subprocess |
| TSK-329 ✅ | `--dir` / `--add-dir` not available | Directory | Set working directory and grant read access to additional paths |
| TSK-330 ✅ | `--file` not available | Input | Pipe a file as stdin to the isolated subprocess |
| TSK-331 ✅ | `--expect` / `--expect-strategy` (fail + default) not available | Validation | Assert output matches expected pattern; exit 3 on mismatch |
| TSK-332 | `--output-file`, `--strip-fences`, `--output-style`, `--summary-fields` not available | Output | Tee output to file, strip code fences, render summary, select fields |
| TSK-333 | `--verbosity`, `--keep-claudecode` not available | Runner Control | Gate CLR verbose logging; suppress env -u CLAUDECODE injection |

---

### Cross-References

| Type | Path | Responsibility |
|------|------|----------------|
| command | `command/01_run.md` | `run` full reference |
| command | `command/05_ask.md` | `ask` reference (alias) |
| command | `command/02_isolated.md` | `isolated` full reference |
| doc | `command_defaults.md` | Injection defaults with Plan 009 design traceability |
| parity | `002_isolated_refresh.md` | `isolated` vs `refresh` credential command comparison |
| invariant | `../invariant/005_isolated_subprocess_defaults.md` | Isolated subprocess injection contracts |
