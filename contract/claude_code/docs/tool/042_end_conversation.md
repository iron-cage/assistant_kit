# Tool: EndConversation

Terminates the conversation from the assistant side.

### Category

Interaction

### Permission Required

Unverified

### Description

Present in the v2.1.220 binary (7 string occurrences) and exposed as a deferred
tool in live v2.1.220 sessions. Absent from every documented tool list this
collection had until now.

**Documented usage constraint.** Live sessions surface this tool with an
explicit narrow mandate: use it only for sustained abuse directed at the
assistant, or when the user explicitly asks to see it demonstrated, and load its
full guidance via `ToolSearch` before invoking. That constraint arrives in the
session's own tool description, not from this collection.

**Why the low occurrence count is not evidence against it.** At 7 occurrences it
scans lower than most tools, but occurrence count tracks how many code paths
mention a string, not how real the tool is. `ReportFindings` scans at 2 and is
equally real. The negative control (0) is what makes any non-zero count
meaningful.

### Parameters

Unverified — schema is deferred and was not loaded. Retrieve it with:

```
ToolSearch  query="select:EndConversation"
```

### Since

Unverified. No entry in the 2.1.74–2.1.220 changelog names `EndConversation`.

### Verification

```bash
V=~/.local/share/claude/versions/2.1.220
for k in EndConversation ToolSearch TOTALLY_FAKE_TOOL_XYZ; do
  printf '%-24s %s\n' "$k" "$(grep -ac "$k" "$V")"
done   # → 7, 53, 0 (last is the negative control)

grep -rl 'EndConversation' ../version/*.md   # → no output; no release note exists
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [038_tool_search.md](038_tool_search.md) | Loads this tool's deferred schema |
| doc | [008_ask_user_question.md](008_ask_user_question.md) | Other Interaction-category tool |
| doc | [041_design_sync.md](041_design_sync.md) | Sibling tool found in the same audit |
