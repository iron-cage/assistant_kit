# TSK-084: Update `spec.md` — `--system-prompt` and `--append-system-prompt`

## Goal

`spec.md` § CLI Flags :: Claude-native flags does not document `--system-prompt` or
`--append-system-prompt`, which exist as builder methods in `claude_runner_core` (FR-28,
TSK-075) and are ready to be exposed via `clr`. The spec is the source of truth for the
DEV repo — the two missing rows block compliant implementation of the CLI flags. Done when
both flags appear in the Claude-native flags table, verified by
`grep -c 'system-prompt' spec.md` ≥ 2.

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/spec.md`
  § CLI Flags :: Claude-native flags table — add two rows after `--verbose`:
  - `| \`--system-prompt <TEXT>\` | Set system prompt (replaces the default) |`
  - `| \`--append-system-prompt <TEXT>\` | Append text to the default system prompt |`

## Out of Scope

- `docs/cli/` — six files updated in TSK-085
- Code implementation (`src/main.rs`) — separate feature task
- `claude_runner_core` spec.md — FR-28 already documents both builder methods there

## Description

`claude_runner_core` FR-28 (TSK-075) added `with_system_prompt()` and
`with_append_system_prompt()` to `ClaudeCommand`. These translate to `--system-prompt`
and `--append-system-prompt` respectively, which the claude CLI natively understands.
`clr` currently has no CLI flags for either — they are builder-only. Before implementing
the CLI flags, `spec.md` must be updated so the spec is ahead of (or aligned with) code.

The spec.md § CLI Flags has two tables:
1. Claude-native flags (passed through to claude) — target table
2. Runner-specific flags (not passed to claude)

Both new flags belong in table 1 because they are forwarded verbatim to the claude subprocess
via `ClaudeCommand.args`.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read spec.md** — open and locate § CLI Flags :: Claude-native flags table (after line 99).
2. **Add `--system-prompt` row** — insert after the `--verbose` row:
   `| \`--system-prompt <TEXT>\` | Set system prompt (replaces the default) |`
3. **Add `--append-system-prompt` row** — insert after `--system-prompt` row:
   `| \`--append-system-prompt <TEXT>\` | Append text to the default system prompt |`
4. **Verify** — `grep -c 'system-prompt' spec.md` returns ≥ 2; no duplications elsewhere.
5. **Update task status** — mark TSK-084 ✅ Complete in `task/readme.md`; move file to `completed/`.

## Test Matrix

| Scenario | Input | Expected |
|----------|-------|----------|
| Both flags in table | spec.md § Claude-native flags | 2 new rows present |
| Correct table | flags in Claude-native, not runner-specific | table 1, not table 2 |
| Style match | row format | matches adjacent rows (backtick flag, description) |
| No duplication | grep outside table | 0 extra occurrences |

## Acceptance Criteria

- `--system-prompt <TEXT>` row present in § CLI Flags :: Claude-native flags table
- `--append-system-prompt <TEXT>` row present in § CLI Flags :: Claude-native flags table
- Both rows placed in the Claude-native flags table, not the Runner-specific flags table
- Row format matches existing rows (backtick-wrapped flag, pipe-separated description)
- No stale or conflicting content introduced elsewhere in spec.md

## Validation Checklist

Desired answer for every question is YES.

- [ ] Is `--system-prompt` present in spec.md § CLI Flags :: Claude-native flags table?
- [ ] Is `--append-system-prompt` present in spec.md § CLI Flags :: Claude-native flags table?
- [ ] Are both rows in the Claude-native flags table (not Runner-specific)?
- [ ] Do both rows use backtick-wrapped flag syntax matching adjacent rows?
- [ ] Is the total flag count in the table correct (3 existing + 2 new = 5)?

## Validation Procedure

### Measurements

**M1 — Both flags in spec.md**
Command: `grep -c 'system-prompt' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/spec.md`
Before: 0. Expected: ≥ 2. Deviation: flag absent if < 2.

**M2 — append-system-prompt present**
Command: `grep -c 'append-system-prompt' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/spec.md`
Before: 0. Expected: ≥ 1. Deviation: flag absent if 0.

**M3 — Placed in correct table (Claude-native, not runner-specific)**
Command: `awk '/Claude-native flags/,/Runner-specific flags/' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/spec.md | grep -c 'system-prompt'`
Before: 0. Expected: 2. Deviation: wrong section if 0.

### Anti-faking checks

**AF1 — No duplication outside table**
Check: `grep 'system-prompt' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/spec.md | grep -vc '^\|'`
Expected: 0 (all occurrences are table rows, none in prose).
