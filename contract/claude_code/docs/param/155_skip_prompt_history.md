# skip_prompt_history

Suppresses recording of user prompts into the prompt history file.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_SKIP_PROMPT_HISTORY` |
| Config Key | — |

### Type

bool

### Default

`false` (history is recorded)

### Since

Unverified. Present in the v2.1.220 binary (9 occurrences) but named by no entry in the 2.1.74–2.1.220 changelog, so no introduction version can be cited.

### Description

Prevents prompts from being appended to the prompt history that backs the up-arrow recall and the history file documented in [`../format/readme.md`](../format/readme.md).

**Certainty is presence, not semantics.** What the binary scan establishes is that the string exists and is referenced 9 times. The *effect* described above is inferred from the name and from the existence of the history format this collection documents — no test in this crate sets the variable and then inspects the history file. Treat the behavior as expected, not confirmed.

**Why it matters for automation.** Long unattended runs push large volumes of generated prompts into history, which both dilutes interactive recall and grows a file that `cleanupPeriodDays` then has to sweep. This variable is the documented lever for that, distinct from `--no-session-persistence` (which suppresses the session transcript, a different artifact).

### Verification

```bash
V=~/.local/share/claude/versions/2.1.220
grep -ac CLAUDE_CODE_SKIP_PROMPT_HISTORY "$V"   # → 9
grep -ac CLAUDE_CONFIG_DIR               "$V"   # → 28 (positive control)
grep -ac TOTALLY_FAKE_VAR_XYZ            "$V"   # → 0  (negative control)

# The absence of a release note is itself checkable:
grep -rl 'SKIP_PROMPT_HISTORY' ../version/*.md  # → no output
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [043_no_session_persistence.md](043_no_session_persistence.md) | Suppresses the session transcript (different artifact) |
| doc | [156_cleanup_period_days.md](156_cleanup_period_days.md) | Retention sweep that would otherwise reclaim this data |
| doc | [../format/readme.md](../format/readme.md) | History file format |
