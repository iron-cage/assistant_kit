# Tool: ReportFindings

Reports code-review findings as a typed list the host UI renders.

### Category

Interaction

### Permission Required

No

### Description

Present in the v2.1.220 binary (2 string occurrences) and exposed in live
v2.1.220 sessions. Absent from every documented tool list this collection had
until now.

**Unlike its two sibling discoveries, this one is not deferred** — its schema is
loaded eagerly in a live session, so its contract is directly observable rather
than requiring a `ToolSearch` round-trip. That is why the parameter table below
is populated where [`041_design_sync.md`](041_design_sync.md) and
[`042_end_conversation.md`](042_end_conversation.md) both say "unverified."

**Conditional-use tool.** It is called only when the active code-review
instructions direct findings to be reported through it; otherwise the review
follows whatever output format those instructions specify. When it is used, it
is called **once** with all verified findings ranked most-severe first (an empty
array if nothing survived verification), and the findings are not also printed
as prose — doing both duplicates them in the UI.

**Re-reporting after fixes.** If the applying instructions ask for it, each
finding carries an `outcome` recording what actually happened: `fixed`,
`skipped`, or `no_change_needed`.

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `findings` | array (max 32) | yes | Verified findings, most-severe first; empty if none survived |
| `level` | enum | no | Effort level the review ran at (`low`/`medium`/`high`/`xhigh`/`max`) |

Each `findings` element:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `file` | string | yes | Repo-relative path the finding is in |
| `summary` | string | yes | One-sentence statement of the defect |
| `failure_scenario` | string | yes | Concrete inputs/state → wrong output/crash |
| `line` | integer | no | 1-indexed line the finding anchors to |
| `category` | string (≤40) | no | kebab-case type slug, e.g. `correctness`, `efficiency` |
| `short_summary` | string (≤60) | no | Compact label: the claim alone, no rationale |
| `verdict` | enum | no | `CONFIRMED` / `PLAUSIBLE`; absent on inline-only reviews |
| `outcome` | enum | no | `fixed` / `skipped` / `no_change_needed`; only when re-reporting |

### Since

Unverified. No entry in the 2.1.74–2.1.220 changelog names `ReportFindings`.

### Verification

```bash
V=~/.local/share/claude/versions/2.1.220
for k in ReportFindings EndConversation TOTALLY_FAKE_TOOL_XYZ; do
  printf '%-24s %s\n' "$k" "$(grep -ac "$k" "$V")"
done   # → 2, 7, 0 (last is the negative control)

grep -rl 'ReportFindings' ../version/*.md   # → no output; no release note exists
```

A count of 2 is low but decisive against the 0 control — the string is present.
The parameter table above comes from the schema as loaded in a live v2.1.220
session, which is a stronger source than the scan.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master tool table |
| doc | [041_design_sync.md](041_design_sync.md) | Sibling tool found in the same audit |
| doc | [042_end_conversation.md](042_end_conversation.md) | Sibling tool found in the same audit |
| doc | [027_todo_write.md](027_todo_write.md) | Other structured-output Interaction tool |
