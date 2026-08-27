# Tool: DesignSync

Synchronizes design assets or design-system state into the session.

### Category

Extensibility

### Permission Required

Unverified

### Description

Present in the v2.1.220 binary (24 string occurrences) and exposed as a deferred
tool in live v2.1.220 sessions, but absent from every documented tool list this
collection had until now — the reason this instance exists.

**What is established:** the tool name exists in the binary at a frequency
comparable to other confirmed tools (`TodoWrite` scans at 13, `EndConversation`
at 7), and it appears in a live session's deferred-tool listing. That places it
firmly in the same class as the 40 already documented.

**What is NOT established:** its parameter schema, its permission behavior, and
its exact function. Deferred tools ship only a name until `ToolSearch` loads the
schema on demand, so a session that never requests it never sees its signature.
Nothing in this crate's tests, and no official Claude Code documentation cited
in this collection, specifies either. The one-line summary above is inferred
from the name and is explicitly a placeholder.

### Parameters

Unverified — schema is deferred and was not loaded. Retrieve it with:

```
ToolSearch  query="select:DesignSync"
```

### Since

Unverified. No entry in the 2.1.74–2.1.220 changelog names `DesignSync`.

### Verification

```bash
V=~/.local/share/claude/versions/2.1.220
for k in DesignSync TodoWrite TOTALLY_FAKE_TOOL_XYZ; do
  printf '%-24s %s\n' "$k" "$(grep -ac "$k" "$V")"
done   # → 24, 13, 0 (last is the negative control)

grep -rl 'DesignSync' ../version/*.md   # → no output; no release note exists
```

To recover the schema, call `ToolSearch` with `select:DesignSync` inside a
session and read the returned JSONSchema — that is the only first-party source
for its parameters.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [038_tool_search.md](038_tool_search.md) | Loads this tool's deferred schema |
| doc | [042_end_conversation.md](042_end_conversation.md) | Sibling tool found in the same audit |
| doc | [043_report_findings.md](043_report_findings.md) | Sibling tool found in the same audit |
