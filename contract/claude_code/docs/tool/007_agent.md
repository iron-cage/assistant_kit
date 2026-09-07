# Tool: Agent

Launch specialized subagent processes.

### Category

Agents

### Description

Launches autonomous subagents to handle complex, multi-step tasks. Subagents run as API inference threads within the existing claude process — they do NOT spawn new OS-level `claude` processes (see behavior B27). Multiple agents can run in parallel and run in the background by default. Supports worktree/remote isolation and model override; a spawned agent is messaged or resumed later via the SendMessage tool, not a parameter of this one (see [036_send_message.md](036_send_message.md)).

### Since

pre-v1.0 (unverified)

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `prompt` | string | ✅ | — | Full task description. Provide complete context — agents start fresh unless `subagent_type: "fork"` is used (see Built-in Subagent Types below). |
| `description` | string | ✅ | — | 3-5 word summary shown in UI. |
| `subagent_type` | string | ❌ | `general-purpose` | Selects the agent variant. See Built-in Subagent Types below. Invalid values silently fall back to `general-purpose`. |
| `model` | enum | ❌ | agent default | Model override: `sonnet`, `opus`, `haiku`, `fable`. Takes precedence over agent definition's model frontmatter. Ignored for `subagent_type: "fork"` — forks always inherit the parent model. |
| `run_in_background` | boolean | ❌ | background (non-blocking) | Agents run in the background by default — a completion notification is delivered automatically. Set `false` to block synchronously until the result is available. |
| `isolation` | enum | ❌ | — | `"worktree"` = run in a temporary git worktree sandbox; cleaned up automatically if no changes, otherwise worktree path and branch are returned. `"remote"` = launch in a remote cloud environment; always runs in background; availability is gated. |
| `name` | string | ❌ | — | Name for the spawned agent (letters/digits/underscores/hyphens, max 64 chars). Makes it addressable via `SendMessage({to: name})` while running. The reserved value `"main"` is rejected. |
| `cwd` | string | ❌ | — | Absolute path to run the agent in; overrides the working directory for all filesystem/shell operations inside this agent. Mutually exclusive with `isolation: "worktree"`. |
| `team_name` | string | ❌ | — | Deprecated; ignored. The session has a single implicit team. |
| `mode` | string | ❌ | — | Deprecated; ignored. Subagents inherit the parent session's permission mode; agent-definition frontmatter may override it. |

**`resume` — ❌ refuted, does not exist.** An earlier revision of this doc documented an active `resume` parameter ("Agent ID from a prior invocation. Resumes with full previous context preserved."). Confirmed absent from v2.1.220: the Agent tool's zod parameter schema (`prompt`/`description`/`subagent_type`/`model`/`run_in_background`, merged with `name`/`team_name`/`mode`, extended with `isolation`/`cwd`) and its `call()` handler's own argument destructuring both contain zero `resume` field. Consistent with [036_send_message.md](036_send_message.md)'s own dating: removed per v2.1.77, replaced by `SendMessage({to: agentId})` — see that doc.

**Schema variants:** at least one internal filtered variant always omits `cwd` (and, in some restricted contexts, `run_in_background` too) from the schema actually exposed to the model — so `cwd` does not necessarily appear in every session's live Agent tool definition. Not present in this doc's own authoring session's live schema at time of writing.

### Returns

| Field | Condition | Value |
|-------|-----------|-------|
| Result message | Always | Agent's final output (single message; not shown to user unless relayed) |
| Agent ID | Always | Opaque string; use with SendMessage's `to` parameter to message or resume the agent — see [036_send_message.md](036_send_message.md) |
| Worktree path | Only if `isolation: "worktree"` AND changes made | Filesystem path of the temp worktree |
| Branch | Only if `isolation: "worktree"` AND changes made | Git branch name for the worktree |

### Built-in Subagent Types

The `subagent_type` parameter itself is a freeform `string` in the schema (no enum) — the tool relies on the runtime, not the schema, to reject unrecognized values.

❌ **"static and embedded in schema description" — refuted.** An earlier revision of this doc claimed the list of available types was static and embedded directly in the Agent tool's own schema description. Confirmed wrong: v2.1.220's Agent tool top-level description instead reads "Available agent types are listed in `<system-reminder>` messages in the conversation" (4 hits) — the list is injected dynamically, per session, via a system-reminder message (exactly as demonstrated by this very collection's own discovery of tools 41-43, and by the `claude`/`fork` types found this way above). There is still no callable tool (`AgentList` or similar) the model can invoke on demand — the list only ever arrives via that pushed system-reminder.

| `subagent_type` | Observed Tool Set (B31) | Can Write Files | Can Spawn Agents | Primary Use Case |
|-----------------|------------------------|-----------------|------------------|-----------------|
| `general-purpose` (default) | ToolSearch (pre) + Bash, Edit, Write, NotebookEdit, EnterWorktree, ExitWorktree, Glob, Grep, Read, Skill, WebFetch, WebSearch (12 deferred) — **13 total** | ✅ | ❌ (Agent absent) | Complex multi-step tasks, research, code execution |
| `Explore` | ToolSearch (pre) + Bash, EnterWorktree, ExitWorktree, Glob, Grep, Read, Skill, WebFetch, WebSearch (9 deferred) — **10 total** | ❌ | ❌ | Codebase search, file/pattern discovery, read-only analysis |
| `Plan` | ToolSearch (pre) + Bash, EnterWorktree, ExitWorktree, Glob, Grep, Read, Skill, WebFetch, WebSearch (9 deferred) — **10 total** (identical to Explore) | ❌ | ❌ | Architecture design, implementation planning, trade-off analysis |
| `claude-code-guide` | Glob, Grep, Read, WebFetch, WebSearch (5 pre-loaded; no ToolSearch; no deferred) — **5 total** | ❌ | ❌ | Questions about Claude Code CLI, Agent SDK, Anthropic API |
| `statusline-setup` | Read, Edit (unverified; not experimentally confirmed) | ✅ (status line config only) | ❌ | Configure Claude Code status line display |
| `claude` | `*` — built-in definition declares `tools:["*"]` (unverified whether this literally includes Agent) | ✅ (unverified) | ❓ (unverified) | Catch-all for any task that doesn't fit a more specific agent; FleetView's default when no agent name is typed |
| `fork` (gated) | Unverified — not a fresh session like every other type here: inherits the **full parent conversation context** (contrast B30) | — (unverified) | — (unverified) | Continuing the current conversation as a background subagent. Requires the `tengu_fork_subagent_enabled` experiment flag — not confirmed generally available |

**Parent-session-exclusive tools (14 — absent from all subagent types):** Agent, AskUserQuestion, CronCreate, CronDelete, CronList, EnterPlanMode, ExitPlanMode, LSP, TaskCreate, TaskGet, TaskList, TaskOutput, TaskStop, TaskUpdate. These are all session-management and meta-agent tools.

**Enumeration**: There is no `AgentList` tool. Running agents cannot be enumerated via tools. Track agent IDs at dispatch time; use `SendMessage({to: agentId})` for re-entry (no `resume` parameter exists on this tool).

**Proactive resume**: `claude-code-guide` recommends checking for a running instance before spawning a new one.

**Thoroughness levels** (for `Explore` and `Plan`): Specify in `prompt` as `"quick"`, `"medium"`, or `"very thorough"`.

### Process Model

Agent subagents are NOT new OS-level `claude` processes. They run as API inference threads within the parent claude process. `pgrep -a claude` is unchanged before, during, and after agent dispatch. Each Bash tool call within an agent spawns a short-lived `rtk` proxy process (~5 MB, ephemeral) under the same parent claude process (see B27, B28).

Contrast: Skill tool invocations (`/doc_tsk`, `/role`, etc.) DO spawn new `claude --print --output-format json` OS processes.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [../param/003_agent.md](../param/003_agent.md) | Agent override parameter |
| doc | [../param/004_agents.md](../param/004_agents.md) | Custom agent definitions |
| doc | [036_send_message.md](036_send_message.md) | Send message to agent teammate |
| doc | [040_workflow.md](040_workflow.md) | Multi-subagent workflow orchestration |
| doc | [../subcommand/001_agents.md](../subcommand/001_agents.md) | Agents subcommand — lists configured agents |
| behavior | [../behavior/027_b27_agent_no_os_process.md](../behavior/027_b27_agent_no_os_process.md) | B27: agent subagents are not OS processes |
| behavior | [../behavior/028_b28_bash_rtk_subprocess.md](../behavior/028_b28_bash_rtk_subprocess.md) | B28: bash tool calls spawn transient rtk processes |
| behavior | [../behavior/029_b29_bash_claude_env.md](../behavior/029_b29_bash_claude_env.md) | B29: CLAUDE_* env vars propagated to all bash subprocesses including agent Bash calls |
| behavior | [../behavior/030_b30_subagent_context_inheritance.md](../behavior/030_b30_subagent_context_inheritance.md) | B30: subagents receive full CLAUDE.md via system-reminder; parent conversation not inherited; scope not propagated |
| behavior | [../behavior/031_b31_subagent_tool_sets.md](../behavior/031_b31_subagent_tool_sets.md) | B31: actual tool sets per subagent type; general-purpose lacks Agent; 14 tools parent-session-exclusive |
