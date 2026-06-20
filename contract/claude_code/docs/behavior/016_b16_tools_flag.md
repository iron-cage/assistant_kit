# Behavior B16: Tools Flag Controls Tool Invocation

### Scope

- **Purpose**: Document that `--tools ""` disables all tool invocation and `--tools "default"` restores all tools.
- **Responsibility**: Authoritative instance for behavior B16 — defines the tools flag semantics, certainty level, and supporting evidence.
- **In Scope**: `--tools ""` (disable all), `--tools "default"` (restore all), `--tools "Bash,Edit,Read"` (named subset); acceptance at CLI parse time.
- **Out of Scope**: Tool definition presence in system prompt even when disabled (→ [B16h](016h_b16h_tools_system_prompt.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 90% | **Tier**: FLAG-VFY | **Since**: pre-v1.0 | **Evidence**: E30, E31

`--tools ""` disables all tool invocation; `--tools "default"` restores all tools; both values are accepted at CLI parse time without parse error.

The flag also accepts specific tool names: `--tools "Bash,Edit,Read"` enables only those tools.

**Note:** FLAG-VFY tier means the test confirms the flag is documented and accepted at parse time. Actual invocation-blocking behavior requires a live `lim_it_` test (runs by default in container). B16h documents a related uncertainty about tool definitions in the assembled system prompt.

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E30 | B16 | Observation | `claude --help` live output | `--tools` flag entry | Help text: "Specify the list of available tools from the built-in set. Use `""` to disable all tools, `"default"` to use all tools, or specify tool names" |
| E31 | B16 | Test | `../../tests/behavior/b16_tools_disable.rs` | `b16a_tools_flag_documented_in_help`, `b16b_tools_empty_string_accepted`, `b16c_tools_default_value_accepted` | Flag documented in help and accepted at CLI parse time without parse error |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [016h_b16h_tools_system_prompt.md](016h_b16h_tools_system_prompt.md) | Tool definitions remain in system prompt even when `--tools ""` is used |
| test | `../../tests/behavior/b16_tools_disable.rs` | Invalidation test (FLAG-VFY) |
