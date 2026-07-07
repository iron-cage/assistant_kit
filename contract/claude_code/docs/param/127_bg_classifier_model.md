# bg_classifier_model

Overrides the lightweight model used to classify and summarize background task state.

### Forms

| | Value |
|-|-------|
| CLI Flag | — |
| Env Var | `CLAUDE_CODE_BG_CLASSIFIER_MODEL` |
| Config Key | — |

### Type

string (model alias or full ID)

### Default

Binary default (an internal small/fast "classifier" model, in the same family as
`ANTHROPIC_SMALL_FAST_MODEL` and the auto-mode permission classifier — exact
default not confirmed; see Description)

### Since

≤ v2.1.197 (undocumented — not present in any changelog entry 001-098; confirmed
only via string/reference inspection of the installed binary at
`~/.local/share/claude/versions/2.1.197`)

### Description

Overrides which model performs "classifier" duties for background tasks — this
project's internal term for lightweight, fast model calls that make a small
judgment or produce a short summary rather than carry a full conversation (the
same vocabulary appears in `CLAUDE_CODE_ENABLE_AUTO_MODE`'s "auto-mode permission
classifier" and `CLAUDE_CODE_CLASSIFIER_SUMMARY`). For background tasks
specifically, this class of call most plausibly backs the one-line natural-language
summaries shown in background-task completion notifications (e.g. "Agent 'X'
finished").

This entry is inferred from static analysis of the shipped binary (the env var
name is registered in the same internal name→getter table as
`ANTHROPIC_SMALL_FAST_MODEL` and sibling model-selection vars) rather than from
official documentation — treat the exact scope of what it controls as
best-effort, not confirmed.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [readme.md](readme.md) | Master parameter table |
| doc | [079_subagent_model.md](079_subagent_model.md) | `CLAUDE_CODE_SUBAGENT_MODEL` — model override for full Agent-tool subagent sessions (contrast: this var is for lightweight classifier calls, not full sessions) |
| doc | [081_enable_auto_mode.md](081_enable_auto_mode.md) | `CLAUDE_CODE_ENABLE_AUTO_MODE` — the other documented "classifier" consumer in this codebase's vocabulary |
| doc | [128_bg_tasks_report_running.md](128_bg_tasks_report_running.md) | Sibling background-task env var discovered in the same research pass |
