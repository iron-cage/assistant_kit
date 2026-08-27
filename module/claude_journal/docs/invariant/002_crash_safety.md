# Invariant: Crash Safety

### Scope

- **Purpose**: Define the durability guarantee provided when a process crash or power failure interrupts `JournalWriter::append()`.
- **Responsibility**: State the at-most-one-corrupt-line guarantee, explain the write protocol that delivers it, and specify the reader's recovery behavior.
- **In Scope**: Crash impact boundary (at most 1 trailing line), append-mode open contract, reader skip-on-parse-failure behavior.
- **Out of Scope**: Append-only constraint (→ `invariant/001_append_only.md`), schema versioning (→ `invariant/003_schema_version.md`).

### Invariant Statement

A process crash or power failure during `JournalWriter::append()` corrupts at most one trailing line in the daily journal file. The reader (`JournalReader`) skips lines that fail JSON parse, treating them as incomplete writes. This is documented behavior, not a bug.

The safety guarantee derives from the write protocol: each `append()` call writes a single line terminated by `\n`. If the process dies mid-write, the partial line lacks a valid JSON closing brace and will fail `serde_json::from_str` on read. All preceding complete lines remain intact because the file was opened in append mode (no seek/truncate).

### Measurement

- **Threshold**: At most 1 corrupted line per crash event
- **Method**: Integration test `it5_corrupt_lines_are_skipped` in `tests/journal_integration_test.rs` — write 2 valid events with a corrupt line between them, read back and assert exactly the 2 valid events parse successfully

### Violation Consequences

- A write protocol that batches multiple lines per `append()` call risks corrupting more than one event per crash
- Opening without `O_APPEND` allows a crash to corrupt previously-written lines (not just the in-flight one)
- A reader that halts on parse failure (rather than skipping) breaks all reads after a single crash event

### Sources

| File | Relationship |
|------|--------------|
| `src/writer.rs` | Append protocol — single-line JSON + `\n` |
| `src/reader.rs` | Skip-on-parse-failure recovery behavior |
