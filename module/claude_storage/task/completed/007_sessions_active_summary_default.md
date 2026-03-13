# TSK-007: Show active session summary as default `clg .sessions` output

## Goal

Replace the session list with a focused active-session summary when `clg .sessions`
is invoked with no extra arguments — confirmed when bare `clg .sessions` prints the
most-recent session's ID, entry count, last-activity timestamp, and last message
(truncated per formula) instead of a multi-project session table.

## In Scope

- `src/cli/mod.rs` — detect when no filters are applied and switch to summary mode
- Active session = highest mtime across all sessions in scope
- Output: session ID, project path, entry count, last-activity age, last message
  (truncated: if `len > 50` → `{first30}...{last30}`, else full text)
- Parse the last JSONL entry of the active session to extract message text
- `unilang.commands.yaml` — document the new default mode if needed
- Tests in `tests/sessions_command_test.rs`

## Out of Scope

- Adding `mode::` parameter to toggle between summary and list (possible follow-up)
- Showing more than the last message (e.g. last N messages)
- Changing behavior when explicit filters are given (`scope::`, `path::`, `limit::`, etc.)
- `.show` command changes

## Description

`clg .sessions` currently lists every session across the subtree. For daily use the
relevant question is "what am I working on right now?" — a single active session
summary answers that immediately. The feature activates only for the no-args default
case; any explicit parameter keeps the existing list behavior. The last-message
truncation formula (`{first30}...{last30}` when len > 50) gives enough context to
recognize the conversation without flooding the terminal.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)

## Proposed Output

```
Active session  e28c6ac2  9d ago  6315 entries
Project  ~/pro/lib/wip_discovery/2026_03_ai_command_ascenyo_client/-default_topic

Last message:
  You have already read the f...s/organizational_principles.md
```

Edge cases:
- Message ≤ 50 chars: `Fix typo in readme` (full, no truncation)
- Message > 50 chars: `You have already read the f...s/organizational_principles.md`
- No sessions in scope: `No active session found.`
- Last entry has no text content (tool result, etc.): skip back until text entry found

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note CLI output style and JSONL entry format.
2. **Explore JSONL format** — read `docs/claude_code/jsonl_format.md`; identify which
   field holds message text (likely `message.content[].text` or `content[].text`).
3. **Write Test Matrix** — populate every row before writing test code.
4. **Write failing tests** — cover: summary output on no-args, truncation at exactly 50
   chars, truncation above 50, no sessions case, explicit filter still uses list mode.
5. **Implement** — add `is_default_invocation` detection; find highest-mtime session;
   read last text entry; apply truncation; format output.
6. **Green state** — `w3 .test l::3` passes with zero failures and zero warnings.
7. **Refactor if needed** — truncation logic as a pure function; no function > 50 lines.
8. **Walk Validation Checklist** — every answer YES.
9. **Update task status** — set ✅, move to `task/completed/`.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | No args, sessions exist | Default invocation | Summary mode: ID, age, count, last message |
| T02 | Last message = 40 chars | Truncation gate | Full message shown, no ellipsis |
| T03 | Last message = 51 chars | Truncation gate | `{first30}...{last30}` format |
| T04 | Last message = 60 chars | Truncation formula | first30 + `...` + last30 = 63 chars total |
| T05 | No sessions in scope | Empty state | `No active session found.` |
| T06 | Explicit `scope::local` given | Filter active | List mode (existing behavior unchanged) |
| T07 | Explicit `limit::5` given | Filter active | List mode (existing behavior unchanged) |

## Acceptance Criteria

- Bare `clg .sessions` outputs summary, not list
- Summary includes: session UUID (first 8 chars), age, entry count, project path,
  last message (truncated if > 50 chars)
- Truncation formula: len > 50 → `{first30}...{last30}`; len ≤ 50 → full text
- Any explicit parameter keeps existing list behavior
- `No active session found.` when scope has no sessions
- `w3 .test l::3` passes with zero failures

## Validation Checklist

Desired answer for every question is YES.

**Default behavior**
- [ ] Does `clg .sessions` (no args) print a summary, not a session table?
- [ ] Does the summary line include the session UUID, age, and entry count?
- [ ] Does the summary include the project path?
- [ ] Is the last message shown below the summary header?

**Truncation**
- [ ] Does a 40-char message appear in full with no ellipsis?
- [ ] Does a 51-char message appear as `{first30}...{last30}`?
- [ ] Is the truncated form exactly `first30 + "..." + last30` (63 chars)?

**Filter passthrough**
- [ ] Does `clg .sessions scope::local` still show the list, not the summary?
- [ ] Does `clg .sessions limit::5` still show the list?

**Negative criteria**
- [ ] Does `clg .sessions` output NOT contain the multi-line project table?

## Validation Procedure

### Measurements

**M1 — Summary mode active**
Command: `clg .sessions | head -1`
Before: `Found N sessions:` or project path header.
Expected: `Active session  {uuid}  ...` format. Deviation: old header = not implemented.

**M2 — Truncation correct**
Command: write fixture with 60-char last message; `clg .sessions`
Before: N/A. Expected: output contains `...` at position 30+3. Deviation: no `...` = truncation not applied.

**M3 — Filter passthrough**
Command: `clg .sessions scope::local | head -1`
Before: `Found N sessions:`. Expected: same `Found N sessions:` (list mode preserved).
Deviation: summary shown = filter detection broken.

### Anti-faking checks

**AF1 — Real JSONL parse**
Command: `clg .sessions` in a project where last message contains a known substring.
Expected: output contains that substring (or its truncation). Deviation: generic
placeholder text = JSONL parse not implemented.

## Outcomes

### Completed — 2026-04-05

**What was delivered:**
- Summary mode: bare `clg .sessions` shows active-session summary (ID, age, count, project path, last message)
- Truncation formula: `{first30}...{last30}` when len > 50 chars; full text otherwise
- Filter passthrough: any explicit param (`scope::`, `path::`, `session::`, `agent::`, `min_entries::`, `limit::`, `verbosity::`) activates list mode
- Empty-scope sentinel: `No active session found.`

**Files changed:**
- `src/cli/mod.rs` — `last_text_entry`, `truncate_message`, `render_active_summary` helpers + `is_default` detection + summary branch
- `tests/sessions_command_test.rs` — 7 new tests (IT-1 rewritten, IT-30..IT-35)
- `tests/common/mod.rs` — 2 new fixture helpers (`write_test_session_with_last_message`, `write_path_project_session_with_last_message`)
- `unilang.commands.yaml` — description/hint updated; removed `default:` from scope/limit/verbosity to enable `is_default` detection
- `readme.md` — updated output format section for summary mode
- `docs/cli/commands.md` — summary mode subsection, scope default fixed to `under`
- `docs/cli/testing/command/sessions.md` — IT-1 rewritten, IT-30..IT-35 added
- `docs/cli/testing/param/scope.md` — EC-7 rewritten for summary mode

**Root cause of key fix:**
YAML `default:` fields caused the framework to inject values before Rust code ran, making `get_string("scope").is_none()` always false. Removed YAML defaults; code already had `unwrap_or()` fallbacks after the `is_default` check.

**Test results:** 275 tests pass, clippy clean, doc tests pass (`w3 .test l::3` ✅)
