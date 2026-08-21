# Parameter Group :: Aggregation

Interaction tests for the Aggregation group: `by`, `keep`, `dry_run`.
Tests validate `.stats`/`.prune` command scoping and the `dry_run` preview
rule. (Historical: a `confirm` param was dropped — `.prune` deletes without
prompting; `dry_run::1` is the only preview mechanism. See
[param/readme.md](../../../../docs/cli/param/readme.md), numbering gap 20.)

**Source:** [param_group/03_aggregation.md](../../../../docs/cli/param_group/03_aggregation.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `.stats by::model` -> groups rows by model dimension | Command Scoping |
| CC-2 | `.prune keep::30d dry_run::1` -> previews candidate list, deletes nothing | Preview |
| CC-3 | `.prune keep::30d` (`dry_run::0` default) -> deletes immediately, no confirmation prompt | Default |
| CC-4 | `.prune keep::30d` filename-date semantics -> today's file never deleted | Boundary |

## Test Coverage Summary

- Command Scoping: 1 test (CC-1)
- Preview: 1 test (CC-2)
- Default: 1 test (CC-3)
- Boundary: 1 test (CC-4)

**Total:** 4 corner cases

## Test Cases
---

### CC-1: `.stats by::model` -> groups rows by model dimension

- **Given:** journal with events across multiple models
- **When:** `clj .stats by::model`
- **Then:** output contains one row per distinct model; `by` has no effect on `.prune`
- **Exit:** 0
- **Source:** [param_group/03_aggregation.md](../../../../docs/cli/param_group/03_aggregation.md)
---

### CC-2: `.prune keep::30d dry_run::1` -> previews candidate list, deletes nothing

- **Given:** journal directory with files older than 30 days
- **When:** `clj .prune keep::30d dry_run::1`
- **Then:** candidate list is printed; no files are deleted
- **Exit:** 0
- **Automated in:** `viewer_integration_test.rs::ec6_prune_dry_run_lists_without_deleting`
- **Source:** [param_group/03_aggregation.md](../../../../docs/cli/param_group/03_aggregation.md)
---

### CC-3: `.prune keep::30d` (`dry_run::0` default) -> deletes immediately, no confirmation prompt

- **Given:** journal directory with files older than 30 days; `dry_run` left at its default (0)
- **When:** `clj .prune keep::30d`
- **Then:** matching files are deleted immediately; no confirmation prompt appears (feature 001 AC-007 — the historical `confirm` param was dropped)
- **Exit:** 0
- **Source:** [param_group/03_aggregation.md](../../../../docs/cli/param_group/03_aggregation.md)
---

### CC-4: `.prune keep::30d` filename-date semantics -> today's file never deleted

- **Given:** journal directory containing today's `YYYY-MM-DD.jsonl` and files with filename dates older than 30 days
- **When:** `clj .prune keep::30d`
- **Then:** deletion is decided by the filename date, and today's file is never deleted regardless of `keep::` window
- **Exit:** 0
- **Automated in:** `viewer_integration_test.rs::ec20_prune_filename_date_semantics`
- **Source:** [param_group/03_aggregation.md](../../../../docs/cli/param_group/03_aggregation.md)
