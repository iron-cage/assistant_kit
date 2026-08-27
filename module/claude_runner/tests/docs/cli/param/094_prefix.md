# Test: `--prefix` (pool name prefix)

Integration test planning for the `--prefix` parameter. See
[param/094_prefix.md](../../../../docs/cli/param/094_prefix.md) for the parameter specification.

`--prefix` names the pool [`pool`](../command/18_pool.md) fills, and therefore decides which
existing topics count toward [`--count`](093_count.md). The CLI's job is to pass it to
`claude_topic_core::pool::validate_prefix` and report a rejection with its reason; the rules
themselves are properties of the `name ↔ index` mapping and are unit-tested where they live.

The rules exist because a pool name is exactly `format!( "{prefix}{index}" )` and the reverse
direction has to be unambiguous or the count is unreliable. Five shapes are rejected, and the
non-obvious one is the digit-trailing rule: with prefix `t1`, index 2 gives `t12`, which reads
equally as prefix `t1` + index 2 and prefix `t` + index 12. That ambiguity is why the default
prefix is `t` rather than something like `t1_`.

Cases are owned by [command/18_pool.md](../command/18_pool.md) (PL-1..PL-18) and implemented in
`tests/pool_command_test.rs`; this file maps the parameter to them.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| PL-8 | `--prefix worker` counts against `worker*` alone, ignoring `t1`/`t2` | Independence |
| PL-9 | A digit-trailing prefix exits 1, naming the ambiguity | Guards |
| PL-10 | An empty prefix and one containing `/` both exit 1 | Guards |
| PL-7 | `t01` is not a pool name — a leading zero does not round-trip through `format!` | Round-trip |
| PL-1 | The default prefix is `t` — an empty base plans `t1`, `t2`, … | Default |

## Test Coverage Summary

- Independence: 1 test (PL-8)
- Guards: 2 tests (PL-9, PL-10)
- Round-trip: 1 test (PL-7)
- Default: 1 test (PL-1)

**Total:** 5 cases, all shared with [command/18_pool.md](../command/18_pool.md)

**Implemented by:** `tests/pool_command_test.rs::pl01`, `pl07`–`pl10`

**Related:** `claude_topic_core/tests/pool_test.rs` — `validate_prefix()`'s five rejection
shapes and `pool_index()`'s exact-inverse property, unit-tested without a CLI.

---

### PL-8: A prefix names an independent pool

- **Given:** a base already holding live `t1` and `t2`
- **Command:** `clr pool --dry-run --prefix worker --count 2`
- **Expected behavior:** `existing: 0` and two `cmd:` lines naming `worker1` and `worker2` — two
  independent pools over one base, four topics in total. Counting `t1`/`t2` toward a `worker`
  target would make `--count`'s meaning depend on unrelated work that happens to share the
  directory
- **Exit:** 0
- **Source:** [param/094_prefix.md](../../../../docs/cli/param/094_prefix.md)

---

### PL-9: A digit-trailing prefix is rejected with its reason

- **Command:** `clr pool --dry-run --prefix t1 --count 2`
- **Expected behavior:** exit 1, stderr quoting `t1` and stating the ambiguity. Accepting it
  would make `pool_index` many-to-one, and the first symptom would not be an error — it would be
  a count that silently disagrees with what `clr topics` shows
- **Exit:** 1
- **Source:** [param/094_prefix.md](../../../../docs/cli/param/094_prefix.md)

---

### PL-10: Structurally impossible prefixes are rejected

- **Commands:** `clr pool --dry-run --prefix "" --count 1`; `clr pool --dry-run --prefix a/b --count 1`
- **Expected behavior:** both exit 1. An empty prefix makes every name a bare number; a prefix
  containing `/` makes the name a path, and a topic name is a single path component. Both are
  rejected before the base is read — a name that cannot exist should never reach a `read_dir`
- **Exit:** 1 both
- **Source:** [param/094_prefix.md](../../../../docs/cli/param/094_prefix.md)

---

### PL-7: A leading-zero name is not a pool name

- **Given:** a base holding a live topic literally named `t01`, alongside unrelated names
  (`auth-refactor` in fork mode, `bench` in dir mode)
- **Command:** `clr pool --dry-run --count 2`
- **Expected behavior:** `existing: 0` and two `cmd:` lines naming `t1` and `t2`.
  `format!( "t{}", 1 )` produces `t1`, never `t01`, so `t01` does not round-trip and is neither
  counted nor generated. Admitting it would put two names in one slot and make the count
  ambiguous
- **Exit:** 0
- **Source:** [param/094_prefix.md](../../../../docs/cli/param/094_prefix.md)
