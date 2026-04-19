# Algorithm: Settings Type Inference

### Scope

- **Purpose**: Document the type inference rules applied to the `value::` parameter in `.settings.set`.
- **Responsibility**: Specify the 4-step inference order, precedence rules, and edge cases (NaN, infinity, large integers).
- **In Scope**: bool inference, i64 inference, f64 inference, string fallback, NaN/infinity handling.
- **Out of Scope**: Settings JSON read/write mechanics (→ `feature/003_settings_management.md`), CLI parameter validation (→ `feature/005_cli_design.md`).

### Abstract

When `.settings.set key::K value::V` is called, the string value `V` must be mapped to a JSON type. This algorithm determines whether `V` should become a JSON boolean, integer, float, or string.

### Algorithm

The inference is applied in strict priority order. The first matching rule wins:

**Step 1 — Boolean check:**
- If `V` is exactly `"true"` → JSON `true`
- If `V` is exactly `"false"` → JSON `false`
- Otherwise → proceed to Step 2

**Step 2 — Integer check:**
- If `V` is parseable as `i64` (including `"0"` and `"1"`) → JSON integer
- Note: `"0"` and `"1"` match here, not as booleans (Step 1 handles only `"true"`/`"false"`)
- Otherwise → proceed to Step 3

**Step 3 — Float check:**
- If `V` is parseable as finite `f64` (but not as `i64`) → JSON float
- If `V` parses as `f64` but is NaN or infinity (including `"nan"`, `"inf"`, `"infinity"`, `"-inf"`) → JSON string (not a float)
- Otherwise → proceed to Step 4

**Step 4 — String fallback:**
- All other values → JSON string (with `\` and `"` escaped in the JSON output)

**Precedence summary:** boolean > integer > finite float > string (NaN/inf fall through to string)

**Examples:**

| Input | JSON Output | Rule |
|-------|-------------|------|
| `"true"` | `true` | Step 1 |
| `"false"` | `false` | Step 1 |
| `"0"` | `0` | Step 2 |
| `"1"` | `1` | Step 2 |
| `"42"` | `42` | Step 2 |
| `"-9"` | `-9` | Step 2 |
| `"3.14"` | `3.14` | Step 3 |
| `"nan"` | `"nan"` | Step 3 → string |
| `"inf"` | `"inf"` | Step 3 → string |
| `"hello"` | `"hello"` | Step 4 |
| `"it's"` | `"it's"` | Step 4 |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| doc | [feature/003_settings_management.md](../feature/003_settings_management.md) | .settings.set that applies this inference |
| source | `../../src/settings_io.rs` | Type inference implementation |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | FR-07 (type inference for value::) |
