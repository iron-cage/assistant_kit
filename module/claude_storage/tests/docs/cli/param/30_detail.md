# Parameter :: `detail::`

Edge case tests for the `detail::` parameter. Tests validate enum enforcement, case-insensitivity, and per-command default behavior (`sessions` on `.projects`, `projects` on `.show`).

**Source:** [param/30_detail.md](../../../../docs/cli/param/30_detail.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | No `detail::` given on `.projects` → full session detail | Default |
| EC-2 | `detail::projects` on `.projects` → terse, project-only view | Happy Path |
| EC-3 | `detail::sessions` on `.projects` → explicit full detail (same as default) | Happy Path |
| EC-4 | `detail::` invalid value → rejected | Type Validation |
| EC-5 | `detail::PROJECTS` (mixed case) → accepted, case-insensitive | Boundary Values |
| EC-6 | No `detail::` given on `.show` → summary + tail only, no session list | Cross-Command Default |
| EC-7 | `detail::sessions` on `.show` → summary + tail + full per-session list | Cross-Command Default |
| EC-8 | `detail::` given with `session_id::` on `.show` → no effect | Non-Interference |

## Test Coverage Summary

- Default: 1 test (EC-1)
- Happy Path: 2 tests (EC-2, EC-3)
- Type Validation: 1 test (EC-4)
- Boundary Values: 1 test (EC-5)
- Cross-Command Default: 2 tests (EC-6, EC-7)
- Non-Interference: 1 test (EC-8)

**Total:** 8 edge cases

**Behavioral Divergence Pair:** EC-1 (`.projects` default `sessions`) ↔ EC-6 (`.show` default `projects`)

Validation edge cases (EC-4, EC-5) are parameter-level (`DetailLevel` type) and shared across consuming commands — not re-tested per command. Only the default-value divergence (EC-1 vs EC-6) and its explicit-override counterpart (EC-3 vs EC-7) are command-specific and re-verified for `.show`.

## Test Cases

---

### EC-1: No `detail::` given on `.projects` → full session detail

- **Commands:** `.projects`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 2 projects, each with sessions)
- **When:** `clg .projects`
- **Then:** Project headers plus full session/family detail beneath each — unchanged pre-consolidation `.projects` behavior (default `sessions`)
- **Exit:** 0
- **Source:** [param/30_detail.md](../../../../docs/cli/param/30_detail.md)

---

### EC-2: `detail::projects` on `.projects` → terse, project-only view

- **Commands:** `.projects`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 2 projects, each with sessions)
- **When:** `clg .projects detail::projects`
- **Then:** One header line per project only; no session or family lines beneath any project
- **Exit:** 0
- **Source:** [param/30_detail.md](../../../../docs/cli/param/30_detail.md)

---

### EC-3: `detail::sessions` on `.projects` → explicit full detail

- **Commands:** `.projects`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 2 projects, each with sessions)
- **When:** `clg .projects detail::sessions`
- **Then:** Identical output to `clg .projects` with no `detail::` given (EC-1) — explicit value matches the default
- **Exit:** 0
- **Source:** [param/30_detail.md](../../../../docs/cli/param/30_detail.md)

---

### EC-4: Invalid value rejected

- **Commands:** `.projects`
- **Given:** clean environment
- **When:** `clg .projects detail::bogus`
- **Then:** Exit 1; error message `detail must be projects|sessions, got bogus`
- **Exit:** 1
- **Source:** [param/30_detail.md](../../../../docs/cli/param/30_detail.md)

---

### EC-5: Mixed-case value accepted

- **Commands:** `.projects`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: 2 projects, each with sessions)
- **When:** `clg .projects detail::PROJECTS`
- **Then:** Identical output to `clg .projects detail::projects` (EC-2) — value parsed case-insensitively
- **Exit:** 0
- **Source:** [param/30_detail.md](../../../../docs/cli/param/30_detail.md)

---

### EC-6: No `detail::` given on `.show` → summary + tail only

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project with 3 sessions, run from its cwd)
- **When:** `clg .show`
- **Then:** Summary block and last `tail::` messages only; no per-session list appended — `.show`'s own default (`projects`), distinct from `.projects`'s default of `sessions` (see EC-1)
- **Exit:** 0
- **Source:** [param/30_detail.md](../../../../docs/cli/param/30_detail.md)

---

### EC-7: `detail::sessions` on `.show` → full per-session list appended

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: project with 3 sessions, run from its cwd)
- **When:** `clg .show detail::sessions`
- **Then:** Summary block and last `tail::` messages (same as EC-6), followed by a full per-session list — one line per session with ID and basic metadata
- **Exit:** 0
- **Source:** [param/30_detail.md](../../../../docs/cli/param/30_detail.md)

---

### EC-8: `detail::` given with `session_id::` on `.show` → no effect

- **Commands:** `.show`
- **Given:** `export CLAUDE_STORAGE_ROOT=/tmp/test-fixture` (fixture: session `-default_topic` with known entries)
- **When:** `clg .show session_id::-default_topic detail::sessions` vs `clg .show session_id::-default_topic`
- **Then:** Identical output in both — session-detail branches ignore `detail::` entirely
- **Exit:** 0
- **Source:** [param/30_detail.md](../../../../docs/cli/param/30_detail.md)
