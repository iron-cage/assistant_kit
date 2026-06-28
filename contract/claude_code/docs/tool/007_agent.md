# Tool: Agent

Launch specialized subagent processes.

### Category

Agents

### Description

Launches autonomous subagents to handle complex, multi-step tasks. Subagents run as API inference threads within the existing claude process — they do NOT spawn new OS-level `claude` processes (see behavior B27). Multiple agents can run in parallel. Supports background execution, worktree isolation, model override, and resumption by agent ID.

### Since

pre-v1.0 (unverified)

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `prompt` | string | ✅ | — | Full task description. Provide complete context — agents start fresh unless `resume` is set. |
| `description` | string | ✅ | — | 3-5 word summary shown in UI. |
| `subagent_type` | string | ❌ | `general-purpose` | Selects the agent variant. See Built-in Subagent Types below. Invalid values silently fall back to `general-purpose`. |
| `model` | enum | ❌ | agent default | Model override: `sonnet`, `opus`, `haiku`. Takes precedence over agent definition's model frontmatter. |
| `run_in_background` | boolean | ❌ | `false` | `false` = block until done; `true` = fire-and-forget, completion notification delivered automatically. |
| `isolation` | enum | ❌ | — | `"worktree"` = run in a temporary git worktree sandbox. Worktree cleaned up automatically if no changes; if changes are made, worktree path and branch are returned. |
| `resume` | string | ❌ | — | Agent ID from a prior invocation. Resumes with full previous context preserved. |

### Returns

| Field | Condition | Value |
|-------|-----------|-------|
| Result message | Always | Agent's final output (single message; not shown to user unless relayed) |
| Agent ID | Always | Opaque string; use with `resume` parameter in future calls |
| Worktree path | Only if `isolation: "worktree"` AND changes made | Filesystem path of the temp worktree |
| Branch | Only if `isolation: "worktree"` AND changes made | Git branch name for the worktree |

### Built-in Subagent Types

The `subagent_type` parameter selects a pre-configured agent variant. The list is static and embedded in the Agent tool's schema description — there is no runtime enumeration API.

| `subagent_type` | Tools Available | Can Write Files | Can Spawn Agents | Primary Use Case |
|-----------------|----------------|-----------------|------------------|-----------------|
| `general-purpose` (default) | All (`*`) | ✅ | ✅ | Complex multi-step tasks, research, code execution |
| `Explore` | All except Agent, ExitPlanMode, Edit, Write, NotebookEdit | ❌ | ❌ | Codebase search, file/pattern discovery, read-only analysis |
| `Plan` | All except Agent, ExitPlanMode, Edit, Write, NotebookEdit | ❌ | ❌ | Architecture design, implementation planning, trade-off analysis |
| `claude-code-guide` | Glob, Grep, Read, WebFetch, WebSearch | ❌ | ❌ | Questions about Claude Code CLI, Agent SDK, Anthropic API |
| `statusline-setup` | Read, Edit | ✅ (status line config only) | ❌ | Configure Claude Code status line display |

**Enumeration**: There is no `AgentList` tool. Running agents cannot be enumerated via tools. Track agent IDs at dispatch time; use `resume` for re-entry.

**Proactive resume**: `claude-code-guide` recommends checking for a running instance before spawning a new one.

**Thoroughness levels** (for `Explore` and `Plan`): Specify in `prompt` as `"quick"`, `"medium"`, or `"very thorough"`.

### Process Model

Agent subagents are NOT new OS-level `claude` processes. They run as API inference threads within the parent claude process. `pgrep -a claude` is unchanged before, during, and after agent dispatch. Each Bash tool call within an agent spawns a short-lived `rtk` proxy process (~5 MB, ephemeral) under the same parent claude process (see B27, B28).

Contrast: Skill tool invocations (`/doc_tsk`, `/role`, etc.) DO spawn new `claude --print --output-format json` OS processes.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [../params/003_agent.md](../params/003_agent.md) | Agent override parameter |
| doc | [../params/004_agents.md](../params/004_agents.md) | Custom agent definitions |
| doc | [036_send_message.md](036_send_message.md) | Send message to agent teammate |
| doc | [040_workflow.md](040_workflow.md) | Multi-subagent workflow orchestration |
| doc | [../subcommand/001_agents.md](../subcommand/001_agents.md) | Agents subcommand — lists configured agents |
| behavior | [../behavior/027_b27_agent_no_os_process.md](../behavior/027_b27_agent_no_os_process.md) | B27: agent subagents are not OS processes |
| behavior | [../behavior/028_b28_bash_rtk_subprocess.md](../behavior/028_b28_bash_rtk_subprocess.md) | B28: bash tool calls spawn transient rtk processes |
| behavior | [../behavior/029_b29_bash_claude_env.md](../behavior/029_b29_bash_claude_env.md) | B29: CLAUDE_* env vars propagated to all bash subprocesses including agent Bash calls |
| behavior | [../behavior/030_b30_subagent_context_inheritance.md](../behavior/030_b30_subagent_context_inheritance.md) | B30: subagents receive full CLAUDE.md via system-reminder; parent conversation not inherited; scope not propagated |
