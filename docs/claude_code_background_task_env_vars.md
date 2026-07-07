# Claude Code Background-Task Environment Variables

## Scope

Environment variables governing Claude Code CLI (`claude`) background task execution, exit-wait/handoff behavior, and session/process identity classification. Scoped to the subset relevant to background-task termination safety during `-p` (print/non-interactive) runs — not a full catalog of all Claude Code environment variables (the official docs page lists ~150).

## Source

- Official docs: `https://code.claude.com/docs/en/env-vars` (scraped snapshot, Claude Code v2.1.197 era)
- Cross-checked against independent GitHub mirror: `oikon48/cc-doc-tracker`
- Binary verification: `strings` analysis of installed Claude Code `~/.local/share/claude/versions/2.1.197`
- Confidence markers: 📖 = officially documented; 🔍 = undocumented — confirmed only via binary analysis, no version-gating guarantee, may change without notice

## Session / Process Identity

| Variable | Status | Purpose | Notes |
|---|---|---|---|
| `CLAUDECODE` | 📖 Documented | Set to `1` in **any** subprocess Claude Code spawns (Bash/PowerShell tools, tmux, hooks, statusline, stdio MCP servers) — also set by IDE extensions in integrated terminals | Broadest/oldest marker. Doesn't distinguish "spawned by a tool call" from "IDE-integrated terminal." |
| `CLAUDE_CODE_CHILD_SESSION` | 📖 Documented (v2.1.172+) | Set to `1` specifically in subprocesses from Bash/PowerShell/Monitor tools, hooks, statusline — **not** set for stdio MCP servers (long-lived, outlive the session) or by IDE extensions | The precise marker for "a nested `claude` process Claude Code itself launched." A nested interactive TUI started this way is auto-excluded from `--resume`/`--continue`/history/`claude agents` list; non-interactive `claude -p` still persists. |
| `CLAUDE_CODE_FORCE_SESSION_PERSISTENCE` | 📖 Documented | Set to `1` to override the exclusion above when it's a false positive (e.g. inherited via `screen` or a background launcher) | The false-positive case: the Bash tool starts a `screen` session; that subprocess inherits `CLAUDE_CODE_CHILD_SESSION=1`; a human later attaches to that `screen` and starts a genuinely-intentional interactive `claude` session inside it. Since the env var is still inherited across the attach, the new session would incorrectly be excluded from resume/history/agents unless this override forces normal registration. v2.1.178+ auto-detects and fixes this specifically for tmux, so tmux no longer needs the override — `screen` and other wrappers still might. |
| `CLAUDE_CODE_ENTRYPOINT` | 🔍 **Undocumented** — absent from both the official doc scrape and the independent mirror | Enum classifier read via `process.env.CLAUDE_CODE_ENTRYPOINT`; `switch`-mapped values seen in the binary: `"claude-vscode"`, `"remote"`, `"remote_baku"`, `"remote_cowork"`, `"remote_desktop"`, `"remote_mobile"`, `"claude-in-teams"`, `"sdk-cli"`, `"sdk-ts"` → categories like `"claude_code_vscode"`, `"claude_code_remote"` | The documented `OTEL_METRICS_INCLUDE_ENTRYPOINT` setting ("include the session entrypoint in metrics attributes") confirms "entrypoint" is a real internal concept — just not one exposed as a settable input. Likely set by the launching wrapper, not meant to be hand-set. |

## Background-Task Exit-Wait & Handoff

| Variable | Status | Purpose | Notes |
|---|---|---|---|
| `CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS` | 📖 Documented, causally tested against a live process | Max ms `-p` mode waits after the final turn for backgrounded subagents/workflows whose result feeds the output. Default `600000` (10 min). `0` = wait indefinitely | Confirmed by direct test: changing the value altered wait behavior for Agent/Workflow dispatches; had zero effect on a plain background shell's cutoff. |
| *(plain background Bash grace — no variable)* | 📖 Documented narratively, not overridable | A `claude -p` background Bash task is killed ~5s after the final result + stdin close (before v2.1.163: a never-exiting one held the process open forever) | No env var found for this specific grace period — matches the empirical finding that zeroing the ceiling var above didn't touch it. |
| `CLAUDE_CODE_DISABLE_BACKGROUND_TASKS` | 📖 Documented | Set to `1` to disable **all** background task functionality: `run_in_background` on Bash/subagent tools, auto-backgrounding, Ctrl+B | Source-confirmed tied directly to the Agent tool's dispatch/stop logic — flips an `awaitCompletion` flag at `"agent-stopped"`. |
| `CLAUDE_CODE_DISABLE_BG_EXIT_HANDOFF` | 📖 Documented (v2.1.196+) | For background sessions (agent-view/supervisor architecture) specifically: stops running background shells/workflows/(v2.1.198+ subagents) from being handed to the session's next process when the supervisor stops/restarts/updates it | Does NOT affect backgrounding via `←`/`/background` (that handoff still happens) — `CLAUDE_DISABLE_ADOPT` turns off both. Source-confirmed gated on `CLAUDE_JOB_DIR`. |
| `CLAUDE_JOB_DIR` | 🔍 **Undocumented** | Source-confirmed as a precondition inside the handoff function: `if(!yi()||!Fe.CLAUDE_JOB_DIR||Fe.CLAUDE_CODE_DISABLE_BG_EXIT_HANDOFF)return{shells:[],workflows:[]}` | Reads as an internal marker present when running under the supervisor/agent-view architecture — not something to set by hand. |
| `CLAUDE_DISABLE_ADOPT` | 📖 Documented (v2.1.195+) | Different lifecycle point: when the user backgrounds a session (`←`/`/background`), in-flight work normally carries over — this stops it instead, with a confirmation prompt | |
| `CLAUDE_CODE_DISABLE_BG_SHELL_PRESSURE_REAP` | 📖 Documented (v2.1.193+) | Unrelated to exit/handoff — stops Claude Code killing background shells on OS memory-pressure signals (macOS/Linux only; normally after 30 min idle + no turn/subagent running) | Windows has no memory-pressure signal, so it's a no-op there. |
| `CLAUDE_ASYNC_AGENT_STALL_TIMEOUT_MS` | 📖 Documented | Stall/no-progress timeout for background subagents, default `600000`ms, resets on each streaming progress event | Distinct axis from the exit-wait ceiling — this can fire mid-session on a stalled agent, not just at process exit. |

## `CLAUDE_CODE_BG_*` and Auto-Backgrounding Classification

| Variable | Status | Purpose | Notes |
|---|---|---|---|
| `CLAUDE_AUTO_BACKGROUND_TASKS` | 📖 Documented — name has no `_CODE_` | Set to `1` to force-enable automatic backgrounding of long-running agent tasks; subagents auto-move to background after ~2 minutes | A search for `CLAUDE_CODE_AUTO_BACKGROUND_TASKS` (with `_CODE_`) correctly finds zero matches in the binary — that name does not exist. |
| `CLAUDE_CODE_BG_CLASSIFIER_MODEL` | 🔍 **Undocumented, low confidence** | Only seen in an export/minification name-map, positioned among model-selection vars (`CLAUDE_CODE_AUTO_MODE_MODEL`, `ANTHROPIC_SMALL_FAST_MODEL`, `ANTHROPIC_MODEL`) | No usage snippet found — only the name mapping. Plausible purpose: which model classifies/decides auto-backgrounding-related behavior. Not directly confirmed. |
| `CLAUDE_CODE_BG_TASKS_REPORT_RUNNING` | 🔍 **Undocumented, medium confidence** | Real usage confirmed: `if(n&&Fe.CLAUDE_CODE_BG_TASKS_REPORT_RUNNING)return!1` where `n = hasRunningBgTasks` | Found adjacent to the ceiling-decision functions. Reads as a flag that, when set, suppresses an idle/"running" status indicator based on whether background tasks exist. |

## Summary

The mechanism that governs "don't let `-p` kill subagents/workflows mid-flight" is `CLAUDE_CODE_PRINT_BG_WAIT_CEILING_MS=0` — confirmed and causally tested. `CLAUDE_CODE_CHILD_SESSION`, `CLAUDE_CODE_ENTRYPOINT`, and the `BG_*` variables are adjacent machinery (session classification, exit-handoff for the separate supervisor/agent-view architecture, auto-backgrounding heuristics) rather than alternate levers for the same problem — none of them override the 5-second plain-background-shell grace period, which has no documented override.
