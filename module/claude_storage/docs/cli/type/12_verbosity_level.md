# Type :: 12. `VerbosityLevel`

**Purpose:** Output detail level controlling information density across all read commands. Provides consistent semantics: `0` is machine-readable, `1` is the standard human-readable default, higher levels add progressively more detail.

**Fundamental Type:** Wrapper around integer (0-5 range)

**Constants:**
- SILENT = 0 (machine-readable / minimal)
- NORMAL = 1 (standard summary — DEFAULT)
- DETAILED = 2 (extended with counts and metadata)
- VERBOSE = 3 (all fields)
- DEFAULT = NORMAL (1)
- MIN = 0
- MAX = 5

**Constraints:**
- Range: 0-5 inclusive
- Error on out-of-range: `"verbosity must be 0-5, got {value}"`
- Error on non-integer: `"verbosity must be an integer 0-5, got {value}"`

**Parsing:**
```
Parse string to integer, validate range:
  Input: "0" → VerbosityLevel(0)
  Input: "1" → VerbosityLevel(1)
  Input: "5" → VerbosityLevel(5)
  Error(out-of-range): "verbosity must be 0-5, got {value}"
  Error(non-int): "verbosity must be an integer 0-5, got {value}"

Pseudocode:
  function parse_verbosity(input: string) -> Result<VerbosityLevel>:
    n = parse_int(input)          # error if not integer
    if n < 0 or n > 5:
      return Error("verbosity must be 0-5, got " + input)
    return VerbosityLevel(n)
```

**Methods:**
- `get() -> integer` — Raw level value
- `is_silent() -> boolean` — True when level is 0
- `is_normal() -> boolean` — True when level is 1 (default)
- `is_detailed() -> boolean` — True when level is 2
- `is_verbose() -> boolean` — True when level ≥ 3
- `default() -> VerbosityLevel` — Returns VerbosityLevel(1)

**Commands:** `.status`, `.list`, `.show`, `.search`, `.projects`

**Usage:**
```
if verbosity.is_verbose():
  print_all_fields()
elif verbosity.is_detailed():
  print_extended_summary()
elif verbosity.is_normal():
  print_standard_summary()
# silent: print minimal output
```

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|---------------|
| 1 | [`.status`](../command/01_status.md) | `verbosity::` |
| 2 | [`.list`](../command/02_list.md) | `verbosity::` |
| 3 | [`.show`](../command/03_show.md) | `verbosity::` |
| 5 | [`.search`](../command/05_search.md) | `verbosity::` |
| 7 | [`.projects`](../command/07_projects.md) | `verbosity::` |

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 19 | [`verbosity::`](../param/19_verbosity.md) | 5 |
