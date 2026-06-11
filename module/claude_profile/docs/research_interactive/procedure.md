# Research Interactive Documentation Operations

- **Actor:** Developer
- **Trigger:** Discovery of non-obvious Claude binary behavior relevant to subprocess execution, authentication handling, or process interaction design.
- **Emits:** —

## Capture New Research Finding

1. Verify the finding is not already covered by an existing instance (check `readme.md` Responsibility Table)
2. Assign the next available ID (current highest ID in `readme.md` + 1)
3. Create `NNN_{snake_case_topic}.md` in this directory; include: observed behavior, reproduction method, design implications, and open questions
4. Add a row to `readme.md` Responsibility Table: `| NNN_file.md | Brief topic summary |`

## Update Existing Research Finding

1. Edit the target `NNN_*.md` file to reflect revised findings
2. If topic scope changed materially: update `readme.md` Responsibility Table row description

## Retire Stale Research Finding

1. If a finding no longer reflects reality (e.g., Claude binary behavior changed), prepend a **Status: Superseded** note to the file — do not delete
2. Append `(superseded)` to the `readme.md` Responsibility Table row description
