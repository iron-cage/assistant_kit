# Invariant Documentation Operations

- **Actor:** Developer
- **Trigger:** A new invariant is identified or an existing constraint changes.
- **Emits:** —

## Rule

Every invariant here must ship with a command whose expected output is written down — a
`grep`, a `cargo tree`, or a `cargo nextest` line. An invariant verifiable only by reading the
code is a convention and belongs in `feature/` or the crate `readme.md`.

Adding any dependency is an invariant change, never a routine edit: it touches
[001_zero_workspace_deps.md](001_zero_workspace_deps.md) by definition. Adding a *non-optional*
dependency additionally breaks [002_offline_parse_core.md](002_offline_parse_core.md) INV-1 —
update both, or don't add it.

## Add Invariant Documentation

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Write the Enforcement Mechanism section with a runnable command and its expected output
4. Run that command and confirm it produces the stated output before committing the doc
5. Register in `readme.md` Overview Table: add row with ID, Name, Purpose, Status
6. Increment the `invariant/` instance count in `../entity.md` and add a Master Doc Instances row

## Update Invariant Documentation

1. Edit the target `NNN_*.md` file
2. Re-run the Enforcement Mechanism command and confirm it still produces the documented output
3. If a pinned version appears in expected output (e.g. `ureq v3.3.0`), reconcile it against `cargo tree` rather than leaving it stale
4. If name or purpose changed: update `readme.md` Overview Table row and `../entity.md`

## Example

Adding invariant document `003_no_token_logging`:

1. Check `readme.md` Overview Table — current highest ID is `002`
2. Create `003_no_token_logging.md` in this directory
3. Enforcement: `grep -nE 'println!|eprintln!|dbg!' module/claude_auth/src/lib.rs` → expected empty
4. Run it — confirm empty before registering
5. Add row: `| 003 | [No Token Logging](003_no_token_logging.md) | Secrets never reach stdout or stderr | ✅ |`
6. Bump `invariant/` instances to 3 in `../entity.md` and add the Master Doc Instances row
