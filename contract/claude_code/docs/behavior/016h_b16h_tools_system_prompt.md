# Behavior B16h: Tool Definitions Remain in System Prompt

### Scope

- **Purpose**: Document the uncertain hypothesis that tool definitions (~12k tokens) remain in the assembled system prompt even when `--tools ""` blocks invocation.
- **Responsibility**: Authoritative instance for behavior B16h — defines the sub-hypothesis, certainty level, and inference basis. Tier is MEASURE (live comparison required).
- **In Scope**: Tool definition token cost; architectural split between definition-assembly layer and invocation-policy layer; MEASURE tier explanation.
- **Out of Scope**: `--tools` flag acceptance at parse time (→ [B16](016_b16_tools_flag.md)).

### Behavior

**Status**: ❓ Uncertain | **Certainty**: 60% | **Tier**: MEASURE | **Evidence**: E32

Tool *definitions* (~12k tokens) remain in the assembled system prompt even when `--tools ""` is given — invocation is blocked but the token cost is unchanged.

**Architectural basis**: Tool definitions are injected into the assembled system prompt before behavioral flags are applied (confirmed for `--system-prompt` replacement via research on Piebald-AI/claude-code-system-prompts and ClaudeLog 2026-04). The `--tools` flag likely operates at the invocation-policy layer (what the model is permitted to call), not the definition-assembly layer (what goes into the system prompt). This is the same architectural split observed for `--system-prompt`.

**Unconfirmed**: Requires live token-count comparison — two identical conversations, one with `--tools "default"` and one with `--tools ""`, measuring the difference in total input tokens. The MEASURE test does this and runs by default in container where `~/.claude` is mounted.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E32 | B16h | Inference | Research: Piebald-AI/claude-code-system-prompts; ClaudeLog (2026-04) | Tool assembly layer analysis | Tool definitions injected into assembled system prompt before behavioral flags are applied. `--tools` likely operates at invocation-policy layer, not definition-assembly layer. Unconfirmed: requires live token-count comparison. |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [016_b16_tools_flag.md](016_b16_tools_flag.md) | `--tools` flag invocation control |
| test | `../../tests/behavior/b16h_tools_system_prompt.rs` | MEASURE test (lim_it; runs by default in container) |
