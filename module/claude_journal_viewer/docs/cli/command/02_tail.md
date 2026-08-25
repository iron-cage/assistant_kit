# .tail

Follow journal events in real-time.

-- **Parameters:** until::, type::, command::, exit::, model::, dir::, creds::, format::, no_color::, journal_dir::
-- **Exit Codes:** 0 (interrupted), 1 (invalid or unknown param)

### Syntax

```
clj .tail [until::DURATION] [type::EVENT_TYPE] [command::CMD] [exit::CODE]
          [model::NAME] [dir::SUBSTR] [creds::NAME]
          [format::FORMAT] [no_color::BOOL] [journal_dir::PATH]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `type` | EventType | -- | No | Filter by event type |
| `command` | String | -- | No | Filter by clr command |
| `format` | OutputFormat | table | No | Output format |
| `no_color` | Boolean | 0 | No | Disable ANSI colors |
| `journal_dir` | Path | ~/.clr/journal/ | No | Journal directory override |

`.tail` builds the same `JournalFilter` as `.list`, so it accepts most of the
filter vocabulary — `until`, `exit`, `model`, `dir`, `creds` — not just the two
listed above by name.

**`since` and `limit` are the two exceptions, and are rejected.** `.tail` follows
the journal forward from the moment it starts, so there is no earlier event for
`since::` to exclude and no end for `limit::` to stop at: `TailIter` passes
neither to the matcher. Both used to be accepted and applied to nothing, which is
worse than refusing them — a cap that silently does not cap reads as "there were
only that many". `until` is genuinely applied, but note what it does here: past
the bound the follow simply stops emitting, it does not exit.

```bash
clj .tail limit::5;  echo "exit=$?"   # exit=1, with the accepted list on stderr
clj .tail since::1h; echo "exit=$?"   # exit=1 — use `clj .list since::1h` for history
```

**`format::json` and `format::jsonl` are identical here.** A JSON *array* has no
valid streaming form: `.tail` does not end, so the array's closing bracket would
never be written and a consumer piping to `jq` would block forever waiting for
it. Both names therefore emit one complete JSON object per line.

`format::csv` prints the header row once, before the first event. `format::table`
deliberately prints no header at all — `.tail` output is open-ended, so a header
written once scrolls away and then misleads for the rest of the session.

The format name is validated before the follow loop starts, so an invalid one
exits 1 immediately rather than when the first event happens to arrive — which
on a quiet journal could be never.

**Algorithm (4 steps):**

1. Open `JournalReader` at configured journal dir with filter from params
2. Parse `format::`, exiting 1 on an unknown name; print its header line if it has one
3. Call `JournalReader::tail()` which polls for new events at ~500ms intervals
4. For each new event, render one line in the selected format and flush stdout

### Examples

```bash
clj .tail                          # Follow all events
clj .tail type::execution         # Follow execution events only
clj .tail command::ask format::json  # Follow ask events as JSON lines
clj .tail format::csv > live.csv  # Append a live CSV, header row included
```

### Referenced User Stories

| # | User Story |
|---|-----------|
| 2 | [Failure Diagnosis](../user_story/002_failure_diagnosis.md) |
| 3 | [Automation Audit](../user_story/003_automation_audit.md) |
