# API Documentation Operations

- **Actor:** Developer
- **Trigger:** A public item is added, removed, or its signature, error set, or side effects change.
- **Emits:** —

## Rule

This directory documents the *contract*, not the algorithm. Every row must state something the
signature alone does not convey. For this crate specifically, four properties are load-bearing
and must be recorded whenever they apply:

1. **Purity** — does the function touch the filesystem, read `HOME`, or spawn a subprocess?
   `get_installed_version` and `get_claude_version_raw` differ chiefly in this, and a caller
   choosing between them needs it stated.
2. **Error absorption** — many functions here return `Option` or nothing at all and swallow
   I/O failure by design (`load_custom_markers`, `purge_stale_versions`, the install steps).
   Say so; a reader will otherwise assume the absence of `Result` means the operation cannot
   fail.
3. **Platform variance** — `read_versions_dir_lock_mode` has two `cfg`-gated definitions with
   materially different behavior. Any item with a `cfg( unix )` / `cfg( not( unix ) )` split
   must document both arms and what a caller may conclude on each.
4. **Totality vs. validation** — where a resolve/validate pair exists
   (`resolve_version_spec` / `validate_version_spec`), state that resolution never rejects, so
   the validation step is not optional.

Group by surface, not by module: 001 covers four small modules together because they form one
coherent configuration surface. Split only when a group stops being readable in one file.

## Add API Documentation

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Document each item's signature, error set, side effects, purity, and any `cfg` variance
4. Register in `readme.md` Overview Table: add row with ID, Name, Purpose, Status
5. Increment the `api/` instance count in `../entity.md` and add a Master Doc Instances row

## Update API Documentation

1. Edit the target `NNN_*.md` file
2. Reconcile every documented signature against `src/` — a stale signature here is worse than
   none, since consumers treat this file as authoritative
3. If a `CoreError` variant was added or removed, note the semver impact: the enum is
   exhaustively matchable by consumers, so any variant change is breaking
4. If name or purpose changed: update `readme.md` Overview Table row and `../entity.md`

## Example

Adding API document `003_history_cache_surface`:

1. Check `readme.md` Overview Table — current highest ID is `002`
2. Create `003_history_cache_surface.md` in this directory
3. Document signatures plus the four load-bearing properties above — in particular whether the
   cache read absorbs a corrupt-file error or surfaces it
4. Add row: `| 003 | [History Cache Surface](003_history_cache_surface.md) | On-disk release-history cache contract | ✅ |`
5. Bump `api/` instances to 3 in `../entity.md` and add the Master Doc Instances row
