# cleanup_period_days

Retention window, in days, for locally-stored conversation and support data.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | — |
| Config Key | `cleanupPeriodDays` (settings.json) |

### Type

integer (days) — **`0` is rejected**, see below

### Default

30 days

### Since

≤v2.1.83 (2026-03-24) — [`../version/009_v2_1_83.md`](../version/009_v2_1_83.md) already treats it as established. No introduction entry in the 2.1.74–2.1.220 window.

### Description

Drives a background sweep that deletes locally-stored data older than the configured window.

**`cleanupPeriodDays: 0` is a validation error, not "disable".** v2.1.89 ([`../version/014_v2_1_89.md`](../version/014_v2_1_89.md)) changed `0` from a silent disable into a rejected value: *"Changed `cleanupPeriodDays: 0` in settings.json to be rejected with a validation error — it previously silently disabled transcript persistence."* A settings file written for an older version and carrying `0` will now fail validation. This is the highest-value fact in this doc: the same literal value changed from "keep everything" to "refuse to start."

**Sweep coverage widened over time.** Two changes moved the boundary of what the retention window governs:

| Version | Change |
|---------|--------|
| v2.1.83 | Fixed tool-result files never being cleaned up — they had ignored the setting entirely |
| v2.1.117 | Sweep extended to `~/.claude/tasks/`, `~/.claude/shell-snapshots/`, and `~/.claude/backups/` |

**A `--setting-sources` interaction caused real data loss.** v2.1.101: *"Fixed `--setting-sources` without `user` causing background cleanup to ignore `cleanupPeriodDays` and delete conversation history older than 30 days."* Omitting `user` from the setting sources dropped the configured value and the sweep fell back to the 30-day default — deleting history a longer window was meant to preserve.

### Verification

```bash
V=~/.local/share/claude/versions/2.1.220
grep -ac cleanupPeriodDays "$V"          # → 12

# The rejection is directly observable — inspect current value first:
grep -n cleanupPeriodDays ~/.claude/settings.json 2>/dev/null || echo "not set (default 30)"

# Changelog provenance for each claim above:
grep -rn 'cleanupPeriodDays' ../version/*.md
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [155_skip_prompt_history.md](155_skip_prompt_history.md) | Suppresses history at write time rather than sweeping it later |
| doc | [../settings/readme.md](../settings/readme.md) | settings.json structure and write semantics |
| doc | [../storage/readme.md](../storage/readme.md) | The directories this sweep covers |
