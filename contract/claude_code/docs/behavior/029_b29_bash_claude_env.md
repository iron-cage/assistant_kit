# Behavior B29: Bash Subprocesses Inherit CLAUDE_* Environment

### Scope

- **Purpose**: Document that every process spawned by a Bash tool call inherits a fixed set of `CLAUDE_*` environment variables from the parent claude process.
- **Responsibility**: Authoritative instance for behavior B29 — defines the behavior statement, certainty level, and supporting evidence. Tier is UNVERIFIED.
- **In Scope**: The 9 `CLAUDE_*` env vars observed in `/proc/self/environ` inside Bash tool calls; their values at time of observation; propagation to both parent-session and subagent Bash calls.
- **Out of Scope**: Full env var reference (→ `../params/`); rtk process identity (→ [B28](028_b28_bash_rtk_subprocess.md)); agent subagent process model (→ [B27](027_b27_agent_no_os_process.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 99% | **Tier**: UNVERIFIED | **Since**: v2.1.74 | **Evidence**: E54

Every `rtk`/bash subprocess spawned by a Bash tool call inherits the following `CLAUDE_*` environment variables from the parent claude process. Observed values as of v2.1.74:

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

These variables are present in both parent-session Bash calls and Agent subagent Bash calls — propagation is not restricted to the top-level session.

`CLAUDECODE=1` is the canonical signal that code is executing within a Claude Code tool call environment. Shell scripts and programs can use this to detect Claude Code context.

The timeout values (`CLAUDE_*_TIMEOUT`) reflect the session configuration set via `--settings` or environment override. The values observed (7200000 ms = 2 hours) come from the custom timeout config loaded by this session's `claude --settings /home/user1/.claude/cld-timeout-config.json`.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E54 | B29 | Experiment | `/proc/self/environ` inspection — this session (2026-06-28) | Agent A Bash tool call | `cat /proc/self/environ | tr '\0' '\n' | grep -i claude` returned 9 CLAUDE_* vars: CLAUDE_CODE_MAX_OUTPUT_TOKENS=100000, CLAUDE_TOOL_TIMEOUT=7200000, CLAUDECODE=1, CLAUDE_EXEC_TIMEOUT=7200000, CLAUDE_BASH_TIMEOUT=7200000, CLAUDE_DEFAULT_TIMEOUT=7200000, CLAUDE_CODE_EFFORT_LEVEL=max, CLAUDE_COMMAND_TIMEOUT=7200000, CLAUDE_CODE_ENTRYPOINT=cli |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [027_b27_agent_no_os_process.md](027_b27_agent_no_os_process.md) | Agent subagents not OS processes — context for why env propagates from one parent |
| behavior | [028_b28_bash_rtk_subprocess.md](028_b28_bash_rtk_subprocess.md) | rtk subprocess identity — the process that holds these env vars |
| param | [../params/013_bash_timeout.md](../params/013_bash_timeout.md) | CLAUDE_CODE_BASH_TIMEOUT — runner-level default timeout |
