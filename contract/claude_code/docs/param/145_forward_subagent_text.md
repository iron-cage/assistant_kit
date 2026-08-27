# forward_subagent_text

Emits subagent text and thinking into the parent stream-json output.

### Forms

| | Value |
|-|-------|
| CLI Flag | `--forward-subagent-text` |
| Env Var | `CLAUDE_CODE_FORWARD_SUBAGENT_TEXT` |
| Config Key | — |

### Type

bool

### Default

`false`

### Since

v2.1.211 (2026-07-15) — [`../version/108_v2_1_211.md`](../version/108_v2_1_211.md): *"Added `--forward-subagent-text` flag and `CLAUDE_CODE_FORWARD_SUBAGENT_TEXT` environment variable to include subagent text and thinking in stream-json output"*

### Description

Help text:

> Forward subagent text and thinking blocks as assistant/user messages with `parent_tool_use_id` set (only works with `--print` and `--output-format=stream-json`)

**Two hard prerequisites.** The flag is inert without *both* `--print` and `--output-format=stream-json`. There is no other output mode that carries the forwarded messages.

**`parent_tool_use_id` is the correlation key.** Forwarded blocks arrive interleaved with the parent's own messages; the only way to attribute one to its originating subagent is the `parent_tool_use_id` field, which points at the `Agent` tool-use that spawned it. Consumers that ignore this field will read subagent output as if the parent produced it.

**Depth-2+ nesting arrived later.** v2.1.219 extended forwarding to subagents spawned at depth 2 and beyond, *"keyed by their spawning Agent `tool_use` id"*. On v2.1.211–v2.1.218 the flag forwards only the first level.

### Verification

```bash
claude --help | grep -A3 -- '--forward-subagent-text'

V=~/.local/share/claude/versions/2.1.220
grep -ac CLAUDE_CODE_FORWARD_SUBAGENT_TEXT "$V"   # → 3
grep -ac TOTALLY_FAKE_VAR_XYZ              "$V"   # → 0 (negative control)
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [033_include_partial_messages.md](033_include_partial_messages.md) | Related stream-json content expansion |
| doc | [147_include_hook_events.md](147_include_hook_events.md) | Sibling stream-json content expansion |
| doc | [../jsonl/readme.md](../jsonl/readme.md) | Entry format, including threading fields |
| doc | [../version/108_v2_1_211.md](../version/108_v2_1_211.md) | Release introducing the flag |
