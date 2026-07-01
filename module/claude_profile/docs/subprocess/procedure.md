# Subprocess Documentation Operations

- **Actor:** Developer
- **Trigger:** Addition of a new `run_isolated()` invocation site in `claude_profile`, changes to the credential write-back protocol, or changes to an existing invocation's parameters, result handling, or error recovery.
- **Emits:** —

## Add New Subprocess Instance

1. Verify the invocation context is not already covered by an existing instance (check `readme.md` Overview Table)
2. Confirm the invocation is within `claude_profile` scope — `run_isolated()` implementation internals in `claude_runner_core` are out of scope
3. Assign the next available ID (current highest ID in `readme.md` + 1)
4. Create `NNN_{invocation_context}.md`; include: `### Scope`, invocation-specific content sections (trigger condition, parameters, result handling, error recovery), typed reference sections
5. Add a row to `readme.md` Overview Table: `| NNN | [Name](NNN_file.md) | One-sentence purpose | ✅ |`

## Update Existing Subprocess Instance

1. Edit the target `NNN_*.md` file to reflect revised invocation parameters, changed error handling, or updated credential write-back behavior
2. Update typed reference sections if cross-references to features, state machines, or invariants changed
3. If the invocation context name changed materially: rename the file to match and update `readme.md` Overview Table row

## Retire Subprocess Instance

1. If an invocation site is removed from `claude_profile`, prepend a `> **Status: Retired** — invocation site removed` blockquote to the file
2. Update `readme.md` Overview Table Status column to `🗄️` and append `(retired)` to the Name link
3. Remove all cross-references to the retired instance from other doc instances
