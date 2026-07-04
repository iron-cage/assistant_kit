# Parity: isolated / refresh

### Scope

- **Purpose**: Behavioral parity comparison of `clr isolated` and `clr refresh` — the two credential-operation commands.
- **Responsibility**: Document where these commands share infrastructure and where they diverge in purpose, injections, timeout, and exit semantics.
- **In Scope**: Param surface, auto-injections, credential lifecycle, timeout rationale, exit code semantics.
- **Out of Scope**: Comparison with task-execution commands (-> `001_run_ask_isolated.md`); implementation internals (-> `docs/feature/001_runner_tool.md`).

---

### Comparison Matrix

| Dimension | `isolated` | `refresh` |
|-----------|-----------|-----------|
| **--- Identity ---** | | |
| Purpose | Execute a user task with credential isolation | Refresh OAuth credentials; no user task executed |
| User task executed? | Yes — user-supplied `MESSAGE` | No — hardcoded `"."` prompt only |
| Primary output | Task result (stdout from claude) | Updated credentials written back to `--creds` |
| Syntax | `clr isolated [--creds F] [--timeout N] [MESSAGE]` | `clr refresh [--creds F] [--timeout N]` |
| Shares implementation | `run_isolated()` (direct call) | `run_isolated()` (with fixed args) |
| **--- Params ---** | | |
| `MESSAGE` positional | Yes (optional) | No (hardcoded `"."` internally) |
| `--creds` | Yes (default: `~/.claude/.credentials.json`) | Yes (same default) |
| `--timeout` | Yes (default: `30 s`) | Yes (default: `45 s`) |
| `--trace` | Yes | Yes |
| `--journal` / `--journal-dir` | Yes | Yes |
| `-h` / `--help` | Yes | Yes |
| Passthrough (`-- <args>`) | Yes | No |
| **--- Auto-Injections ---** | | |
| Default model | `opus` (`ISOLATED_DEFAULT_MODEL`) | `claude-sonnet-5` (`REFRESH_DEFAULT_MODEL`) |
| Default effort | `max` | `low` |
| `--dangerously-skip-permissions` | Yes — when MESSAGE present; No in no-message REPL | No — never (refresh invokes no tools) |
| `--no-session-persistence` | Yes — always | Yes — always |
| `--chrome` | On (ClaudeCommand default) | Off (`--no-chrome` injected) |
| `env -u CLAUDECODE` | Yes | Yes |
| Minimal `CLAUDE.md` written to temp HOME | Yes (suppress interactive prompts) | Yes (same content) |
| `CLAUDE_CODE_MAX_OUTPUT_TOKENS` | `200,000` | `200,000` |
| **--- Session ---** | | |
| HOME directory | Fresh temp dir | Fresh temp dir |
| Temp HOME cleanup | Yes (unconditional delete) | Yes (unconditional delete) |
| Session persistence | Always off | Always off |
| **--- Timeout ---** | | |
| Default timeout | `30 s` | `45 s` (extra headroom for OAuth exchange + network latency) |
| `--timeout 0` = unlimited | Yes | Yes |
| Exit on timeout (no refresh detected) | `2` | `2` |
| Exit on timeout (refresh completed before deadline) | `0` (not `2`) | Not applicable — refresh exits before timeout if token exchange completes |
| **--- Credential Handling ---** | | |
| Credential writeback | Yes — if OAuth refresh detected during run | Yes — primary purpose; always written if token exchanged |
| Exit `0` meaning | Task success OR creds refreshed | Creds refreshed and written back (only successful outcome) |
| Exit `1` meaning | Error: creds not found / spawn failure / I/O | Error: creds not found / spawn failure / I/O / **no refresh occurred** |
| Exit `2` meaning | Timeout — no refresh detected | Timeout — no refresh occurred |
| Exit `N` (subprocess passthrough) | Yes | No (no passthrough exits documented) |
| **--- Execution Mode ---** | | |
| No-message -> REPL | Yes (interactive; no `--dangerously-skip-permissions`) | No (no MESSAGE param; always print with `"."`) |
| With-message -> print | Yes (+ `--dangerously-skip-permissions`) | Yes (always; fixed `"."`) |
| Passthrough override | Yes via `-- <args>` | No |
| **--- Rationale for Divergence ---** | | |
| Model rationale | Maximum capability for real user tasks | Sonnet sufficient for trivial OAuth-trigger `"."` prompt |
| Effort rationale | `max` — real task may require deep reasoning | `low` — minimal reasoning for a one-character ping |
| `--chrome` rationale | Browser tools may be needed for real tasks | OAuth exchange is pure HTTP; browser context adds overhead |
| `--dangerously-skip-permissions` rationale | Tool calls must not block interactively in print mode | No tool use; no permission prompts possible |
| Timeout rationale | 30s adequate for typical Claude startup | 45s extra headroom for slow networks + OAuth rate limits |
| **--- Journal ---** | | |
| `--journal` / `--journal-dir` | Yes | Yes |

---

### Key Takeaways

- Both commands share the same underlying `run_isolated()` implementation and the same temp HOME lifecycle (create, populate, spawn, delete).
- The defining difference: `isolated` runs a real user task (maximized capability: Opus + max effort + chrome); `refresh` runs a trivial ping (minimized footprint: Sonnet + low effort + no-chrome).
- `refresh` exit `1` covers "no refresh occurred" — if the token was already fresh, there is nothing to write back, so exit `1` signals that nothing changed. This differs from `isolated` where exit `1` means only spawn/I/O errors.
- `isolated` can fall back to exit `0` on timeout if the OAuth refresh completed before the deadline; `refresh` always exits `2` on timeout (the token exchange is all-or-nothing).
- `isolated` supports passthrough args (`-- --effort medium`); `refresh` does not — its injections are fixed by design.

---

### Cross-References

| Type | Path | Responsibility |
|------|------|----------------|
| command | `command/03_isolated.md` | `isolated` full reference |
| command | `command/04_refresh.md` | `refresh` full reference |
| parity | `001_run_ask_isolated.md` | `run` / `ask` / `isolated` comprehensive comparison |
| doc | `command_defaults.md` | Default injection matrix (all 4 commands) with Plan 009 traceability |
| invariant | `../invariant/005_isolated_subprocess_defaults.md` | Isolated/refresh subprocess injection contracts |
| param | `../param/019_creds.md` | `--creds` file path parameter |
| param | `../param/020_timeout.md` | `--timeout` for isolated/refresh (exit 2) |
