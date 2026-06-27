# User Story: Failure Diagnosis

**As a** developer debugging CLR automation failures,
**I want to** quickly find and examine failed invocations,
**so that** I can identify error patterns and fix root causes.

### Persona

Developer investigating why automation scripts fail intermittently.

### Primary Commands

| # | Command | Role in Story |
|---|---------|---------------|
| 1 | [`.list`](../command/01_list.md) | Filter by exit code to find failures |
| 2 | [`.tail`](../command/02_tail.md) | Follow events in real-time during debugging |
| 4 | [`.search`](../command/04_search.md) | Full-text search for error messages |

### Acceptance Criteria

| # | Criterion |
|---|-----------|
| AC-01 | `clj .list exit::2 since::1d` shows all rate-limit failures in last day |
| AC-02 | `clj .list exit::4 since::1d` shows all timeout failures in last day |
| AC-03 | `clj .search pattern::"rate limit" since::7d` finds rate limit messages |
| AC-04 | `clj .search pattern::"error" include_stdout::1` searches subprocess output |
| AC-05 | `clj .tail type::retry` follows retry events in real-time |
| AC-06 | Event output includes enough context to diagnose: exit code, error class, duration, model |
| AC-07 | When journal level is `full`, stdout/stderr content is available for inspection |

### Workflow

```bash
# Quick: what failed today?
clj .list exit::1 since::1d

# Deep: search for specific error in output
clj .search pattern::"connection refused" include_stdout::1 since::7d

# Live: monitor retries during debugging
clj .tail type::retry

# Analyze: error class distribution
clj .stats by::error since::7d
```

### Referenced Parameters

| # | Parameter | Usage |
|---|-----------|-------|
| 01 | [`since`](../param/01_since.md) | Narrow time window |
| 03 | [`type`](../param/03_type.md) | Filter to retry/timeout events |
| 05 | [`exit`](../param/05_exit.md) | Filter by exit code |
| 14 | [`pattern`](../param/14_pattern.md) | Search for error messages |
| 28 | [`include_stdout`](../param/28_include_stdout.md) | Search subprocess output |
