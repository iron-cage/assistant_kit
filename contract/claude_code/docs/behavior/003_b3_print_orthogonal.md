# Behavior B3: Print Flag Is Orthogonal to Session Selection

### Scope

- **Purpose**: Document that `-p`/`--print` is an output mode control only and does not affect which session is selected or created.
- **Responsibility**: Authoritative instance for behavior B3 — defines the behavior statement, certainty level, and supporting evidence.
- **In Scope**: `-p`/`--print` as output-only flag; interaction with session creation default.
- **Out of Scope**: Session selection mechanics (→ [B5](005_b5_mtime_selection.md)); `--no-session-persistence` which does affect storage (→ [B22](022_b22_no_session_persistence.md)).

### Behavior

**Status**: ✅ Confirmed | **Certainty**: 95% | **Tier**: FLAG-VFY | **Evidence**: E3, E13

`-p` / `--print` switches output capture mode to non-interactive. It does not interact with `--continue` or session creation. A `-p` invocation starts a new session (binary default, B1) unless `-c` / `--continue` is also passed.

```
claude -p "message"          # new session + print output (non-interactive)
claude -p -c "message"       # resume session + print output
```

### Evidence

| ID | Supports | Type | Source | Location | Content |
|----|----------|------|--------|----------|---------|
| E3 | B3 | Code | `../../../../module/claude_runner/src/main.rs` | lines 83, 124 | `-p, --print  Non-interactive mode` and `-p` branch sets print-only; no session flag change |
| E13 | B3 | Test | `../../tests/behavior/b03_print_flag.rs` | `b3_print_flag_documented_as_output_mode` | `claude --help` documents `-p` / `--print` as output mode |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Master index: evidence table, statistical summary, invalidation tests |
| behavior | [022_b22_no_session_persistence.md](022_b22_no_session_persistence.md) | `--no-session-persistence` flag that does affect storage (requires `--print`) |
| test | `../../tests/behavior/b03_print_flag.rs` | Invalidation test |
