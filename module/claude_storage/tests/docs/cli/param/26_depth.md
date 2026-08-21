# Parameter :: `depth::`

Edge case tests for the `depth::` parameter. Tests validate the component-distance cap applied to tree-walking `scope::` values, the `0`-means-unbounded boundary, the scopes that ignore `depth::` entirely, and integer validation.

**Two commands, one implementation:** `.usage` and `.rollup` both register `depth::`, reusing `beyond_depth`/`component_distance` from `src/cli/scope.rs` unchanged. `.usage` carries the exhaustive coverage; `.rollup`'s cases are wiring smoke tests confirming the shared code is reached, not a re-derivation (`tests/docs/cli/command/14_rollup.md` states this split).

**Source:** [param/26_depth.md](../../../../docs/cli/param/26_depth.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `depth::1` drops candidates beyond one path component | Boundary Values |
| EC-2 | `depth::0` is unbounded | Boundary Values |
| EC-3 | `scope::global` ignores `depth::` | Scope Interaction |
| EC-4 | Negative `depth::` rejected on `.usage` | Input Validation |
| EC-5 | `depth::` cap reaches `.rollup` | Cross-Command Reuse |
| EC-6 | Negative `depth::` rejected on `.rollup` | Input Validation |

## Test Coverage Summary

- Boundary Values: 2 tests (EC-1, EC-2)
- Scope Interaction: 1 test (EC-3)
- Input Validation: 2 tests (EC-4, EC-6)
- Cross-Command Reuse: 1 test (EC-5)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (`depth::1`, distant candidates dropped) ↔ EC-2 (`depth::0`, every candidate kept)

## Test Cases

---

### EC-1: `depth::1` drops candidates beyond one path component

- **Commands:** `.usage`
- **Given:** projects at varying component distances from the `path::` anchor
- **When:** `clg .usage scope::under depth::1`
- **Then:** only candidates within one path component of the anchor appear; more distant projects are excluded from the table
- **Exit:** 0
- **Covered by:** `cli_cmd_usage_test.rs` — `int_7_depth_caps_component_distance`
- **Source:** [param/26_depth.md](../../../../docs/cli/param/26_depth.md)

---

### EC-2: `depth::0` is unbounded

- **Commands:** `.usage`
- **Given:** the same fixture as EC-1, including a project far enough away that the default `depth::3` would exclude it
- **When:** `clg .usage depth::0` on a tree-walking scope
- **Then:** no depth cap is applied — every candidate the scope reaches is included
- **Exit:** 0
- **Covered by:** `cli_cmd_usage_test.rs` — `int_8_depth_zero_is_unbounded`
- **Source:** [param/26_depth.md](../../../../docs/cli/param/26_depth.md)

---

### EC-3: `scope::global` ignores `depth::`

- **Commands:** `.usage`
- **Given:** a storage whose projects span more than the default depth from any anchor
- **When:** `clg .usage scope::global` with `path::` and `depth::` both supplied
- **Then:** both are ignored — `global` has no anchor to measure distance from, so the whole storage is reported regardless
- **Exit:** 0
- **Covered by:** `cli_cmd_usage_test.rs` — `int_5_global_ignores_path_and_depth`
- **Source:** [param/26_depth.md](../../../../docs/cli/param/26_depth.md)

---

### EC-4: Negative `depth::` rejected on `.usage`

- **Commands:** `.usage`
- **Given:** clean environment
- **When:** `clg .usage depth::-1`
- **Then:** Exit 1; error indicating `depth` must be non-negative
- **Exit:** 1
- **Covered by:** `cli_cmd_usage_test.rs` — `int_20_negative_depth_rejected`
- **Source:** [param/26_depth.md](../../../../docs/cli/param/26_depth.md)

---

### EC-5: `depth::` cap reaches `.rollup`

- **Commands:** `.rollup`
- **Given:** candidates at varying component distances from the anchor
- **When:** `clg .rollup scope::under depth::N`
- **Then:** candidates beyond the component distance are dropped, exactly as on `.usage` — the shared scope machinery is wired through
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_12_depth_caps_component_distance_smoke`
- **Source:** [param/26_depth.md](../../../../docs/cli/param/26_depth.md)

---

### EC-6: Negative `depth::` rejected on `.rollup`

- **Commands:** `.rollup`
- **Given:** clean environment
- **When:** `clg .rollup depth::-1`
- **Then:** Exit 1; error indicating `depth` must be non-negative — validation is not skipped on the reusing command
- **Exit:** 1
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_20_negative_depth_rejected`
- **Source:** [param/26_depth.md](../../../../docs/cli/param/26_depth.md)
