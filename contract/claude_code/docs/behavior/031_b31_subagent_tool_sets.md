# Behavior B31: Subagent Tool Sets Are Narrower Than Parent Session; general-purpose Cannot Spawn Subagents

### Scope

- **Purpose**: Document the actual tool sets available to each subagent type, the 14 tools exclusive to the parent session, and the correction that general-purpose subagents do NOT have access to the Agent tool despite the documented claim of "All (*)".
- **Responsibility**: Authoritative instance for behavior B31 — defines the behavior statement, certainty level, and supporting evidence. Tier is UNVERIFIED.
- **In Scope**: Observed deferred tool lists per subagent type; pre-loaded vs. deferred distinction; parent-session-exclusive tools; general-purpose Agent tool absence; Explore/Plan tool set identity; claude-code-guide loading model (pre-loaded only, no ToolSearch).
- **Out of Scope**: Environment variable inheritance (→ [B29](029_b29_bash_claude_env.md)); context/CLAUDE.md inheritance (→ [B30](030_b30_subagent_context_inheritance.md)); process model (→ [B27](027_b27_agent_no_os_process.md)); tool parameter schemas (→ `../tool/`).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 99% | **Tier**: UNVERIFIED | **Since**: v2.1.74 | **Evidence**: E57

Each subagent type receives a distinct, constrained tool set. All subagents receive fewer tools than the parent interactive session (27 tools). The documented "All (*)" claim for `general-purpose` is empirically incorrect — it excludes all session-management and meta-agent tools.

**Observed tool sets by type:**

| Tool | Parent | `general-purpose` | `Explore` | `Plan` | `claude-code-guide` |
|------|:------:|:-----------------:|:---------:|:------:|:-------------------:|
| ToolSearch | ✅ pre | ✅ pre | ✅ pre | ✅ pre | ❌ |
| Read | ✅ def | ✅ def | ✅ def | ✅ def | ✅ pre |
| Glob | ✅ def | ✅ def | ✅ def | ✅ def | ✅ pre |
| Grep | ✅ def | ✅ def | ✅ def | ✅ def | ✅ pre |
| WebFetch | ✅ def | ✅ def | ✅ def | ✅ def | ✅ pre |
| WebSearch | ✅ def | ✅ def | ✅ def | ✅ def | ✅ pre |
| Bash | ✅ def | ✅ def | ✅ def | ✅ def | ❌ |
| EnterWorktree | ✅ def | ✅ def | ✅ def | ✅ def | ❌ |
| ExitWorktree | ✅ def | ✅ def | ✅ def | ✅ def | ❌ |
| Skill | ✅ def | ✅ def | ✅ def | ✅ def | ❌ |
| Edit | ✅ def | ✅ def | ❌ | ❌ | ❌ |
| Write | ✅ def | ✅ def | ❌ | ❌ | ❌ |
| NotebookEdit | ✅ def | ✅ def | ❌ | ❌ | ❌ |
| Agent | ✅ def | ❌ | ❌ | ❌ | ❌ |
| AskUserQuestion | ✅ def | ❌ | ❌ | ❌ | ❌ |
| CronCreate | ✅ def | ❌ | ❌ | ❌ | ❌ |
| CronDelete | ✅ def | ❌ | ❌ | ❌ | ❌ |
| CronList | ✅ def | ❌ | ❌ | ❌ | ❌ |
| EnterPlanMode | ✅ def | ❌ | ❌ | ❌ | ❌ |
| ExitPlanMode | ✅ def | ❌ | ❌ | ❌ | ❌ |
| LSP | ✅ def | ❌ | ❌ | ❌ | ❌ |
| TaskCreate | ✅ def | ❌ | ❌ | ❌ | ❌ |
| TaskGet | ✅ def | ❌ | ❌ | ❌ | ❌ |
| TaskList | ✅ def | ❌ | ❌ | ❌ | ❌ |
| TaskOutput | ✅ def | ❌ | ❌ | ❌ | ❌ |
| TaskStop | ✅ def | ❌ | ❌ | ❌ | ❌ |
| TaskUpdate | ✅ def | ❌ | ❌ | ❌ | ❌ |
| **Total** | **27** | **13** | **10** | **10** | **5** |

`pre` = pre-loaded (full schema immediately available) · `def` = deferred (name only; ToolSearch required before invocation)

**Specific findings:**

1. **general-purpose does NOT have Agent.** It cannot spawn sub-subagents. The tool/007_agent.md "All (*) — Can Spawn Agents: ✅" claim is empirically incorrect. The Agent tool does not appear in the deferred list.

2. **Explore and Plan are identical.** Both receive the same 10 tools: ToolSearch (pre) + Bash, EnterWorktree, ExitWorktree, Glob, Grep, Read, Skill, WebFetch, WebSearch (deferred). The documented exclusion list ("all except Agent, ExitPlanMode, Edit, Write, NotebookEdit") understates the actual restriction — Cron*, Task*, LSP, AskUserQuestion, EnterPlanMode are also absent.

3. **claude-code-guide uses a static pre-loaded model.** Its 5 tools (Glob, Grep, Read, WebFetch, WebSearch) are fully loaded at conversation start with complete schemas. There is no ToolSearch and no deferred list — the toolset is fixed and non-extensible.

4. **14 tools are parent-session-exclusive** — available only in the interactive parent session, absent from all subagent types:
   `Agent`, `AskUserQuestion`, `CronCreate`, `CronDelete`, `CronList`, `EnterPlanMode`, `ExitPlanMode`, `LSP`, `TaskCreate`, `TaskGet`, `TaskList`, `TaskOutput`, `TaskStop`, `TaskUpdate`
   These are all session-management and meta-agent tools — not execution tools.

5. **ToolSearch is the bootstrapping gateway** for all subagent types except `claude-code-guide`. It is always pre-loaded (never deferred) in types that have it. It is absent from `claude-code-guide` entirely, consistent with that type having a fixed, pre-resolved tool set.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E57 | B31 | Experiment | 4-agent parallel tool inventory — this session (2026-06-29) | Agents a0421c818fd857c2b (general-purpose), a5c1902758f7bef17 (Explore), afa16d2f3f479ce74 (Plan), a4e092d7ff1371904 (claude-code-guide) | Each agent reported its complete available-deferred-tools list verbatim. general-purpose: 12 deferred + ToolSearch (no Agent). Explore: 9 deferred + ToolSearch. Plan: 9 deferred + ToolSearch (identical to Explore). claude-code-guide: 5 pre-loaded tools only, no ToolSearch, no deferred. Parent session (same conversation): 26 deferred + ToolSearch = 27 total. |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| tool | [../tool/007_agent.md](../tool/007_agent.md) | Agent tool — Built-in Subagent Types table (corrected per B31 findings) |
| behavior | [030_b30_subagent_context_inheritance.md](030_b30_subagent_context_inheritance.md) | B30: CLAUDE.md context inheritance (separate concern from tool access) |
| behavior | [027_b27_agent_no_os_process.md](027_b27_agent_no_os_process.md) | B27: agent subagents not OS processes — process model context |
