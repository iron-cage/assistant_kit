# TSK-085: Update `docs/cli/` — `--system-prompt` and `--append-system-prompt`

## Goal

Six `docs/cli/` files for `claude_runner` don't document `--system-prompt` or
`--append-system-prompt`, creating inconsistency between the builder API (FR-28) and the
user-facing CLI reference. Updating all six files unblocks accurate user-facing documentation
and enables the new CLI flags to be discoverable. Done when the parameter count reaches 15
in all affected headings and tables, verified by
`grep -c 'system-prompt' docs/cli/commands.md` ≥ 2.

## In Scope

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/readme.md`
  § Navigation — fix "13 parameters" → 15; "5 types" → 6; "2 groups" → 3

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/commands.md`
  § All Commands table — fix `run` Params count 13 → 15
  § Command :: 1 `run` Parameters table — add two rows after `--trace`:
  - `--system-prompt` row linking to params.md #parameter--14
  - `--append-system-prompt` row linking to params.md #parameter--15

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/params.md`
  § All Parameters heading — fix "(13 total)" → "(15 total)"
  § All Parameters summary table — add rows 14 and 15
  § Groups note — append "Parameters 14–15 form [System Prompt](...)"
  § Body — add Parameter :: 14 and Parameter :: 15 sections after Parameter :: 13

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/parameter_groups.md`
  § All Groups table — fix "2 total" → "3 total"; add Group 3 row
  § Body — add Group :: 3 "System Prompt" section

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/types.md`
  § All Types table — fix "5 total" → "6 total"; add `SystemPromptText` row
  § Body — add Type :: 6 `SystemPromptText` section

- `/home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/workflows.md`
  § All Workflows table — fix "8 total" → "9 total"; add Workflow 9 row
  § Body — add Workflow :: 9 "Custom System Prompt" section

## Out of Scope

- `spec.md` — covered in TSK-084
- `docs/cli/dictionary.md` — no new vocabulary entry required for system prompt
- Code implementation (`src/main.rs`) — separate feature task

## Description

`claude_runner_core` FR-28 (TSK-075) added `with_system_prompt()` and
`with_append_system_prompt()`. When `clr` exposes these as `--system-prompt <TEXT>` and
`--append-system-prompt <TEXT>`, all six docs/cli/ reference files must reflect the
expanded surface.

**Parameter numbering:** `--system-prompt` = param 14, `--append-system-prompt` = param 15.
Both are sequentially next after the current 13.

**Type:** Both flags use a new type `SystemPromptText` (Type 6). This is semantically
distinct from `MessageText` (Type 1, the positional user-turn argument) — the system prompt
sets the model's behavioral context, not the user turn message.

**Group:** Both flags belong to a new Group 3 "System Prompt". The existing groups are
"Claude-Native Flags" (params 2–4, flags passed to claude subprocess) and "Runner Control"
(params 5–13, flags consumed by the runner). Although `--system-prompt` and
`--append-system-prompt` ARE passed to the claude subprocess, adding them to Group 1 would
create a non-contiguous range (2–4 and 14–15). A dedicated Group 3 keeps ranges clean and
makes the system-prompt concern explicit. Group coherence test: "Is this flag used to
inject or extend the system prompt sent to claude?" — YES for both.

**Workflow:** Workflow 9 "Custom System Prompt" demonstrates the primary use case:
narrowing Claude's behavior for domain-specific automation.

### Exact content for new additions

**params.md summary table rows:**
```
| 14 | `--system-prompt` | [`SystemPromptText`](types.md#type--6-systemprompttext) | — | Any text | Set system prompt (replaces the default) | 1 cmd |
| 15 | `--append-system-prompt` | [`SystemPromptText`](types.md#type--6-systemprompttext) | — | Any text | Append text to the default system prompt | 1 cmd |
```

**parameter_groups.md Group 3 row (in All Groups table):**
```
| 3 | System Prompt | 2 | Flags that inject or extend the system prompt sent to claude |
```

**types.md Type 6 row (in All Types table):**
```
| 6 | `SystemPromptText` | String | [`--system-prompt`](params.md#parameter--14---system-prompt), [`--append-system-prompt`](params.md#parameter--15---append-system-prompt) | Free-form system prompt text |
```

**commands.md Parameters table rows (after --trace row):**
```
| [`--system-prompt`](params.md#parameter--14---system-prompt) | [`SystemPromptText`](types.md#type--6-systemprompttext) | — | Set system prompt (replaces the default) |
| [`--append-system-prompt`](params.md#parameter--15---append-system-prompt) | [`SystemPromptText`](types.md#type--6-systemprompttext) | — | Append text to the default system prompt |
```

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; note CLI documentation constraints.

2. **Edit `readme.md`** —
   a. Navigation "13 parameters" → "15 parameters".
   b. Navigation "5 types" → "6 types".
   c. Navigation "2 groups" → "3 groups".

3. **Edit `types.md`** —
   a. All Types heading: "(5 total)" → "(6 total)".
   b. All Types table: add row after VerbosityLevel:
      `| 6 | \`SystemPromptText\` | String | [\`--system-prompt\`](...), [\`--append-system-prompt\`](...) | Free-form system prompt text |`
   c. Body: add Type :: 6 `SystemPromptText` section after Type :: 5.
      Content: Base type String, Constraints any UTF-8 text no length limit, Parsing consumed
      as next token after `--system-prompt` or `--append-system-prompt`, Used by both params.

4. **Edit `parameter_groups.md`** —
   a. All Groups table heading: "2 total" → "3 total".
   b. All Groups table: add Group 3 row.
   c. Body: add Group :: 3 "System Prompt" section after Group :: 2.
      Content: coherence test "Is this flag used to inject or extend the system prompt?" YES for
      both. Parameters table with --system-prompt and --append-system-prompt rows. Why NOT in
      this group: `--model`, `--print`, `--verbose` (Claude-native but not prompt-related),
      all Runner Control flags (consumed by runner).

5. **Edit `params.md`** —
   a. Heading: "(13 total)" → "(15 total)".
   b. Summary table: add rows 14 and 15 after row 13.
   c. Groups note: append sentence "Parameters 14–15 form [System Prompt](parameter_groups.md#group--3-system-prompt)."
   d. Body: add Parameter :: 14 section for `--system-prompt` after Parameter :: 13.
      Fields: Type SystemPromptText, Default — (none; default system prompt unchanged when absent),
      Command run, Group System Prompt, note that omitting this flag leaves Claude's built-in
      system prompt in effect.
   e. Body: add Parameter :: 15 section for `--append-system-prompt` after Parameter :: 14.
      Fields: Type SystemPromptText, Default — (none; nothing appended when absent),
      Command run, Group System Prompt, note that this is additive (does not replace);
      use --system-prompt to fully replace.

6. **Edit `commands.md`** —
   a. All Commands table: run row Params `13` → `15`.
   b. Command :: 1 Parameters table: add two rows after `--trace` row (see § Description for
      exact row content).

7. **Edit `workflows.md`** —
   a. All Workflows table heading: "8 total" → "9 total".
   b. All Workflows table: add row 9: `| 9 | Custom system prompt | \`--system-prompt\` | Domain-specific behavior injection |`
   c. Body: add Workflow :: 9 "Custom System Prompt" section after Workflow :: 8.
      Show: `clr --system-prompt "You are a Rust expert. Be concise." "Review this PR"`,
      and `clr --append-system-prompt "Always respond in JSON." "List failing tests"`.
      Explain: --system-prompt replaces default (strongest override), --append-system-prompt
      adds constraints on top of default (lighter touch).

8. **Verify** — run all measurements from Validation Procedure.
9. **Update task status** — mark TSK-085 ✅ Complete in `task/readme.md`; move file to `completed/`.

## Test Matrix

| Scenario | Check target | Expected |
|----------|-------------|----------|
| readme.md param count | Navigation line | "15 parameters" |
| readme.md type count | Navigation line | "6 types" |
| readme.md group count | Navigation line | "3 groups" |
| commands.md param count | All Commands run row | 15 |
| commands.md param rows | Parameters table | 2 new rows present |
| params.md total | Heading | "(15 total)" |
| params.md summary | Table | rows 14 and 15 present |
| params.md groups note | Groups line | mentions Parameters 14–15 |
| params.md body 14 | Parameter :: 14 section | present, complete |
| params.md body 15 | Parameter :: 15 section | present, complete |
| parameter_groups.md count | All Groups table | 3 total |
| parameter_groups.md Group 3 | Group :: 3 section | present, 2 params |
| types.md count | All Types heading | "(6 total)" |
| types.md Type 6 | All Types table + body | present |
| workflows.md count | All Workflows heading | "9 total" |
| workflows.md Workflow 9 | table row + body | present |
| No stale "13 parameters" | readme.md Navigation | 0 occurrences |
| No stale "5 types" | readme.md Navigation | 0 occurrences |
| No stale "2 groups" | readme.md Navigation | 0 occurrences |

## Acceptance Criteria

- `readme.md` Navigation shows 15 parameters, 6 types, 3 groups
- `commands.md` `run` command documents 15 parameters; both new rows present in Parameters table
- `params.md` heading shows 15 total; rows 14 and 15 in summary table; groups note updated; two new body sections present
- `parameter_groups.md` shows 3 groups; Group :: 3 "System Prompt" section present with 2 parameters
- `types.md` shows 6 total types; Type :: 6 `SystemPromptText` section present
- `workflows.md` shows 9 total workflows; Workflow :: 9 "Custom System Prompt" section present
- All cross-references between files use correct `#anchor` IDs

## Validation Checklist

Desired answer for every question is YES.

- [ ] Does readme.md Navigation show "15 parameters"?
- [ ] Does readme.md Navigation show "6 types"?
- [ ] Does readme.md Navigation show "3 groups"?
- [ ] Does commands.md All Commands table show run Params = 15?
- [ ] Are `--system-prompt` and `--append-system-prompt` rows in Command :: 1 Parameters table?
- [ ] Does params.md heading show "(15 total)"?
- [ ] Are rows 14 and 15 in the params.md summary table?
- [ ] Does the params.md Groups note reference "Parameters 14–15" and "System Prompt"?
- [ ] Is the Parameter :: 14 body section present in params.md?
- [ ] Is the Parameter :: 15 body section present in params.md?
- [ ] Does parameter_groups.md All Groups table show "3 total"?
- [ ] Is the Group :: 3 "System Prompt" section present in parameter_groups.md?
- [ ] Does types.md All Types table show "(6 total)"?
- [ ] Is the Type :: 6 `SystemPromptText` section present in types.md?
- [ ] Does workflows.md All Workflows table show "9 total"?
- [ ] Is the Workflow :: 9 "Custom System Prompt" section present in workflows.md?
- [ ] Are all `params.md#parameter--14` and `params.md#parameter--15` anchors correct?
- [ ] Is there no remaining "13 parameters" in readme.md Navigation?

## Validation Procedure

### Measurements

**M1 — commands.md documents both flags**
Command: `grep -c 'system-prompt' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/commands.md`
Before: 0. Expected: ≥ 2. Deviation: missing flag docs if < 2.

**M2 — params.md total count**
Command: `grep 'All Parameters' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/params.md`
Before: "13 total". Expected: "15 total". Deviation: stale count if not updated.

**M3 — params.md has rows 14 and 15**
Command: `grep -c '^| 1[45] ' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/params.md`
Before: 0. Expected: 2. Deviation: missing summary rows if < 2.

**M4 — parameter_groups.md has 3 groups**
Command: `grep 'All Groups' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/parameter_groups.md`
Before: "2 total". Expected: "3 total". Deviation: group not added if not updated.

**M5 — types.md has 6 types**
Command: `grep 'All Types' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/types.md`
Before: "5 total". Expected: "6 total". Deviation: type not added if not updated.

**M6 — workflows.md has 9 workflows**
Command: `grep 'All Workflows' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/workflows.md`
Before: "8 total". Expected: "9 total". Deviation: workflow not added if not updated.

### Anti-faking checks

**AF1 — No stale "13 parameters" in readme.md Navigation**
Check: `grep '13 parameters' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/readme.md`
Expected: 0 matches.

**AF2 — Parameter :: 14 body section exists**
Check: `grep -c 'Parameter :: 14' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/params.md`
Expected: ≥ 1.

**AF3 — Parameter :: 15 body section exists**
Check: `grep -c 'Parameter :: 15' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/params.md`
Expected: ≥ 1.

**AF4 — Group :: 3 body section exists**
Check: `grep -c 'Group :: 3' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/parameter_groups.md`
Expected: ≥ 1.

**AF5 — Type :: 6 body section exists**
Check: `grep -c 'Type :: 6' /home/user1/pro/lib/wip_core/claude_tools/dev/module/claude_runner/docs/cli/types.md`
Expected: ≥ 1.
