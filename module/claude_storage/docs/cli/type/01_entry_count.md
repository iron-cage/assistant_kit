# Type :: 1. `EntryCount`

**Purpose:** Non-negative integer representing a minimum session entry threshold. Semantically distinct from general integers — negative values are meaningless for entry counts.

**Fundamental Type:** Wrapper around unsigned integer

**Constants:**
- MIN = 0 (no minimum)
- DEFAULT = unset (no filtering applied)

**Constraints:**
- Range: 0 to i64::MAX
- Negative values rejected: `"min_entries must be ≥ 0, got {value}"`
- Non-integer rejected: `"min_entries must be a non-negative integer, got {value}"`

**Parsing:**
```
Parse string to non-negative integer:
  Input: "0", "10", "100"
  Output: EntryCount(0), EntryCount(10), EntryCount(100)
  Error(negative): "min_entries must be ≥ 0, got {value}"
  Error(non-int): "min_entries must be a non-negative integer, got {value}"

Pseudocode:
  function parse_entry_count(input: string) -> Result<EntryCount>:
    n = parse_int(input)           # error if not integer
    if n < 0:
      return Error("min_entries must be ≥ 0, got " + input)
    return EntryCount(n)
```

**Methods:**
- `get() -> integer` — Raw value accessor
- `is_zero() -> boolean` — True when count is 0 (no minimum)
- `exceeds(actual: integer) -> boolean` — True when actual count is below threshold

**Commands:** `.list`, `.projects`

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 2 | [`.list`](../command/02_list.md) | `min_entries::` |
| 7 | [`.projects`](../command/07_projects.md) | `min_entries::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 7 | [`min_entries::`](../param/07_min_entries.md) | 2 |
