# Behavior B27: Agent Tool Subagents Are Not OS Processes

### Scope

- **Purpose**: Document that Agent tool subagents do not spawn new `claude` OS processes — they run as API inference threads within the existing parent claude process.
- **Responsibility**: Authoritative instance for behavior B27 — defines the behavior statement, certainty level, and supporting evidence. Tier is UNVERIFIED (no automated invalidation test yet).
- **In Scope**: Agent tool `run_in_background` dispatch; `pgrep -a claude` process count before/during/after; the three-level process model (persistent claude OS process → API inference thread → transient bash subprocess).
- **Out of Scope**: Bash tool subprocess identity (→ [B28](028_b28_bash_rtk_subprocess.md)); CLAUDE_* env propagation (→ [B29](029_b29_bash_claude_env.md)); agent session storage layout (→ [B13](013_b13_subagent_directory.md)); skill invocations, which DO spawn new OS processes via `claude --print --output-format json`.

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 99% | **Tier**: UNVERIFIED | **Since**: v2.1.74 | **Evidence**: E52

Agent tool subagents do not create new `claude` binary OS processes. `pgrep -a claude` returns identical results before, during, and after dispatching Agent subagents — even when agents are actively executing Bash tool calls.

Three-level process model:

1. **Persistent claude OS process** — the long-running `claude` binary instance (appears in `pgrep`). Each interactive session and each skill invocation (`claude --print --output-format json`) is a separate OS process at this level.
2. **API inference thread** — the Agent subagent. Runs as an API conversation within the parent Level 1 process. Has no OS PID. Invisible to `pgrep`, `ps`, or any OS-level process enumeration tool.
3. **Transient bash subprocess** — short-lived `rtk` wrapper processes spawned by the Level 1 claude process when executing Bash tool calls. Both parent and subagent Bash calls are spawned by the same Level 1 parent.

Consequence: there is no OS-level mechanism to enumerate or monitor running Agent subagents. The only re-entry point is the `resume` parameter with a known agent ID returned at dispatch time.

Contrast with skill invocations: skills invoked via the Skill tool spawn new `claude --print --output-format json` OS processes, which ARE visible in `pgrep`. Agent tool subagents and Skill tool invocations use different execution models.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E52 | B27 | Experiment | Live `pgrep` snapshot — this session (2026-06-28) | Parent session, pre/during/post agent dispatch | `pgrep -a claude` returned 13 processes before launching 2 background agents; 13 during active execution (agents running Bash tool calls); 13 after completion. Net delta: 0. Agent Bash call PIDs (3348183, 3356028, 3373973) absent from `pgrep -a claude` output. |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [028_b28_bash_rtk_subprocess.md](028_b28_bash_rtk_subprocess.md) | Bash tool calls spawn transient rtk processes (Level 3 of process model) |
| behavior | [029_b29_bash_claude_env.md](029_b29_bash_claude_env.md) | CLAUDE_* env vars propagated to all bash subprocesses |
| behavior | [013_b13_subagent_directory.md](013_b13_subagent_directory.md) | Agent session storage layout (separate concern from process model) |
| tool | [../tool/007_agent.md](../tool/007_agent.md) | Agent tool — parameter schema, subagent types, returns |
| tool | [../tool/013_skill.md](../tool/013_skill.md) | Skill tool — invocations observed as separate `claude --print --output-format json` OS processes (contrast with Agent subagents) |
