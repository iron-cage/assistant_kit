# Parameter Group :: Aggregation

Interaction tests for the Aggregation group: `by`, `keep`, `dry_run`, `confirm`.
Tests validate `.stats`/`.prune` command scoping and the `dry_run`/`confirm`
precedence rule.

**Source:** [param_group/03_aggregation.md](../../../../docs/cli/param_group/03_aggregation.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| CC-1 | `.stats by::model` -> groups rows by model dimension | Command Scoping |
| CC-2 | `.prune keep::30d dry_run::1 confirm::1` -> `dry_run` overrides `confirm`, no deletion | Precedence |
| CC-3 | `.prune keep::30d confirm::1` (`dry_run::0` default) -> deletes without prompting | Precedence |
| CC-4 | `.prune keep::30d` (both defaults) -> prompts for confirmation before deleting | Default |

## Test Coverage Summary

- Command Scoping: 1 test (CC-1)
- Precedence: 2 tests (CC-2, CC-3)
- Default: 1 test (CC-4)

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

### CC-2: `.prune keep::30d dry_run::1 confirm::1` -> `dry_run` overrides `confirm`

- **Given:** journal directory with files older than 30 days
- **When:** `clj .prune keep::30d dry_run::1 confirm::1`
- **Then:** candidate list is printed; no confirmation prompt appears; no files are deleted, even though `confirm::1` was also set
- **Exit:** 0
- **Source:** [param_group/03_aggregation.md](../../../../docs/cli/param_group/03_aggregation.md)
---

### CC-3: `.prune keep::30d confirm::1` (`dry_run::0` default) -> deletes without prompting

- **Given:** journal directory with files older than 30 days; `dry_run` left at its default (0)
- **When:** `clj .prune keep::30d confirm::1`
- **Then:** matching files are deleted immediately; no confirmation prompt appears
- **Exit:** 0
- **Source:** [param_group/03_aggregation.md](../../../../docs/cli/param_group/03_aggregation.md)
---

### CC-4: `.prune keep::30d` (both defaults) -> prompts for confirmation before deleting

- **Given:** journal directory with files older than 30 days; interactive terminal
- **When:** `clj .prune keep::30d`
- **Then:** a confirmation prompt is shown before any file is deleted
- **Exit:** 0
- **Source:** [param_group/03_aggregation.md](../../../../docs/cli/param_group/03_aggregation.md)
