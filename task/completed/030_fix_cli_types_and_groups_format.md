# Fix docs/cli types.md and parameter_groups.md Formatting Gaps

## Goal

Fix three formatting gaps in `claude_runner/docs/cli/` that prevent the documentation
from meeting `cli_design.rulebook.md` format standards: (1) add sequential numbers to all
4 type headers in `types.md`, (2) add Methods sections to all 4 types in `types.md`, and
(3) add "Why NOT" rationale subsections to all 4 parameter groups in `parameter_groups.md`.
All changes are documentation-only. Verified by manual inspection against rulebook requirements.

## In Scope

- `module/claude_runner/docs/cli/types.md` — 4 type headers gain `N.` prefix; 4 types gain
  Methods sections listing their public interface methods
- `module/claude_runner/docs/cli/parameter_groups.md` — 4 parameter groups gain
  "Why NOT [Group Name]" subsections explaining excluded parameters

## Out of Scope

- Adding new types (VerbosityLevel is covered in task 032)
- Changing the `param::value` format (covered in task 031)
- Creating `parameter_interactions.md` (covered in task 029)
- Changes to source code

## Description

Two documentation files have format violations against `cli_design.rulebook.md`:

**types.md violations:**

*Missing sequential numbers* — `cli_design.rulebook.md` § Type Documentation requires
`### Type :: N. \`TypeName\`` format. All 4 headers currently use `### Type :: \`TypeName\``
(no sequential number). Fix: prefix each with `1.`, `2.`, `3.`, `4.`.

*Missing Methods sections* — Each type must document its public interface methods
(`get()`, `is_X()` predicates, `as_str()`, validation methods). None of the 4 types
(`MessageText`, `PathArg`, `TokenCount`, `ModelName`) have a Methods section.

**parameter_groups.md violations:**

*Missing "Why NOT" rationale* — `cli_design.rulebook.md` § Parameter Groups requires each
group to include a "Why NOT [Group Name]" subsection explaining what parameters were
considered for inclusion but excluded, and why. This prevents future grouping inconsistency
and preserves the reasoning behind current group membership. None of the 4 groups
(Input, Environment, Behavior Flags, Resource Control) have this subsection.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   `cli_design.rulebook.md` — sequential type numbers; Methods per type; "Why NOT" per group
-   File naming: no changes to file names (files already exist)
-   No duplication: Methods sections describe interface only, not restate Description content
-   "Why NOT" sections explain exclusion decisions, not just absent parameters

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; re-read `cli_design.rulebook.md` for exact
   Type header format and "Why NOT" section requirements.
2. **Fix types.md headers** — add `1.`, `2.`, `3.`, `4.` to the four type section headers.
3. **Add Methods section to MessageText** — document `get() -> &str`, `is_empty() -> bool`,
   `as_str() -> &str` and any validation methods.
4. **Add Methods section to PathArg** — document `get() -> &Path`, `as_str() -> &str`,
   `exists() -> bool` (if applicable), and validation.
5. **Add Methods section to TokenCount** — document `get() -> u32`, `is_zero() -> bool`,
   numeric interface methods.
6. **Add Methods section to ModelName** — document `get() -> &str`, `as_str() -> &str`,
   `is_known() -> bool` (if applicable).
7. **Add "Why NOT Input" to parameter_groups.md** — explain why `continue`, `session_id`,
   `model` etc. are NOT in the Input group.
8. **Add "Why NOT Environment" subsection** — explain exclusion reasoning for Environment group.
9. **Add "Why NOT Behavior Flags" subsection** — explain why `continue`, `model`, etc.
   are not in Behavior Flags (this is the most important one — these are the border cases).
10. **Add "Why NOT Resource Control" subsection** — explain exclusion reasoning.
11. **Walk Validation List** — every answer must be YES.
12. **Update task status** — set ✅, recalculate advisability=0, move to `task/completed/`.

## Test Matrix

*(Not applicable — documentation-only task.)*

## Acceptance Criteria

-   All 4 type headers in `types.md` use `### Type :: N. \`TypeName\`` format
-   All 4 types in `types.md` have a Methods section with at least 2 methods documented
-   All 4 parameter groups in `parameter_groups.md` have "Why NOT [Group Name]" subsections
    with substantive reasoning (not just "these parameters exist elsewhere")

## Validation List

Desired answer for every question is YES.

**types.md headers**
-   [ ] Does `### Type :: 1. \`MessageText\`` appear in types.md?
-   [ ] Does `### Type :: 2. \`PathArg\`` appear in types.md?
-   [ ] Does `### Type :: 3. \`TokenCount\`` appear in types.md?
-   [ ] Does `### Type :: 4. \`ModelName\`` appear in types.md?
-   [ ] Are all original un-numbered headers removed?

**types.md Methods sections**
-   [ ] Does MessageText have a Methods section with at least `get()` and `as_str()`?
-   [ ] Does PathArg have a Methods section?
-   [ ] Does TokenCount have a Methods section?
-   [ ] Does ModelName have a Methods section?

**parameter_groups.md "Why NOT" subsections**
-   [ ] Does the Input group have "Why NOT Input" content?
-   [ ] Does the Environment group have "Why NOT Environment" content?
-   [ ] Does the Behavior Flags group have "Why NOT Behavior Flags" content?
-   [ ] Does the Resource Control group have "Why NOT Resource Control" content?
-   [ ] Does each "Why NOT" subsection mention at least one specific excluded parameter
    with reasoning?

**Anti-duplication**
-   [ ] Do Methods sections describe interface only (not restate Description content)?
-   [ ] Do "Why NOT" subsections explain exclusion reasoning (not just list parameters)?

## Validation Procedure

### Measurements

**M1 — Header format verification**
`grep '### Type ::' module/claude_runner/docs/cli/types.md`
Expected: 4 lines, all matching `### Type :: [0-9]+\.` pattern.

**M2 — Methods section count**
`grep -c '^**Methods**\|^### Methods' module/claude_runner/docs/cli/types.md`
Expected: 4 (one per type).

**M3 — Why NOT section count**
`grep -c 'Why NOT' module/claude_runner/docs/cli/parameter_groups.md`
Expected: 4 (one per group).

### Anti-faking checks

**AF1 — Methods have actual content**
For each Methods section, verify it contains at least one method signature in backticks.
`grep -A3 'Methods' module/claude_runner/docs/cli/types.md | grep '`' | wc -l` → expected: ≥4.

**AF2 — "Why NOT" sections have parameter names**
`grep -A5 'Why NOT' module/claude_runner/docs/cli/parameter_groups.md | grep '`' | wc -l`
Expected: ≥4 (each "Why NOT" mentions at least one parameter by name).
