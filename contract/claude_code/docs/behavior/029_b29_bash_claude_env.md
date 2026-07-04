# Behavior B29: Bash Subprocesses Inherit Full Parent OS Environment

### Scope

- **Purpose**: Document that every process spawned by a Bash tool call inherits the complete OS environment of the parent claude process — including all `CLAUDE_*` vars, project vars, desktop session vars, API keys, and system vars. Parent-session and subagent Bash subprocess environments are byte-for-byte identical.
- **Responsibility**: Authoritative instance for behavior B29 — defines the behavior statement, certainty level, and supporting evidence. Tier is UNVERIFIED.
- **In Scope**: The full environment (107 vars observed) in `/proc/self/environ` inside Bash tool calls; the `CLAUDE_*` subset (operationally critical); identity of parent vs subagent environments; non-`CLAUDE_*` timeout vars; API key propagation.
- **Out of Scope**: Full env var reference (→ `../params/`); rtk process identity (→ [B28](028_b28_bash_rtk_subprocess.md)); agent subagent process model (→ [B27](027_b27_agent_no_os_process.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 99% | **Tier**: UNVERIFIED | **Since**: v2.1.74 | **Evidence**: E54, E56

Every `rtk`/bash subprocess spawned by a Bash tool call inherits the **complete OS environment** of the parent claude process. In a direct parent vs. subagent comparison (E56), 107 environment variables were observed — all identical, zero differences.

**The `CLAUDE_*` subset (operationally critical):**

| Variable | Observed Value | Purpose |
|----------|---------------|---------|
| `CLAUDECODE` | `1` | Marker: executing inside Claude Code session |
| `CLAUDE_CODE_ENTRYPOINT` | `cli` | Invocation method: CLI (not API, IDE, etc.) |
| `CLAUDE_CODE_MAX_OUTPUT_TOKENS` | `100000` | Max tokens in model output per turn |
| `CLAUDE_CODE_EFFORT_LEVEL` | `max` | Effort level setting for this session |
| `CLAUDE_TOOL_TIMEOUT` | `7200000` | Default tool call timeout (ms) — 2 hours |
| `CLAUDE_EXEC_TIMEOUT` | `7200000` | Execution timeout (ms) — 2 hours |
| `CLAUDE_BASH_TIMEOUT` | `7200000` | Bash-specific timeout (ms) — 2 hours |
| `CLAUDE_DEFAULT_TIMEOUT` | `7200000` | Default timeout fallback (ms) — 2 hours |
| `CLAUDE_COMMAND_TIMEOUT` | `7200000` | Command execution timeout (ms) — 2 hours |

**Other notable inherited vars (non-exhaustive):**

| Category | Examples |
|----------|---------|
| Project vars | `PRO`, `GENAI`, `GENAI_BIN`, `ENTRY`, `PBK`, `DPBK`, `HOSTING`, `WDISCOVERY` |
| Non-CLAUDE_* timeouts | `COMMAND_TIMEOUT=7200`, `EXEC_TIMEOUT=7200`, `TIMEOUT=7200`, `TOOL_TIMEOUT=7200`, `NODE_TIMEOUT=7200000` |
| API keys | `FIRECRAWL_API_KEY` (and any other key set in the parent session environment) |
| Node/NVM | `NVM_BIN`, `NVM_DIR`, `NVM_INC`, `NODE_OPTIONS=--max-old-space-size=8192`, `UV_THREADPOOL_SIZE=128` |
| Desktop/system | `DISPLAY`, `DBUS_SESSION_BUS_ADDRESS`, `XDG_*`, `GNOME_*`, `TERM`, `SHELL`, `HOME`, `USER` |
| Git | `GIT_EDITOR=true`, `GIT_MERGE_AUTOEDIT=no` |
| RTK | `_=/home/user1/.cargo/bin/rtk` (last command = rtk binary) |

`CLAUDECODE=1` is the canonical signal that code is executing within a Claude Code tool call environment.

**Parent = subagent:** Because subagents are API inference threads within the same Level-1 claude OS process (B27), both parent-session and subagent Bash calls are spawned by that same process and inherit its full environment unchanged. No filtering occurs between the two levels.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E54 | B29 | Experiment | `/proc/self/environ` inspection — this session (2026-06-28) | Agent A Bash tool call | `cat /proc/self/environ \| tr '\0' '\n' \| grep -i claude` returned 9 CLAUDE_* vars: CLAUDE_CODE_MAX_OUTPUT_TOKENS=100000, CLAUDE_TOOL_TIMEOUT=7200000, CLAUDECODE=1, CLAUDE_EXEC_TIMEOUT=7200000, CLAUDE_BASH_TIMEOUT=7200000, CLAUDE_DEFAULT_TIMEOUT=7200000, CLAUDE_CODE_EFFORT_LEVEL=max, CLAUDE_COMMAND_TIMEOUT=7200000, CLAUDE_CODE_ENTRYPOINT=cli |
| E56 | B29 | Experiment | Full env comparison — parent vs subagent (2026-06-29) | `cat /proc/self/environ \| tr '\0' '\n' \| sort` in parent Bash call and general-purpose subagent Bash call | 107 variables enumerated; zero differences between parent and subagent. Confirmed: entire OS environment inherited unchanged, including project vars ($PRO, $GENAI, etc.), API keys (FIRECRAWL_API_KEY), non-CLAUDE_* timeouts (COMMAND_TIMEOUT=7200, TOOL_TIMEOUT=7200), NVM, desktop session (XDG_*, GNOME_*, DISPLAY), and git config. |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [027_b27_agent_no_os_process.md](027_b27_agent_no_os_process.md) | Agent subagents not OS processes — context for why env propagates from one parent |
| behavior | [028_b28_bash_rtk_subprocess.md](028_b28_bash_rtk_subprocess.md) | rtk subprocess identity — the process that holds these env vars |
| param | [../params/013_bash_timeout.md](../params/013_bash_timeout.md) | CLAUDE_CODE_BASH_TIMEOUT — runner-level default timeout |
