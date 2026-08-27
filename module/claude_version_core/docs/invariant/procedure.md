# Invariant Documentation Operations

- **Actor:** Developer
- **Trigger:** A new invariant is identified, an existing constraint changes, or a pinned literal is bumped.
- **Emits:** —

## Rule

Every invariant here must ship with a command whose expected output is written down. An
invariant verifiable only by reading the code is a convention and belongs in `api/` or the
crate `readme.md`.

Two of this crate's constraints are **not** fully mechanical, and that must be stated rather
than papered over:

- [002_alias_literal_consistency.md](002_alias_literal_consistency.md)'s triage step is
  judgment — the commands produce a candidate list, not a verdict. Never present such a rule
  as a one-line `grep` fix; a blanket replace on that list actively damages unrelated fixtures.
- [001_layer_one_boundary.md](001_layer_one_boundary.md)'s Known Documentation Split records a
  deviation rather than enforcing a rule. When adding a `docs/…` citation to `src/`, add its
  row there in the same change.

Adding a dependency is an invariant change by definition, never a routine edit: it touches
INV-1 and INV-2 of `001_layer_one_boundary.md`.

## Add Invariant Documentation

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Write the Enforcement Mechanism section with a runnable command and its expected output; where a step needs human judgment, label it as judgment and give the triage rule
4. Run every command and confirm it produces the stated output before registering the doc
5. Register in `readme.md` Overview Table: add row with ID, Name, Purpose, Status
6. Increment the `invariant/` instance count in `../entity.md` and add a Master Doc Instances row

## Update Invariant Documentation

1. Edit the target `NNN_*.md` file
2. Re-run the Enforcement Mechanism commands and confirm they still produce the documented output
3. If a literal value appears in expected output, substitute the current one rather than leaving a stale example
4. If name or purpose changed: update `readme.md` Overview Table row and `../entity.md`

## Example

Adding invariant document `003_no_network_outside_install`:

1. Check `readme.md` Overview Table — current highest ID is `002`
2. Create `003_no_network_outside_install.md` in this directory
3. Enforcement: `grep -rn 'install.sh\|Command::new' module/claude_version_core/src/` → expected: hits only in `version.rs` install and detection paths, none in `config_*` or `params_catalog`
4. Run it — confirm the output matches before registering
5. Add row: `| 003 | [No Network Outside Install](003_no_network_outside_install.md) | Only the install and detection paths leave the process | ✅ |`
6. Bump `invariant/` instances to 3 in `../entity.md` and add the Master Doc Instances row
