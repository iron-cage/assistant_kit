# Behavior B14: Agent meta.json Sidecars

### Scope

- **Purpose**: Document that agent JSONL files have sibling `.meta.json` files recording the spawn arguments of the agent — always `agentType`, plus up to nine optional fields.
- **Responsibility**: Authoritative instance for behavior B14 — defines the sidecar format, certainty level, and supporting evidence.
- **In Scope**: `.meta.json` sidecar file; the `agentType` field and its seven known values; the nine optional fields (`spawnDepth`, `description`, `toolUseId`, `isFork`, `model`, `parentAgentId`, `stoppedByUser`, `worktreePath`, `worktreeBranch`); value distribution; the nested `subagents/workflows/wf_{id}/` sidecar location.
- **Out of Scope**: Agent JSONL entry format (→ [`../jsonl/010_sidechain_sessions.md`](../jsonl/010_sidechain_sessions.md)); agent directory layout (→ [B13](013_b13_subagent_directory.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 95% | **Tier**: VALIDATED | **Since**: pre-v1.0 | **Evidence**: E24, E28, E77

Each agent JSONL file may have a sibling `.meta.json` file containing agent metadata:

```json
{"agentType":"Explore"}
{"agentType":"general-purpose"}
{"agentType":"Plan"}
{"agentType":"claude-code-guide"}
{"agentType":"Explore","description":"Read organizational principles rulebook"}
{"agentType":"workflow-subagent","spawnDepth":1}
{"agentType":"fork","isFork":true,"description":"Inventory tmu's full bash CLI surface","toolUseId":"toolu_01YbnCFu9uHfJmJyhvfEYP6Y","spawnDepth":1}
{"agentType":"claude","description":"Run full PROC1-S9 VERIFY Gate on BUG-1029","toolUseId":"toolu_016bg24eViEEgyQW9cheLzSv","spawnDepth":1}
{"agentType":"Explore","description":"Survey uni_* provider ecosystem","toolUseId":"toolu_01VDAzvdC4N4ynnnoSUqYB3B","spawnDepth":1,"model":"sonnet"}
{"agentType":"general-purpose","description":"Review scan.rs streaming scanner file","toolUseId":"toolu_019uWoiTgNaeggLQmVUZ5h6v","parentAgentId":"a9568f1d6cca9e144","spawnDepth":2}
{"agentType":"general-purpose","description":"Verify Part A tsk.rulebook.md citations","toolUseId":"toolu_01LqRHUJw6G3bSGjdVStv5zL","spawnDepth":1,"stoppedByUser":true}
{"agentType":"general-purpose","worktreePath":"/…/.claude/worktrees/agent-a06da593ca0f6733d","worktreeBranch":"worktree-agent-a06da593ca0f6733d","description":"Hunt genuine docs/ coverage gaps","toolUseId":"toolu_01YZNbQ8FCWzTbXRG26vh1x3","spawnDepth":1}
```

**Known `agentType` values** — full census of all 16713 sidecars carrying the field,
2026-08-27, v2.1.220. Counts drift as sessions run; shares are the stable figure:

| `agentType` | Count | Share | Notes |
|-------------|-------|-------|-------|
| `general-purpose` | 11088 | 66.3% | |
| `Explore` | 5217 | 31.2% | |
| `workflow-subagent` | 269 | 1.6% | Default subagent type of the `Workflow` tool; sidecars land in `subagents/workflows/wf_{id}/`, not directly in `subagents/` |
| `fork` | 119 | 0.7% | Carries `"isFork":true` — 1:1, see below |
| `Plan` | 10 | 0.06% | |
| `claude-code-guide` | 6 | 0.04% | |
| `claude` | 4 | 0.02% | Catch-all agent type |

The three types below `Plan` in that table were all absent from this document until an
audit on 2026-08-27; each is independently corroborated by a live session's own tool
schemas — `fork` and `claude` appear in the Agent tool's available-agent-type listing,
`workflow-subagent` is named in the `Workflow` tool description as its default.

**Fields.** The prior version of this document listed two (`agentType`, `description`)
and claimed `description` appears "only on some `Explore` agents". Both were wrong: a
full key census finds ten fields, and `description` is near-universal across types.

| Field | Count | Type | Meaning |
|-------|-------|------|---------|
| `agentType` | 16713 | string | Always present — it is what makes a sidecar a sidecar |
| `spawnDepth` | 16095 | integer | Nesting depth: 1 (14857), 2 (1055), 3 (160), 4 (23) |
| `description` | 15769 | string | Task summary passed at spawn |
| `toolUseId` | 15766 | string | `toolu_`-prefixed ID of the tool call that spawned the agent |
| `isFork` | 119 | boolean | Always `true`; present on exactly the `fork` sidecars and no others |
| `model` | 76 | string | Model override, as the **alias** not the API ID: `sonnet` (57), `opus` (19) |
| `parentAgentId` | 11 | string | Spawning agent's ID; observed only with `spawnDepth` ≥ 2 |
| `stoppedByUser` | 8 | boolean | Always `true`; the agent was interrupted |
| `worktreePath` | 3 | string | Absolute path of the agent's git worktree |
| `worktreeBranch` | 3 | string | Branch name in that worktree |

Three of these correspond exactly to documented Agent tool parameters — `model` to its
`model` override, `worktreePath`/`worktreeBranch` to `isolation: "worktree"` — which is
independent corroboration that the sidecar records spawn arguments. Semantics are ❓
Uncertain beyond the observed shapes: no first-party documentation of this format exists,
so `parentAgentId`'s ID space and whether `spawnDepth` is bounded at 4 or merely unobserved
past it are inferences from naming and census, not confirmed facts.

Two fields are absent whenever false rather than written as `false`: `isFork` and
`stoppedByUser` each show `true` in 100% of occurrences. Treat them as presence flags,
not booleans to be read.

**Two storage layouts.** Sidecars are written either directly in
`{session}/subagents/` (16384 with `agentType`) or nested under
`{session}/subagents/workflows/wf_{id}/` (329 — 269 `workflow-subagent`, 60 `Explore`).
Any consumer that reads this directory non-recursively silently misses the nested set;
that defect is recorded as Fix(A4) in
[`../../tests/behavior/mod.rs`](../../tests/behavior/mod.rs).

Re-derive every number above:

```bash
cd ~/.claude/projects   # see the two traps below before substituting an absolute path

# agentType census
find . -path '*/subagents/*' -name '*.meta.json' -size +0 \
  -exec grep -ho '"agentType"[[:space:]]*:[[:space:]]*"[^"]*"' {} + \
  | sed 's/.*"\([^"]*\)"$/\1/' | sort | uniq -c | sort -rn

# every key present, with frequency
find . -path '*/subagents/*' -name '*.meta.json' -size +0 \
  -exec grep -ho '"[a-zA-Z]*"[[:space:]]*:' {} + | tr -d '": ' | sort | uniq -c | sort -rn

# nested vs flat
find . -path '*/subagents/*/*' -name '*.meta.json' | wc -l   # nested only
find . -path '*/subagents/*'   -name '*.meta.json' | wc -l   # both layouts
```

**Trap 1 — do not extract with `sed -n s///p`.** No sidecar ends with a newline. GNU sed
preserves that, so each file's extracted value is emitted unterminated and consecutive
values concatenate into one token (`general-purposefork`), silently corrupting the census.
`grep -o` always terminates its matches, which is why the recipe uses it. Reproduce the
trap: `printf '{"agentType":"a"}' > /tmp/x.json; sed -n 's/.*"\([a-z]*\)"}/\1/p' /tmp/x.json | xxd | tail -1`
— no trailing `0a`.

**Trap 2 — `find` under an absolute `~/.claude` path can return nothing.** Some sandboxed
shells let `find` open that directory but not enumerate it, so it prints the root and exits
**0** with no results, which is indistinguishable from "no sidecars exist". `cd` into the
directory and use a relative `.` root, then sanity-check against `ls -1 | wc -l` before
trusting a zero.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E24 | B14 | Observation | Live storage | `~/.claude/projects/*/subagents/*.meta.json` | `meta.json` files contain `{"agentType":"Explore"}` or `{"agentType":"general-purpose"}` or `{"agentType":"Plan"}`; some include `description` |
| E28 | B14 | Test | `../../tests/behavior/b14_agent_meta_json.rs` | `b14_meta_json_contains_agent_type` | Real `.meta.json` file contains `agentType` field with known value |
| E77 | B14 | Experiment | Live storage census (2026-08-27, v2.1.220) | `~/.claude/projects/**/subagents/**/*.meta.json` | 16713 sidecars carrying `agentType`: general-purpose 11088, Explore 5217, workflow-subagent 269, fork 119, Plan 10, claude-code-guide 6, claude 4. Layout split: 16384 flat in `subagents/`, 329 nested in `subagents/workflows/wf_*/` (269 workflow-subagent + 60 Explore). Full key census finds ten fields, not two: `agentType` 16713, `spawnDepth` 16095 (values 1-4), `description` 15769, `toolUseId` 15766, `isFork` 119, `model` 76, `parentAgentId` 11, `stoppedByUser` 8, `worktreePath` 3, `worktreeBranch` 3 |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [007_b7_agent_sessions_sibling.md](007_b7_agent_sessions_sibling.md) | Flat agent layout (no `.meta.json` sidecars in flat format) |
| behavior | [013_b13_subagent_directory.md](013_b13_subagent_directory.md) | Hierarchical agent layout (directory containing the sidecar) |
| jsonl | [`../jsonl/010_sidechain_sessions.md`](../jsonl/010_sidechain_sessions.md) | Agent JSONL entry format |
| test | `../../tests/behavior/b14_agent_meta_json.rs` | Invalidation test |
