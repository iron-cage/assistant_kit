# Dictionary

### Scope

- **Purpose**: Domain vocabulary for the `clj` CLI.
- **Responsibility**: Term definitions for events, filters, sorting, and architecture concepts.
- **In Scope**: All domain terms used across `clj` documentation.
- **Out of Scope**: Command reference (→ `command/readme.md`), parameter reference (→ `param/readme.md`).

### Terms

| Term | Definition | Example |
|------|------------|---------|
| Event | A single journaled action recorded by `clr` as one JSONL line | `{"v":1,"type":"execution","ts":"2026-06-27T10:00:00Z",...}` |
| Event Type | One of 8 canonical categories of journal events | `execution`, `retry`, `timeout` |
| Journal | The collection of JSONL files in the journal directory | `~/.clr/journal/*.jsonl` |
| Journal Directory | Filesystem directory containing all journal files | `~/.clr/journal/` |
| Journal File | A single daily JSONL file named by date | `2026-06-27.jsonl` |
| Journal Level | Recording detail level set in `clr` | `full` (default), `meta`, `off` |
| Duration | Human-friendly time offset string | `1h`, `7d`, `4w`, `3M` |
| Retention Spec | Age or size threshold for pruning | `30d`, `100mb`, `1gb` |
| Filter | A constraint that excludes non-matching events | `type::execution since::1d` |
| AND Combination | Multiple filters require ALL conditions to match | type AND command AND since |
| Sort Field | Event field used as sort key for listing | `time`, `cost`, `duration` |
| Group By | Dimension for bucketing events in stats | `day`, `model`, `error` |
| Output Format | Serialization format for rendered events | `table`, `json`, `jsonl`, `csv` |
| Truncation Cap | Maximum size for stdout/stderr at `full` level | 1 MB per field |
| Schema Version | Forward-compatible version field in events | `"v": 1` |
| Crash Safety | At most 1 corrupted line per crash guarantee | Append-only write pattern |
| Error Class | CLR error classification from exit code | `RateLimit`, `Auth`, `Timeout` |
| Super-app | The `ast` aggregator binary | `ast .journal.list since::1d` |
| Pruning | Deleting old journal files by age or size | `clj .prune keep::30d` |
| Web Viewer | Embedded HTTP dashboard for journal browsing | `clj .serve` on port 8411 |

### Provenance

| File | Notes |
|------|-------|
| [../dictionary.md](../dictionary.md) | Original un-migrated source; retained as reference |
