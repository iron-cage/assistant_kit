# Parameter :: `columns::`

Edge case tests for the `columns::` parameter. Tests validate the display projection over `.rollup`'s 14 column keys — which print, in what order — the five keys the default set omits, the `rank` column's post-`limit::` numbering, the `cache_write`/`cache_read` split, and unknown-key validation.

**Projection is display-only:** every column but `rank` is computed by `claude_storage_core::rollup::build_rollup()` regardless of what is projected, so these cases pin what is *printed*, never what is computed. `rank` is the exception — synthesized by the CLI from each row's final rendered position, which is why EC-6 exists.

**Source:** [param/38_columns.md](../../../../docs/cli/param/38_columns.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Custom subset projects only the chosen columns | Happy Path |
| EC-2 | Default set omits `first`/`last` | Default |
| EC-3 | Default set omits `rank`/`cache_write`/`cache_read` | Default |
| EC-4 | `columns::first,last` renders raw ISO-8601 timestamps | Happy Path |
| EC-5 | `columns::rank` numbers rows by sorted position | Happy Path |
| EC-6 | `rank` reflects position after `limit::` truncates | Boundary Values |
| EC-7 | `cache_write` + `cache_read` sums to `cache` | Happy Path |
| EC-8 | Unknown column key rejected | Input Validation |
| EC-9 | `columns::` composes with the other `.rollup` parameters | Composition |

## Test Coverage Summary

- Happy Path: 4 tests (EC-1, EC-4, EC-5, EC-7)
- Default: 2 tests (EC-2, EC-3)
- Boundary Values: 1 test (EC-6)
- Input Validation: 1 test (EC-8)
- Composition: 1 test (EC-9)

**Total:** 9 edge cases

**Behavioral Divergence Pair:** EC-2/EC-3 (default projection — five keys absent) ↔ EC-4/EC-5/EC-7 (those same keys requested explicitly and printed)

## Test Cases

---

### EC-1: Custom subset projects only the chosen columns

- **Commands:** `.rollup`
- **Given:** a populated storage
- **When:** `clg .rollup columns::group,total,calls`
- **Then:** exactly the named columns print, left-to-right in the order given; no default-set column leaks through
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_7_columns_custom_subset_projects_only_those`
- **Source:** [param/38_columns.md](../../../../docs/cli/param/38_columns.md)

---

### EC-2: Default set omits `first`/`last`

- **Commands:** `.rollup`
- **Given:** sessions carrying timestamps
- **When:** `clg .rollup` with no `columns::`
- **Then:** the two verbose ISO-8601 timestamp columns are absent — they are opt-in only, even though the underlying values are computed
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_8_columns_default_excludes_first_last`
- **Source:** [param/38_columns.md](../../../../docs/cli/param/38_columns.md)

---

### EC-3: Default set omits `rank`/`cache_write`/`cache_read`

- **Commands:** `.rollup`
- **Given:** sessions with both cache-write and cache-read token activity
- **When:** `clg .rollup` with no `columns::`
- **Then:** neither the `Rank` column nor the split `CacheW`/`CacheR` columns appear — the default set carries the combined `Cache` column only
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_28_columns_default_excludes_rank_and_cache_split`
- **Source:** [param/38_columns.md](../../../../docs/cli/param/38_columns.md)

---

### EC-4: `columns::first,last` renders raw ISO-8601 timestamps

- **Commands:** `.rollup`
- **Given:** sessions with recorded first and last timestamps
- **When:** `clg .rollup columns::group,total,first,last`
- **Then:** the two timestamp columns print their raw ISO-8601 values, surfacing the span each row covers
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_24_columns_first_last_render_timestamps`
- **Source:** [param/38_columns.md](../../../../docs/cli/param/38_columns.md)

---

### EC-5: `columns::rank` numbers rows by sorted position

- **Commands:** `.rollup`
- **Given:** several rows with distinct sort-key values
- **When:** `clg .rollup columns::rank,group,total`
- **Then:** each row carries its 1-indexed position after `sort::`/`order::` have applied
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_25_columns_rank_numbers_rows_by_sorted_position`
- **Source:** [param/38_columns.md](../../../../docs/cli/param/38_columns.md)

---

### EC-6: `rank` reflects position after `limit::` truncates

- **Commands:** `.rollup`
- **Given:** more rows than the requested limit
- **When:** `clg .rollup columns::rank,group,total limit::N`
- **Then:** rank numbers the *rendered* rows — it is synthesized from final position, so truncation cannot leave a gap or an out-of-range number
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_26_rank_reflects_post_limit_position`
- **Source:** [param/38_columns.md](../../../../docs/cli/param/38_columns.md)

---

### EC-7: `cache_write` + `cache_read` sums to `cache`

- **Commands:** `.rollup`
- **Given:** sessions with both cache-creation and cache-read tokens
- **When:** `clg .rollup columns::group,cache,cache_write,cache_read`
- **Then:** the two split columns print separately and their sum equals the combined `Cache` column on the same row
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_27_columns_cache_write_cache_read_split_sums_to_cache`
- **Source:** [param/38_columns.md](../../../../docs/cli/param/38_columns.md)

---

### EC-8: Unknown column key rejected

- **Commands:** `.rollup`
- **Given:** clean environment
- **When:** `clg .rollup columns::group,bogus`
- **Then:** Exit 1; error names the offending key and lists all 14 valid ones — one bad entry rejects the whole list, it is not silently skipped
- **Exit:** 1
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_19_invalid_columns_rejected`
- **Source:** [param/38_columns.md](../../../../docs/cli/param/38_columns.md)

---

### EC-9: `columns::` composes with the other `.rollup` parameters

- **Commands:** `.rollup`
- **Given:** three models with distinct session counts
- **When:** `clg .rollup group::model sort::sessions order::asc columns::group,sessions limit::1`
- **Then:** the projection hides every field except the two requested — no other column label leaks through, despite `group::`, `sort::`, `order::` and `limit::` all being active in the same invocation
- **Exit:** 0
- **Covered by:** `cli_cmd_rollup_test.rs` — `int_22_multiple_parameters_compose_correctly_together`
- **Source:** [param/38_columns.md](../../../../docs/cli/param/38_columns.md)
