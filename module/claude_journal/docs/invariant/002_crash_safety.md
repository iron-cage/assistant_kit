# Crash Safety

**Status**: Planned | **Since**: 1.3.0

## Description

A process crash or power failure during `JournalWriter::append()` corrupts at most one trailing line in the daily journal file. The reader (`JournalReader`) skips lines that fail JSON parse, treating them as incomplete writes. This is documented behavior, not a bug.

The safety guarantee derives from the write protocol: each `append()` call writes a single line terminated by `\n`. If the process dies mid-write, the partial line lacks a valid JSON closing brace and will fail `serde_json::from_str` on read. All preceding complete lines remain intact because the file was opened in append mode (no seek/truncate).

## Measurement

- **Threshold**: At most 1 corrupted line per crash event (measured by simulated crash test — kill writer mid-append, verify all-but-last line parse successfully)
- **Method**: Integration test `crash_safety_test.rs` writes N events, simulates crash after partial write of event N+1, reads back file and asserts N events parse successfully

## Sources

- `src/writer.rs` — append protocol (single-line JSON + `\n`)
- `src/reader.rs` — skip-on-parse-failure behavior
