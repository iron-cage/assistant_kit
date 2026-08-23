# tests/storage_fixture/

Shared fixture module for the `export.rs`, `search.rs`, and `filtering.rs`
integration binaries. Lives in a subdirectory so cargo does not auto-discover it as
a test binary of its own.

Each consumer declares `mod storage_fixture;` and builds its own `TempDir` storage
tree, so no test in those three binaries reads the developer's real `~/.claude/`
directory and none of them can be skipped by an empty-storage guard.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `mod.rs` | Temp storage trees and JSONL entry lines shared by storage test binaries |
