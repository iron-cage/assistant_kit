# Test: Capacity Planning

### Scope

- **Purpose**: US- acceptance tests verifying developers can understand usage volume and manage journal storage retention.
- **Responsibility**: Acceptance criteria coverage for the capacity planning workflow.
- **In Scope**: Journal health/size status, per-file size breakdown, volume trend and peak detection, prune preview and execution, size-based retention.
- **Out of Scope**: Cost analysis (-> `001_cost_tracking.md`), team-wide reporting (-> `005_team_reporting.md`).

Test case planning for [user_story/004_capacity_planning.md](../../../../docs/cli/user_story/004_capacity_planning.md).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| US-1 | `.status` shows file count, total size, and date range | Journal Health |
| US-2 | `.status verbosity::2` shows per-file size breakdown | Journal Health |
| US-3 | `.stats by::day since::30d` shows daily invocation volume trend | Volume Trend |
| US-4 | `.stats by::hour since::1d` shows hourly distribution (peak detection) | Volume Trend |
| US-5 | `.prune keep::30d dry_run::1` previews what would be pruned | Retention |
| US-6 | `.prune keep::100mb` maintains journal under 100MB | Retention |
| US-7 | `.prune keep::7d confirm::1` prunes without confirmation | Retention |

## Test Coverage Summary

- Journal Health: 2 tests (US-1, US-2)
- Volume Trend: 2 tests (US-3, US-4)
- Retention: 3 tests (US-5, US-6, US-7)

**Total:** 7 tests

---

### US-1: `.status` shows file count, total size, and date range

- **Given:** journal dir with several `.jsonl` files spanning multiple dates
- **When:** `clj .status`
- **Then:** exit 0; output shows file count, total size, and oldest/newest date
- **Exit:** 0
- **Source:** [user_story/004_capacity_planning.md](../../../../docs/cli/user_story/004_capacity_planning.md) AC-01

---

### US-2: `.status verbosity::2` shows per-file size breakdown

- **Given:** journal dir with several `.jsonl` files of varying sizes
- **When:** `clj .status verbosity::2`
- **Then:** exit 0; output lists each file with its individual size, in addition to the summary shown at default verbosity
- **Exit:** 0
- **Source:** [user_story/004_capacity_planning.md](../../../../docs/cli/user_story/004_capacity_planning.md) AC-02

---

### US-3: `.stats by::day since::30d` shows daily invocation volume trend

- **Given:** journal with events spread across the last 30 days
- **When:** `clj .stats by::day since::30d`
- **Then:** exit 0; output contains one row per day showing that day's invocation count
- **Exit:** 0
- **Source:** [user_story/004_capacity_planning.md](../../../../docs/cli/user_story/004_capacity_planning.md) AC-03

---

### US-4: `.stats by::hour since::1d` shows hourly distribution (peak detection)

- **Given:** journal with events spread across the last day, with a concentration in specific hours
- **When:** `clj .stats by::hour since::1d`
- **Then:** exit 0; output contains one row per hour, making peak hours identifiable by count
- **Exit:** 0
- **Source:** [user_story/004_capacity_planning.md](../../../../docs/cli/user_story/004_capacity_planning.md) AC-04

---

### US-5: `.prune keep::30d dry_run::1` previews what would be pruned

- **Given:** journal dir with files older and newer than 30 days
- **When:** `clj .prune keep::30d dry_run::1`
- **Then:** exit 0; output lists files that would be deleted; no files are actually deleted
- **Exit:** 0
- **Source:** [user_story/004_capacity_planning.md](../../../../docs/cli/user_story/004_capacity_planning.md) AC-05

---

### US-6: `.prune keep::100mb` maintains journal under 100MB

- **Given:** journal dir whose total size exceeds 100MB
- **When:** `clj .prune keep::100mb`
- **Then:** exit 0; oldest files are removed until total journal size is at or under 100MB
- **Exit:** 0
- **Source:** [user_story/004_capacity_planning.md](../../../../docs/cli/user_story/004_capacity_planning.md) AC-06

---

### US-7: `.prune keep::7d confirm::1` prunes without confirmation

- **Given:** journal dir with files older than 7 days
- **When:** `clj .prune keep::7d confirm::1`
- **Then:** exit 0; files older than 7 days are deleted immediately, with no interactive confirmation prompt
- **Exit:** 0
- **Source:** [user_story/004_capacity_planning.md](../../../../docs/cli/user_story/004_capacity_planning.md) AC-07
