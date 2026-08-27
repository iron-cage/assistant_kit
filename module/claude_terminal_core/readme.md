# claude_terminal_core

Pure library for interpreting terminal output as plain text (zero dependencies).

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `Cargo.toml` | Crate manifest and dependency configuration |
| `src/` | Core library implementation |
| `tests/` | Test suite for escape removal, cursor motion, and trimming |
| `docs/` | Behavioral requirements: features, invariants, api |
| `verb/` | Shell scripts for each `do` protocol verb. |

## overview

Turns a terminal's raw byte stream into the text a person would have read.
Knows nothing about Claude Code, daemons, or sessions — it is a scanner over a
`&str`. Zero dependencies.

This is the counterpart to `claude_pty_core`, and the split is deliberate: that
crate owns the **device** (allocating a pseudo-terminal, putting a child process
on it), this one owns the **protocol spoken over it**. Interpreting escape
sequences needs no pty, and a caller holding captured bytes should not have to
link POSIX FFI to read them.

## features

- **Zero dependencies**: one scanner over `&str`, no emulator crate, no `libc`
- **Line-accurate rewrites**: `\r`, `ESC [ K` and `\b` are obeyed exactly — the
  idioms a command-line program uses to rewrite what it already printed
- **Screen sequences removed, never obeyed**: cursor addressing, scroll regions
  and alternate screens are recognised only well enough to be dropped
- **Bounded desync**: a parameter run past `MAX_ESCAPE_PARAM_CHARS` abandons the
  sequence rather than swallowing the rest of the stream

## usage

```toml
[dependencies]
claude_terminal_core = { workspace = true }
```

```rust
use claude_terminal_core::to_plain_text;

// A spinner rewriting its own line leaves only what it settled on.
assert_eq!( to_plain_text( "working... \r\u{1b}[Kdone" ), "done" );

// Colour is presentation; the text under it survives.
assert_eq!( to_plain_text( "\u{1b}[31mred\u{1b}[0m" ), "red" );
```

## architecture

**Why a line renderer and not an emulator.** A real emulator is a large amount of
state to carry, and the alternative to carrying it is not "slightly worse output"
but a dependency on getting scroll semantics right in order to print a sentence.
A full-screen program that repaints by moving the cursor renders here as every
repaint concatenated in emission order — legible, but not what a screen would
have shown. Callers that need the bytes exactly as they arrived should ask for
them; the raw stream is always still there.

**Why a function and not a type.** Rendering needs the whole stream: a `\r` at
the boundary between two reads rewrites text that arrived in the previous one.
Rather than carry cursor state across calls and make every caller thread it
correctly, a caller accumulates the raw text it read and renders once. Cheap, and
impossible to hold wrong.
