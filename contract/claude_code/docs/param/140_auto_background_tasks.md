# auto_background_tasks

Force-enable automatic backgrounding of long-running agent tasks.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_AUTO_BACKGROUND_TASKS` |
| Config Key | — |

### Type

bool

### Default

`false` (unset)

### Since

≤v2.1.197 (documented)

### Description

Set to `1` to force-enable automatic backgrounding: subagents that run past
roughly two minutes are automatically moved to the background rather than
continuing to block the main thread.

Naming note: this variable has **no** `_CODE_` infix, unlike most other
variables in this collection — the name is `CLAUDE_AUTO_BACKGROUND_TASKS`,
not `CLAUDE_CODE_AUTO_BACKGROUND_TASKS`. A search for the `_CODE_` form
returns zero matches in the installed binary; only the shorter name exists.
This is easy to get wrong when working from memory or an unverified summary.

### Cross-References

| Type | File | Responsibility |
|------|------|-----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [127_bg_classifier_model.md](127_bg_classifier_model.md) | Adjacent `_CODE_`-prefixed variable — naming contrast |
| doc | [136_disable_background_tasks.md](136_disable_background_tasks.md) | Global kill-switch this heuristic is subject to |
