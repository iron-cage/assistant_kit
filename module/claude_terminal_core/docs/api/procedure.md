# API Documentation Operations

- **Actor:** Developer
- **Trigger:** A public item is added, removed, or has its signature or error behavior changed.
- **Emits:** —

## Add API Documentation

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Record each item's signature, its error conditions, and what a caller must guarantee
4. Register in `readme.md` Overview Table: add row with ID, Name, Purpose, Status
5. Register in `../entity.md` Master Doc Instances Table and increment the `api/` row's `Instances` count

## Update API Documentation

1. Edit the target `NNN_*.md` file
2. A changed signature is a breaking change — record it here in the same change that makes it, not afterward
3. If name or purpose changed: update `readme.md` Overview Table row and the `../entity.md` instance row

## Example

Adding API document `002_streaming_surface`:

1. Check `readme.md` Overview Table — current highest ID is `001`
2. Create `002_streaming_surface.md` in this directory
3. Add row: `| 002 | Streaming Surface | Incremental rendering contract for a stateful caller | ✅ |`
4. Add the matching `../entity.md` instance row and bump `api/` Instances from 1 to 2
