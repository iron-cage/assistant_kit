# Algorithm Documentation Operations

- **Actor:** Developer
- **Trigger:** A new algorithm or data processing approach is documented or an existing one changes.
- **Emits:** —

## Add Algorithm Documentation

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Create `NNN_{snake_case_name}.md` in this directory
3. Register in `readme.md` Overview Table: add row with ID, Name, File link, Status

## Update Algorithm Documentation

1. Edit the target `NNN_*.md` file
2. If name or purpose changed: update `readme.md` Overview Table row

## Example

Adding algorithm document `002_cache_eviction`:

1. Check `readme.md` Overview Table — current highest ID is `001`
2. Create `002_cache_eviction.md` in this directory
3. Add row: `| 002 | Cache Eviction | [002_cache_eviction.md](002_cache_eviction.md) | Active |`
