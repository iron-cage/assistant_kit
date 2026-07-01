# Pitfall Documentation Operations

- **Actor:** Developer
- **Trigger:** A bug fix reveals a cross-feature systemic design trap (two or more related bugs in the same area), or a design review identifies a non-obvious constraint not captured in feature docs.
- **Emits:** —

## Add New Pitfall Instance

1. Verify the pitfall pattern is not already documented in an existing instance (check `readme.md` Overview Table)
2. Confirm the pitfall has cross-feature relevance — single-occurrence bugs belong in bug files, not pitfall instances
3. Assign the next available ID (current highest ID in `readme.md` + 1)
4. Create `NNN_{snake_case_topic}.md`; include: `### Scope`, `### Pattern`, one or more numbered `### Pitfall N — ...` sections (each with Fix and Rule subsections), typed reference sections
5. Add a row to `readme.md` Overview Table: `| NNN | [Name](NNN_file.md) | One-sentence purpose | ✅ |`

## Update Existing Pitfall Instance

1. Edit the target `NNN_*.md` file to add new pitfall sub-sections, update fix details, or revise rules
2. Update typed reference sections if new bugs, features, or algorithms are referenced
3. If the pitfall pattern scope changed materially: update `readme.md` Overview Table row Purpose column

## Retire Pitfall Instance

1. If a pitfall is resolved by a systemic architectural change that eliminates the failure mode entirely, prepend a `> **Status: Resolved** — [description of architectural change]` blockquote to the file
2. Update `readme.md` Overview Table Status column to `🗄️` and append `(resolved)` to the Name link
