# Invariant: Tools Array Doc Sync

### Scope

- **Purpose**: Ensure `clr tools`'s hardcoded `TOOLS` array always presents the complete, accurate set of Claude Code built-in tools — never a stale subset left behind after the contract documentation grows.
- **Responsibility**: State the bijection requirement between `tools.rs`'s `TOOLS` array and `contract/claude_code/docs/tool/readme.md`'s Tool Table, and the enforcement test that keeps it true.
- **In Scope**: The `TOOLS: &[(&str, &str, &str)]` static array in `src/cli/tools.rs` (name, category, description per entry) and its source of truth, `contract/claude_code/docs/tool/readme.md`'s Tool Table.
- **Out of Scope**: The contract documentation's own accuracy (→ maintained independently as Claude Code's own tool inventory); `clr tools`'s filtering/projection/format behavior (→ `docs/cli/command/08_tools.md`, `docs/cli/param_group/07_tool_listing.md`).

### Invariant Statement

For every row in `contract/claude_code/docs/tool/readme.md`'s Tool Table, there
MUST exist exactly one corresponding entry in `tools.rs`'s `TOOLS` array with
the same tool name and the same category, and vice versa — the two collections
form a bijection. In particular, `TOOLS.len()` MUST equal the contract doc's
Tool Table row count at all times; this must hold as a general correctness
property that tracks the doc source, not as a one-time hardcoded count.

### Enforcement Mechanism

A test reads `contract/claude_code/docs/tool/readme.md`'s Tool Table (row
count and, at minimum, the set of tool names) and asserts equality against
`TOOLS.len()` and the set of names in `TOOLS`, failing loudly — not silently
truncating or ignoring extras — on any divergence in either direction (doc
grew without a matching code update, or code has a name absent from the doc).

### Violation Consequences

If this invariant is violated (as it was found to be during the 2026-07
`clr tools` CLI redesign — `TOOLS` held only 26 of the doc's 40 entries,
having not been regenerated when 14 later tools, e.g. `TodoWrite`, `Artifact`,
`Monitor`, `PowerShell`, `PushNotification`, `RemoteTrigger`, `ScheduleWakeup`,
`SendMessage`, `Workflow`, were added to the contract documentation):
`clr tools` silently omits tools that are actually available to the model,
misleading users constructing `--allowed-tools`/`--disallowed-tools` flags
into believing a real, callable tool does not exist. The failure is silent —
no error, panic, or warning signals that the displayed set is incomplete.

### Sources

| File | Notes |
|------|-------|
| `../../src/cli/tools.rs` | `TOOLS` static array — the enforced side of the bijection |
| `../../../../contract/claude_code/docs/tool/readme.md` | Tool Table — the authoritative source of truth |

### Tests

| File | Notes |
|------|-------|
| `../../tests/tools_command_test.rs` | Sync-guard test comparing `TOOLS` against the contract doc's Tool Table (added as part of the 2026-07 `clr tools` redesign; see task tracking `TOOLS` array regeneration) |

### Commands

| File | Relationship |
|------|--------------|
| [docs/cli/command/08_tools.md](../cli/command/08_tools.md) | The command whose data source this invariant guards |
