# API: Terminal Surface

### Scope

- **Purpose**: Pin the signature and behavioral contract of every item `claude_terminal_core` exports, so a consumer can depend on it without reading the source.
- **In Scope**: All items re-exported from `lib.rs`, plus the `render` module path they are also reachable through.
- **Out of Scope**: The private scanner helpers (`put`, `apply_csi`, `tidy`), which are implementation.

### Errors

None. The crate exports no error type and no fallible function. Rendering is total: every `&str` is valid input, including empty, including a stream truncated mid-sequence.

### `to_plain_text`

| Signature | Contract |
|-----------|----------|
| `to_plain_text( raw : &str ) -> String` | Total, never panics, never fails. Obeys in-line cursor motion, removes every other escape sequence, trims trailing whitespace per line and blank lines at both ends. |

**Guarantees:**

- **Total.** Any `&str` is accepted. A sequence truncated at end of input is dropped; a desynchronised stream is bounded by `MAX_ESCAPE_PARAM_CHARS` rather than swallowing the remainder.
- **Pure.** Output depends only on `raw`. No clock, no environment, no filesystem, no interior mutability.
- **Whole-stream.** Cursor state does not persist across calls, by design — a `\r` spanning two calls cannot rewrite the earlier one. Accumulate, then render once ([feature/001](../feature/001_readable_output.md), "Why a Function, Not a Type").
- **Bounded modelling.** Exactly one line of cursor state; screen sequences are removed, never obeyed. This is a guarantee, not an unfinished emulator — [invariant/002](../invariant/002_line_renderer_boundary.md).

**What changes the text** — printable characters, `\n`, `\r`, `\t`, `\b`, and `ESC [ 0K` / `ESC [ 1K` / `ESC [ 2K`. Everything else recognised is removed without effect. The full tables are in [feature/001](../feature/001_readable_output.md).

```rust
use claude_terminal_core::to_plain_text;

assert_eq!( to_plain_text( "working... \r\u{1b}[Kdone" ), "done" );
assert_eq!( to_plain_text( "\u{1b}[31mred\u{1b}[0m" ), "red" );
assert_eq!( to_plain_text( "" ), "" );
```

### `MAX_ESCAPE_PARAM_CHARS`

| Signature | Contract |
|-----------|----------|
| `pub const MAX_ESCAPE_PARAM_CHARS : usize` | `64`. The longest parameter run accepted inside one escape sequence; past it the sequence is abandoned and scanning returns to text. |

Public so a caller constructing adversarial input for its own tests can name the boundary rather than hardcode `64`. Raising it is a compatible change; lowering it can alter output for a desynchronised stream.

### Module Path

Both items are exported at the crate root and from the `render` module:

```rust
use claude_terminal_core::to_plain_text;          // preferred
use claude_terminal_core::render::to_plain_text;  // equivalent
```

The root re-export is the stable path; `render` is public so that documentation links resolve against the module that carries the scanner's own doc comment.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/lib.rs` | The re-exports this document pins |
| source | `src/render.rs` | `to_plain_text`, `MAX_ESCAPE_PARAM_CHARS`, and the scanner |
| doc | [feature/001_readable_output.md](../feature/001_readable_output.md) | Behavior behind `to_plain_text` |
| doc | [invariant/002_line_renderer_boundary.md](../invariant/002_line_renderer_boundary.md) | The modelling boundary as a guarantee |
| test | `../../tests/render_test.rs` | Sixteen exact-output cases, including the parameter cap |
