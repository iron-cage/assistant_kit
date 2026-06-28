# 004_performance

Test spec for `docs/invariant/004_performance.md`.

## Overview

| Case ID | Category | Status |
|---------|----------|--------|
| PM-1 | Fast-path source contains no JSONL read calls | ⏳ |
| PM-2 | count_entries uses byte search, not JSON deserialization | ⏳ |

## Cases

### PM-1: Fast-path status source does not open or read JSONL content

- **Given:** Source files implementing the fast-path code path for `.status v::0` and `.status v::1` in `module/claude_storage_core/src/` and `module/claude_storage/src/`
- **When:** The fast-path implementation is scanned for JSONL content-reading API calls: `read_to_string`, `BufReader`, `serde_json::from_str`, `serde_json::from_slice`, `from_reader`, `read_to_end`
- **Then:** None of these patterns appear in the fast-path code paths; only filesystem metadata operations (`read_dir`, `DirEntry`, `metadata`, filename inspection) are present — confirming O(P+S) cost and no JSONL byte reads at verbosity 0/1

### PM-2: count_entries implementation uses byte-level pattern search

- **Given:** Source file implementing `count_entries()` or the equivalent entry-counting logic in `module/claude_storage_core/src/`
- **When:** The implementation is inspected for the search patterns it uses to count message entries
- **Then:** It searches for `"type":"user"` and `"type":"assistant"` as byte or string substrings (not via `serde_json` deserialization of individual lines); no `serde_json::from_str` or full-struct deserialization appears in the entry-counting code path — confirming the documented byte-level cost model
