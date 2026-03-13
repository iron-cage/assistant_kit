# Create docs/cli/parameter_interactions.md (L4 Completeness Blocker)

## Goal

Create `module/claude_runner/docs/cli/parameter_interactions.md` with all six mandatory
sections defined by `cli_design.rulebook.md`, documenting all known parameter interactions
in `claude_runner`'s CLI. This unblocks the documentation from L3 to L4 completion and
ensures every parameter co-dependency, conditional relationship, mutual exclusion, edge
case, cascading effect, and interaction test is captured in one authoritative location.

## In Scope

- Create `module/claude_runner/docs/cli/parameter_interactions.md` (new file)
- Cover all 9 current parameters: `message`, `model`, `continue`, `dry_run`, `verbose`,
  `output_file`, `session_id`, `max_turns`, `token_limit`
- All 6 mandatory sections per `cli_design.rulebook.md`:
  1. Co-Dependencies (parameters that must appear together)
  2. Conditional Parameters (parameters that enable other parameters)
  3. Mutually Exclusive (parameters that conflict)
  4. Edge Cases (interaction edge cases)
  5. Cascading Effects (one parameter changes behavior of others)
  6. Testing Matrix (interaction test coverage)
- Update `docs/cli/readme.md` completion matrix row for L4

## Out of Scope

- Changes to actual parameter implementation in source code
- Adding new parameters (covered by task 032)
- Changing `param::value` format (covered by task 031)
- Adding Methods to `types.md` (covered by task 030)

## Description

`cli_design.rulebook.md` mandates `parameter_interactions.md` with 6 required sections
as part of L4 documentation completeness. The file is entirely absent — this is the sole
blocker preventing the documentation from advancing from L3 to L4.

Known interactions that MUST be documented:

**Co-Dependencies**: `continue::true` implies an existing session must be detectable
(interacts with session detection path in `claude_session`); `output_file` and `message`
together define the complete I/O contract.

**Conditional**: `verbose::true` only has observable effect when actual Claude execution
runs (has no effect combined with `dry_run::true`); `max_turns` only applies during
multi-turn sessions.

**Mutually Exclusive**: `dry_run::true` makes `output_file` semantically meaningless
(no output is produced); `continue::true` conflicts with fresh-session semantics.

**Edge Cases**: `continue::true` + no prior session → error; `token_limit` at boundary
values; `model` name with unknown/invalid string; empty `message`.

**Cascading Effects**: `model` selection cascades to default `token_limit` behavior;
`verbose::true` cascades to Claude's own output verbosity (not runner verbosity).

**Testing Matrix**: Cross-reference table of parameter pairs and whether they have test
coverage for their interaction.

## Requirements

-   All work must strictly adhere to all applicable rulebooks
    (discover via `kbase .rulebooks`)
-   `cli_design.rulebook.md` — all 6 sections mandatory with complete content
-   `organizational_principles.rulebook.md` — Unique Responsibility; no duplication with
    `params.md` (interactions belong here, individual semantics belong in `params.md`)
-   File naming: `lowercase_snake_case`, no hyphen prefix (permanent file)
-   Update parent `docs/cli/readme.md` Responsibility Table with new file row

## Work Procedure

Execute in order. Do not skip or reorder steps.

1. **Read rulebooks** — `kbase .rulebooks`; re-read `cli_design.rulebook.md` § Parameter
   Interactions section for exact section requirements.
2. **Audit existing parameters** — read `docs/cli/params.md` to enumerate all 9 parameters
   and their documented behavior. Note any interaction hints already embedded there.
3. **Draft Co-Dependencies section** — list parameter pairs that require each other.
4. **Draft Conditional Parameters section** — list parameters that unlock/enable others.
5. **Draft Mutually Exclusive section** — list parameter pairs that cannot coexist.
6. **Draft Edge Cases section** — document interaction edge cases with expected behavior.
7. **Draft Cascading Effects section** — document parameters whose presence alters the
   behavior of other parameters.
8. **Draft Testing Matrix section** — table of parameter-pair interactions vs test coverage
   status (✅ covered, ⚠️ partial, ❌ missing).
9. **Update docs/cli/readme.md** — add `parameter_interactions.md` row to Responsibility
   Table; update completion matrix L4 row to ✅.
10. **Walk Validation List** — every answer must be YES.
11. **Update task status** — set ✅, recalculate advisability=0, move to `task/completed/`.

## Test Matrix

*(Not applicable — this task creates documentation, not test code.)*

## Acceptance Criteria

-   `docs/cli/parameter_interactions.md` exists and is non-empty
-   All 6 mandatory sections are present with substantive content
-   Every known interaction between the 9 parameters is captured
-   `docs/cli/readme.md` references `parameter_interactions.md` in its Responsibility Table
-   No duplication with `params.md` (interactions here, individual semantics there)

## Validation List

Desired answer for every question is YES.

**File existence and structure**
-   [ ] Does `module/claude_runner/docs/cli/parameter_interactions.md` exist?
-   [ ] Does it contain `## Co-Dependencies` section?
-   [ ] Does it contain `## Conditional Parameters` section?
-   [ ] Does it contain `## Mutually Exclusive` section?
-   [ ] Does it contain `## Edge Cases` section?
-   [ ] Does it contain `## Cascading Effects` section?
-   [ ] Does it contain `## Testing Matrix` section?

**Content quality**
-   [ ] Is every parameter (all 9) mentioned in at least one interaction section?
-   [ ] Does the Testing Matrix cover all parameter pairs that have known interactions?
-   [ ] Is the `dry_run + output_file` mutual exclusion documented?
-   [ ] Is the `verbose + dry_run` conditional relationship documented?
-   [ ] Is the `continue + no-prior-session` edge case documented?

**Cross-references**
-   [ ] Is `parameter_interactions.md` listed in `docs/cli/readme.md` Responsibility Table?
-   [ ] Does `docs/cli/readme.md` completion matrix show L4 as complete?

**Anti-duplication**
-   [ ] Does `parameter_interactions.md` avoid restating individual parameter semantics
    already documented in `params.md`?

## Validation Procedure

### Measurements

**M1 — Section count**
`grep -c '^## ' module/claude_runner/docs/cli/parameter_interactions.md`
Expected: exactly 6 (one per mandatory section).

**M2 — Parameter coverage**
`grep -c 'dry_run\|verbose\|continue\|output_file\|model\|message\|session_id\|max_turns\|token_limit' module/claude_runner/docs/cli/parameter_interactions.md`
Expected: ≥9 (every parameter mentioned at least once).

### Anti-faking checks

**AF1 — Substantive content, not placeholder**
Each section must contain at least 2 substantive lines beyond the heading.
`grep -A5 '## Co-Dependencies' parameter_interactions.md | wc -l` → expected: ≥5.

**AF2 — Testing Matrix is a real table**
`grep '|' module/claude_runner/docs/cli/parameter_interactions.md | wc -l`
Expected: ≥3 (header + separator + at least one data row).
