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
| Full param set (69+ params) | Yes | Yes (identical) | No — minimal set only |
| Param count | All 69+ | All 69+ | 12: `MESSAGE`, `--creds`, `--timeout`, `--trace`, `--dry-run`, `--dir`, `--add-dir`, `--file`, `--expect`, `--expect-strategy`, `--journal`, `--journal-dir` |
| Passthrough override (`-- <args>`) | No | No | Yes (e.g., `-- --effort medium`) |
| **--- Session ---** | | | |
| Session continuation (`-c`) | Yes — auto (last session in dir) | Yes — auto | No (temp HOME has no history) |
| `--new-session` suppresses `-c` | Yes | Yes | N/A |
| Session persistence | On (default) | On (default) | Always off (`--no-session-persistence` injected) |
| `--no-persist` opt-out | Yes | Yes | No (always no-persist) |
| HOME directory | Real `$HOME` | Real `$HOME` | Fresh temp dir (deleted after subprocess exits) |
| Temp HOME contents | — | — | `.claude/.credentials.json` + minimal `CLAUDE.md` |
| **--- Auto-Injections ---** | | | |
| `env -u CLAUDECODE -u CLAUDE_CODE_CHILD_SESSION claude …` | Yes (`CLAUDECODE` suppressed by `--keep-claudecode`; `CLAUDE_CODE_CHILD_SESSION` unconditional — no suppression flag) | Yes | Yes |
| `-c` (continue session) | Yes (suppressed by `--new-session`) | Yes | No |
| `--dangerously-skip-permissions` | Yes — **always** | Yes — **always** | Yes — only when MESSAGE present; No in no-message REPL mode |
| `--effort max` | Yes (suppressed by `--no-effort-max`) | Yes | Yes (override with `--effort <level>`; suppress entirely with `--no-effort-max`) |
| `--chrome` | Yes in interactive / No in print (auto-suppressed; `--no-chrome` opt-out) | No (always print) | Yes (ClaudeCommand default; suppress with `--no-chrome`) |
| Ultrathink suffix on MESSAGE | Yes (suppressed by `--no-ultrathink`) | Yes | No |
| `--no-session-persistence` | Via `--no-persist` flag | Via `--no-persist` | Always injected |
| Default model injection | Reads `model` from config file (`.clr.toml`/`~/.clr/config.toml`) when set (unless [provider-gated](../config_param.md#provider-gate)); uses claude binary default otherwise | Reads `model` from config file (`.clr.toml`/`~/.clr/config.toml`) when set (unless [provider-gated](../config_param.md#provider-gate)); uses claude binary default otherwise | Reads `model` from config file (`.clr.toml`/`~/.clr/config.toml`) when set; else `opus` alias (`ISOLATED_DEFAULT_MODEL`) |
| Minimal `CLAUDE.md` written to HOME | No | No | Yes (instructs: execute immediately, no clarifying questions, no confirmation) |
| `CLAUDE_CODE_MAX_OUTPUT_TOKENS` | `128,000` | `128,000` | `128,000` |
| **--- Execution Modes ---** | | | |
| No-message -> REPL | Yes — interactive | Yes — interactive | Yes — interactive (no `--dangerously-skip-permissions`) |
| With-message -> print (auto) | Yes | Yes | Yes (+ `--dangerously-skip-permissions`) |
| `--interactive` with message opt-in | Yes | Yes | No (flag not available) |
| `--dry-run` preview | Yes | Yes | Yes |
| `--trace` | Yes | Yes | Yes |
| **--- Timeout ---** | | | |
| Default timeout (print mode) | `0` = unlimited (`DEFAULT_PRINT_TIMEOUT_SECS = 0`, TSK-503) | `0` = unlimited | `30 s` |
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
| `--max-sessions` gate | Yes (default: 8) | Yes | Yes (3-tier: CLI flag + `"max-sessions"` JSON key + `CLR_MAX_SESSIONS` env var; no config-file tier) |
| Blocks when gate hit | Yes (waits for slots) | Yes | Yes (bypassed by `--dry-run`) |
| **--- Credentials ---** | | | |
| `--creds` flag | No | No | Yes (default: `~/.claude/.credentials.json`) |
| Credential isolation (temp HOME) | No | No | Yes |
| Credential writeback on OAuth refresh | No | No | Yes (written back to `--creds` in-place) |
| Temp HOME cleanup | No | No | Yes (unconditional delete after subprocess exits) |
| **--- Model & Effort ---** | | | |
| `--model` flag | Yes | Yes | Yes |
| Default model | config-file `model` if set (unless [provider-gated](../config_param.md#provider-gate)); otherwise claude binary default | config-file `model` if set (unless [provider-gated](../config_param.md#provider-gate)); otherwise claude binary default | config-file `model` if set; otherwise `opus` alias (`ISOLATED_DEFAULT_MODEL`) |
| `--fallback-model` | Yes | Yes | No |
| `--effort` flag | Yes | Yes | Yes |
| Default effort | `max` (injected) | `max` (injected) | `max` (injected; override with `--effort` or suppress with `--no-effort-max`) |
| `--no-effort-max` suppress | Yes | Yes | Yes |
| **--- Output ---** | | | |
| `--output-style` (summary/raw) | Yes | Yes | Yes (default: `raw`, not `summary`) |
| `--summary-fields` | Yes | Yes | Yes |
| `--output-file` (tee to file) | Yes | Yes | Yes |
| `--output-format` (text/json/stream-json) | Yes | Yes | No (passthrough) |
| `--strip-fences` | Yes | Yes | Yes |
| **--- Validation ---** | | | |
| `--expect` / `--expect-strategy` | Yes | Yes | Yes (`fail` + `default:<V>` only; `retry` not supported → exit 1) |
| **--- Input ---** | | | |
| `MESSAGE` positional | Yes | Yes | Yes (optional) |
| `--file` (pipe file as stdin) | Yes | Yes | Yes |
| `--json-schema` | Yes | Yes | Yes |
| `--mcp-config` | Yes | Yes | Yes |
| **--- Directory ---** | | | |
| `--dir` (working directory) | Yes (default: cwd) | Yes | Yes (validated before spawn; `CLR_DIR` env fallback) |
| `--topic` | Yes | Yes | No |
| `--add-dir` | Yes | Yes | Yes (repeatable; `CLR_ADD_DIR` env fallback) |
| `--session-dir` (deprecated, inert) | Yes | Yes | No |
| **--- System Prompt ---** | | | |
| `--system-prompt` | Yes | Yes | Yes |
| `--append-system-prompt` | Yes | Yes | Yes |
| **--- Tools & Budget ---** | | | |
| `--allowed-tools` / `--disallowed-tools` | Yes | Yes | Yes |
| `--max-budget-usd` | Yes | Yes | Yes |
| `--max-turns` | Yes | Yes | Yes |
| **--- Output Suppression ---** | | | |
| `--verbose` | Yes | Yes | No |
| `--quiet` | Yes (default: false) | Yes | No |
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
| Claude-Native Flags | Yes | Yes | Partial (9/14 — model, effort, json-schema, mcp-config, max-turns, allowed-tools, disallowed-tools, max-budget-usd, add-dir) |
| Runner Control | Yes | Yes | Partial (no-effort-max, no-chrome) |
| System Prompt | Yes | Yes | Yes |
| Credential Operations | No | No | Yes |

---

### Key Takeaways

- `run` and `ask` resolve `model` from the config file (project `.clr.toml` overriding user `~/.clr/config.toml`) as a fourth and final tier, when `--model` is absent and `CLR_MODEL` is unset — task 408 removed the `prefs.json` fifth tier that previously ran here (BUG-008's original fix), since it was a no-op for anyone with `config.toml`'s `model` key set (that tier already resolves earlier in the same sequence); any pre-existing `prefs.json` pin was carried forward into `config.toml` once as part of that change. `isolated` resolves its own separate 2-tier cascade via `resolve_isolated_default_model()` (project `.clr.toml` → user `~/.clr/config.toml`, TSK-407/410), falling back to `ISOLATED_DEFAULT_MODEL` (`opus` alias) only when neither tier sets a value — task 410 retired the `prefs.json` tier this cascade previously fell through to. The remaining difference: `isolated` now has `--model` as a native flag with `CLR_MODEL` env fallback (TSK-443), but its terminal fallback is still the `opus` alias rather than the claude binary's own default — `run`/`ask` fall through to the claude binary's default when no config tier sets a value; `isolated` falls through to the `opus` alias. One further asymmetry: while the seat's env block pins `ANTHROPIC_MODEL` (`~/.claude/settings.json`), `run`/`ask`/`topic` ignore the config-tier `model`/`fallback_model` entirely ([Provider Gate](../config_param.md#provider-gate)); `isolated`'s cascade is deliberately unaffected (explicit creds, temp `HOME` strips the env block).
- `run` vs `ask` — zero behavioral difference; `ask` is a pure documentation signal for "this is a question".
- `isolated` shares 21+ params with `run`/`ask` after TSK-443 added 12 native flags; remaining gaps: `--output-format` (passthrough-only), `--verbose`, `--quiet`, `--interactive`, all retry params, and session-control params (`-c`, `--new-session`, `--session-dir`, `--topic`, `--no-persist`) — all excluded by design (see Exclusion Rationale).
- The defining `isolated`-specific behaviors: temp HOME lifecycle, credential writeback, timeout exits as `2` (not `4`), and `--dangerously-skip-permissions` conditional on MESSAGE presence.
- Passthrough (`-- <args>`) remains available for `--output-format` and any other claude-native flag not covered natively; last-wins ordering means passthrough still overrides any native flag.
- Both `run` and `ask` suppress `--chrome` automatically in print mode (BUG-304 fix); `isolated` injects `--chrome` unconditionally unless `--no-chrome` is set.
- `--output-format` is not a gap — it is passthrough-covered (`-- --output-format json`). If TSK-332 adds `--output-style`, the Path B auto-inject supplies `--output-format json` automatically.

---

### Exclusion Rationale

Params not in the gap closure table are excluded by design. Four categories:

| Category | Params | Reason |
|----------|--------|--------|
| **Temp HOME = meaningless** | `--topic`, `--session-dir`, `--new-session`, `--no-persist`, `-c` | Temp HOME has no session history; these params control session state that does not exist (`--session-dir` is additionally deprecated and inert everywhere — BUG-493 — so its exclusion needs no temp-HOME-specific rationale at all) |
| **One-shot = no retry** | 20 retry params, `--expect-retries`, `--fallback-model` | No retry loop in `run_isolated_command()`; retrying bad credentials is pointless |
| **Passthrough covers it** | `--output-format` | The only remaining native-flag gap after TSK-443; no CLR-level validation or transformation; override via `-- --output-format json` |
| **Architecture mismatch** | `--interactive`, `--verbose`, Ultrathink suffix | `--interactive` conflicts with message-present contract; `--verbose` is a claude-native passthrough (no CLR-internal gating); ultrathink conflicts with "execute immediately" CLAUDE.md directive |

---

### Planned Gap Closures

The following gaps between `isolated` and `run`/`ask` are tracked as implementation tasks. Each gap was qualified as **Actual Gap** (not by design) in the gap analysis.

| Task | Gap | Dimension | `isolated` After Closure |
|------|-----|-----------|--------------------------|
| TSK-328 ✅ | `--dry-run` not available | Execution Modes | Preview injected command without spawning subprocess |
| TSK-329 ✅ | `--dir` / `--add-dir` not available | Directory | Set working directory and grant read access to additional paths |
| TSK-330 ✅ | `--file` not available | Input | Pipe a file as stdin to the isolated subprocess |
| TSK-331 ✅ | `--expect` / `--expect-strategy` (fail + default) not available | Validation | Assert output matches expected pattern; exit 3 on mismatch |
| TSK-332 ✅ | `--output-file`, `--strip-fences`, `--output-style`, `--summary-fields` not available | Output | Tee output to file, strip code fences, render summary, select fields |
| TSK-443 ✅ | 12 params (`--model`, `--effort`, `--no-effort-max`, `--system-prompt`, `--append-system-prompt`, `--json-schema`, `--mcp-config`, `--allowed-tools`, `--disallowed-tools`, `--max-budget-usd`, `--max-turns`, `--no-chrome`) available only via `--` passthrough | Model & Effort, Auto-Injections, Input, System Prompt, Tools & Budget | All 12 now have native `isolated` flags with `CLR_*` env var fallbacks and `--args-file` JSON support |

---

### Cross-References

| Type | Path | Responsibility |
|------|------|----------------|
| command | `command/01_run.md` | `run` full reference |
| command | `command/05_ask.md` | `ask` reference (alias) |
| command | `command/03_isolated.md` | `isolated` full reference |
| doc | `002_command_defaults.md` | Injection defaults with Plan 009 design traceability |
| parity | `002_isolated_refresh.md` | `isolated` vs `refresh` credential command comparison |
| invariant | `../invariant/005_isolated_subprocess_defaults.md` | Isolated subprocess injection contracts |
