# CLI Test Planning Operations

- **Actor:** Developer
- **Trigger:** A new test tier is added or test planning scope changes.
- **Emits:** —

## Add Test Tier

1. Create sub-entity directory with `readme.md` and `procedure.md`
2. Register in this directory's `readme.md` Responsibility Table
3. Add Navigation entry in `readme.md`

## Update Test Tier

1. Edit the target sub-entity's `readme.md`
2. If responsibility changed: update this directory's `readme.md` Responsibility Table row

## Example

Adding test tier `type/` for per-type edge case tests:

1. Create `type/readme.md` with Scope and Responsibility Table
2. Create `type/procedure.md` with Add/Update operations
3. Add row to `readme.md` Responsibility Table: `| type/ | Per-type edge case test indices |`
4. Add Navigation entry: `- [Type Tests](type/) — Edge case tests per type`
