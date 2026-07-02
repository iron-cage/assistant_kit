# Behavior B28: Bash Tool Calls Spawn Transient rtk Processes

### Scope

- **Purpose**: Document that each Bash tool call spawns a short-lived `rtk` wrapper process that exits immediately after the command completes — not a persistent bash shell.
- **Responsibility**: Authoritative instance for behavior B28 — defines the behavior statement, certainty level, and supporting evidence. Tier is UNVERIFIED.
- **In Scope**: `/proc/self/status` identity inside Bash tool; process name `rtk`; memory footprint (~5 MB); file descriptor count (4); ephemeral lifetime; PID uniqueness per call; `$$` and `$PPID` unreliability inside tool calls.
- **Out of Scope**: Agent subagents not being OS processes (→ [B27](027_b27_agent_no_os_process.md)); CLAUDE_* env propagation (→ [B29](029_b29_bash_claude_env.md)); rtk token-saving filter behavior (out of scope for claude_code contract).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 95% | **Tier**: UNVERIFIED | **Since**: v2.1.74 | **Evidence**: E53

Each Bash tool call spawns a distinct, short-lived `rtk` proxy process. The process is not a persistent shell — it exits immediately after the command completes. Consequently:

- Each Bash call receives a unique PID from the kernel (observed range: sequential in the low-millions).
- `/proc/self/status` inside any Bash call shows `Name: rtk`, not `bash` or `claude`.
- Memory footprint is approximately 5 MB VmRSS — consistent with a lightweight proxy binary, not a Node.js or Electron runtime.
- File descriptor count is 4 (stdin, stdout, stderr, the fd directory itself) — minimal, no persistent pipes to a parent agent loop.
- The parent process (`$PPID`) disappears before the next Bash call executes. `/proc/$PPID/` is gone by the time a subsequent command runs.
- `$$` and `$PPID` shell variables are unreliable inside tool calls. rtk intercepts command strings before shell expansion in some contexts, causing `$$` to expand to an empty string. Use `cat /proc/self/status | grep ^Pid` to obtain the current process PID reliably.
- `cat /proc/self/cmdline` is rewritten by rtk to `rtk read /proc/self/cmdline`, reflecting the proxy interception layer.

The `cwd` of spawned processes is the session working directory (`/home/user1/pro` in the observed session). The cgroup matches the parent terminal session scope — no isolation boundary.

**Platform note**: Observed on Linux 6.8.0-124-generic with rtk installed. On systems without rtk, the process name would differ (likely `bash`). The transient-lifetime and per-call-PID properties hold regardless of the wrapper binary name.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E53 | B28 | Experiment | `/proc/self/status` inspection — this session (2026-06-28) | Agent A and B Bash tool calls | `Name: rtk`, `Pid: 3349457`, `VmRSS: 4884 kB`, `Threads: 1`; `ls /proc/self/fd | wc -l` = 4; parent PID gone before next command; `cat /proc/self/cmdline` returned `rtk read /proc/self/cmdline`; `$$` empty in some invocations |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [027_b27_agent_no_os_process.md](027_b27_agent_no_os_process.md) | Agent subagents not OS processes (Level 2 of process model) |
| behavior | [029_b29_bash_claude_env.md](029_b29_bash_claude_env.md) | CLAUDE_* env vars propagated to these rtk processes |
| tool | [../tool/004_bash.md](../tool/004_bash.md) | Bash tool — command, timeout, run_in_background parameters |
| filesystem | [../filesystem/004_proc_system.md](../filesystem/004_proc_system.md) | `/proc/` filesystem paths — process scanner context (distinct from subprocess self-inspection) |
