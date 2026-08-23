# src/

Append-only JSONL event journal for CLR.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `lib.rs` | Crate root; public module re-exports and crate documentation |
| `event.rs` | `EventType`, `EventRecord`, `EventFields` serializable record types |
| `writer.rs` | Append-only JSONL writer; open-write-close per event |
| `reader.rs` | Filtered query and tail iteration over JSONL files |
| `rotation.rs` | Daily UTC-dated filename generation and retention pruning |
