# Algorithm: JSON Parser

### Scope

- **Purpose**: Parse JSON values from Claude Code's JSONL conversation files without any runtime dependencies.
- **Responsibility**: Documents the hand-written recursive descent parser design, rationale, and correctness properties.
- **In Scope**: Parser design, zero-dependency rationale, supported JSON types, UTF-8 handling fix.
- **Out of Scope**: The JSONL entry structure it parses (→ `data_structure/001_storage_hierarchy.md`), the storage reading pipeline.

### Abstract

The parser is a hand-written recursive descent implementation covering all JSON value types. It was written specifically to satisfy the zero-runtime-dependency invariant — no `serde_json` or equivalent. It is optimized for Claude Code's specific JSONL format and provides full control over error messages.

### Algorithm

The parser processes a UTF-8 string using a byte-oriented cursor. Dispatch is based on the first non-whitespace byte:

- `{` → parse object: read key-value pairs until `}`, separated by `,`.
- `[` → parse array: read values until `]`, separated by `,`.
- `"` → parse string: advance byte-by-byte, handling escape sequences.
- `t`/`f` → parse boolean literal.
- `n` → parse null literal.
- digit/`-` → parse number as f64.

**String escape handling.** Recognizes `\n`, `\t`, `\r`, `\\`, `\"`, `\/`, `\b`, `\f`, and `\uXXXX` Unicode escapes. Surrogate pairs (`\uD800`–`\uDFFF`) are handled by pairing high and low surrogates and converting to the corresponding Unicode scalar.

**UTF-8 correctness.** The parser uses byte offsets for cursor positioning. Multi-byte UTF-8 sequences are handled correctly because string content is accumulated by scanning escape sequences (which are ASCII) and copying raw bytes for non-escape content. The UTF-8 fix (bug-1, 2025) addressed an indexing error at the boundary between escape-sequence scanning and raw byte copying.

**Performance.** Approximately 80ns per parse operation on typical Claude Code entry JSON. The parser is fast enough that it is not the bottleneck for any measured workload.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `../../src/json.rs` | Full parser implementation (~690 lines) |
| test | `../../tests/json_multibyte_bug.rs` | UTF-8 multi-byte character regression test |
| test | `../../tests/json_surrogate_pair_bug.rs` | Surrogate pair Unicode handling regression test |
| doc | `../invariant/001_safety_guarantees.md` | Format validation invariant enforced during parse |

### Sources

| File | Notes |
|------|-------|
| `spec.md` (deleted — migrated here) | Combined specification; JSON parser section extracted here |
