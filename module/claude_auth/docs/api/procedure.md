# API Documentation Operations

- **Actor:** Developer
- **Trigger:** A public item is added, removed, or its signature or error set changes.
- **Emits:** —

## Rule

This directory documents the *contract*, not the behavior — what a caller can rely on without
reading `src/`. Every row must state something a signature alone does not convey: which
`AuthError` variants a function can return, whether it blocks, whether it retries, whether it
reads a clock, and which feature it requires.

Because `claude_auth` has one public surface, prefer editing
[001_auth_surface.md](001_auth_surface.md) over adding an instance. A second instance is
warranted only if the crate grows a genuinely separate surface (e.g. a device-authorization
flow alongside refresh).

`002_token_refresh_api.md` currently sits alongside 001 in apparent violation of that rule. It
is not a sanctioned second surface — it is an unconsolidated merge artifact awaiting a
keep/fold decision, per [readme.md](readme.md) § Overview Table. Do not treat it as precedent
for adding a third.

Any signature change is a semver event: adding a variant to `AuthError` is breaking, since the
enum is exhaustively matchable by consumers. Record the new contract here in the same change
that edits `src/lib.rs`.

## Add API Documentation

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Document each item's signature, error set, blocking behavior, and required feature
4. Register in `readme.md` Overview Table: add row with ID, Name, Purpose, Status
5. Increment the `api/` instance count in `../entity.md` and add a Master Doc Instances row

## Update API Documentation

1. Edit the target `NNN_*.md` file
2. Reconcile every documented signature against `src/lib.rs` — a stale signature here is worse than none, since consumers treat this file as authoritative
3. If an `AuthError` variant was added or removed, note the semver impact
4. If name or purpose changed: update `readme.md` Overview Table row and `../entity.md`

## Example

Adding API document `002_device_flow_surface`:

1. Check `readme.md` Overview Table — current highest ID is `001`
2. Create `002_device_flow_surface.md` in this directory
3. Document the new items: signature, error set, blocking behavior, required feature
4. Add row: `| 002 | [Device Flow Surface](002_device_flow_surface.md) | Device-authorization grant contract | ✅ |`
5. Bump `api/` instances to 2 in `../entity.md` and add the Master Doc Instances row
