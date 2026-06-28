# Behavior B30: Agent Subagents Inherit Configuration Context But Not Conversation History

### Scope

- **Purpose**: Document exactly what context Agent tool subagents receive at conversation start: all CLAUDE.md configuration files are fully loaded; the parent agent's conversation history is absent and inaccessible.
- **Responsibility**: Authoritative instance for behavior B30 — defines the behavior statement, certainty level, and supporting evidence. Tier is UNVERIFIED (no automated invalidation test yet).
- **In Scope**: CLAUDE.md content availability in subagents; system-reminder injection mechanism; parent conversation history absence; scope variable propagation (`SCOPE_DIR`/`SCOPE_READY`); `isSidechain` JSONL marker; the `prompt` parameter as the sole data boundary crossing.
- **Out of Scope**: Agent process model (→ [B27](027_b27_agent_no_os_process.md)); `CLAUDE_*` environment variable propagation (→ [B29](029_b29_bash_claude_env.md)); session storage layout (→ [B13](013_b13_subagent_directory.md)); Bash subprocess identity (→ [B28](028_b28_bash_rtk_subprocess.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 99% | **Tier**: UNVERIFIED | **Since**: v2.1.74 | **Evidence**: E55

Agent subagents receive the full configuration context (all CLAUDE.md files and the auto-memory file) assembled at conversation start, but receive no parent conversation history.

**What IS available to subagents:**
- `~/.claude/CLAUDE.md` — global user instructions (all six absolute rules, SOPs, vocabulary)
- `~/.claude/RTK.md` and any other `@`-referenced files in CLAUDE.md
- `{project}/CLAUDE.md` — project-level instructions
- Auto-memory file (`~/.claude/projects/{slug}/memory/MEMORY.md`) when referenced
- All `CLAUDE_*` and project environment variables (`$PRO`, `$GENAI`, `$ENTRY`, `$PBK`, etc. — see B29)

**What is NOT available to subagents:**
- Parent conversation history — turns, tool calls, discoveries, and session findings from the parent
- `SCOPE_DIR`, `SCOPE_READY`, `SCOPE_LEVEL` — scope must be re-established explicitly (run `eval "$(scope)"`)
- Any in-memory state accumulated by the parent agent during the session

**Mechanism — system-reminder injection:**
- CLAUDE.md content is injected at API call time via a system-reminder block, not stored in the subagent JSONL. This is why subagents know all rules without reading any files.
- The subagent JSONL starts at `parentUuid: null` with `isSidechain: true` on every entry — the parent conversation chain is entirely absent from the stored record.
- The only data crossing the parent → subagent boundary at dispatch time is the `prompt` parameter value passed to the Agent tool.

**JSONL structural markers:**
- `isSidechain: true` — present on every entry; canonical marker for sidechain isolation
- `parentUuid: null` — first entry only; no parent chain injected
- First `message.content`: contains only the task prompt text; no prior conversation turns

**Consequence for subagent task design:**
- Subagents are rules-aware from conversation start (know all CLAUDE.md constraints, vocabulary, scope discipline, MAAV requirements) but context-blind (no knowledge of parent session findings or ongoing decisions).
- All task-specific context — file paths, prior observations, relevant state, background facts — must be explicitly included in the `prompt` parameter.
- This isolation is also why MAAV produces valid independent validation: each subagent begins with a clean context slate, making self-verification forgery structurally impossible across the parent→subagent boundary.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E55 | B30 | Experiment | Dual MAAV agent experiment — this session (2026-06-28) | Agents ae4bc9897199f0fef (probe) and a4ee9bfe2aedf5c12 (adversarial) | Probe agent answered 10/10 CLAUDE.md knowledge questions YES before reading any files (2-space indent, cargo fmt forbidden, scope command, MAAV, kbase, temp file naming, w3 .test — all known). Mechanism confirmed as system-reminder injection: probe re-read `~/.claude/CLAUDE.md` and confirmed content exactly matched what was already in context — no new information acquired. Adversarial agent confirmed zero knowledge of parent conversation (pgrep experiments, wplan_daemon discussion) — JSONL starts at `parentUuid: null`, `isSidechain: true`. `SCOPE_DIR`/`SCOPE_READY`/`SCOPE_LEVEL` absent from both agents' environments. |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [027_b27_agent_no_os_process.md](027_b27_agent_no_os_process.md) | B27: agent subagents are not OS processes (process model) |
| behavior | [028_b28_bash_rtk_subprocess.md](028_b28_bash_rtk_subprocess.md) | B28: Bash tool calls spawn transient rtk processes |
| behavior | [029_b29_bash_claude_env.md](029_b29_bash_claude_env.md) | B29: CLAUDE_* env vars inherited by bash subprocesses (not scope vars) |
| behavior | [013_b13_subagent_directory.md](013_b13_subagent_directory.md) | B13: subagent session storage at `{parent-uuid}/subagents/agent-{id}.jsonl` |
| tool | [../tool/007_agent.md](../tool/007_agent.md) | Agent tool — `prompt` parameter note: "agents start fresh unless `resume` is set" |
| behavior | [031_b31_subagent_tool_sets.md](031_b31_subagent_tool_sets.md) | B31: tool sets per subagent type — sister concern to context inheritance |
