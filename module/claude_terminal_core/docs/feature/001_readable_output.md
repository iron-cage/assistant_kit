# Feature: Readable Output

### Scope

- **Purpose**: Turn a terminal's raw output stream into text a person can read, without pretending to be a terminal emulator.
- **In Scope**: `render::to_plain_text`, `render::MAX_ESCAPE_PARAM_CHARS`, and the boundary between what is modelled and what is merely removed.
- **Out of Scope**: Obtaining the stream in the first place — allocating a terminal and spawning onto it (→ [`claude_pty_core`](../../../claude_pty_core/docs/feature/readme.md)), retaining and addressing a hosted session's output (→ [`claude_daemon_core`](../../../claude_daemon_core/docs/feature/004_session_output.md)).

### Why This Exists

A program running on a real terminal does not emit text. What comes back is a byte stream addressed to a screen: escape sequences that set colour, sequences that move a cursor, carriage returns that rewrite a line already printed, spaces that pad a column. Printed verbatim, it is unreadable.

The motivating consumer is `claude_daemon_core`, which hosts Claude Code's interactive REPL on a pty because that REPL only exists on one — a client that sends a prompt and prints the answer needs this last step. But nothing about the transformation depends on that: the input is a `&str` and the output is a `&str`, and any caller holding captured terminal bytes has the same problem.

### What Is Modelled

Exactly one thing: **a cursor moving within the current line.**

| Input | Effect |
|-------|--------|
| printable character | Written at the cursor, overwriting what was there; cursor advances |
| `\n` | Ends the line; cursor to column zero of the next |
| `\r` | Cursor to column zero of the *same* line — what follows overwrites |
| `\t` | Cursor advances to the next multiple of 8 |
| `\b` | Cursor back one column |
| `ESC [ 0K` / `ESC [ K` | Erase from the cursor to the end of the line |
| `ESC [ 1K` | Blank from the start of the line to the cursor, inclusive |
| `ESC [ 2K` | Clear the line; the cursor does not move |

Those cover the idioms a command-line program actually uses to rewrite what it has already printed. `\r` plus `ESC [ K` is the spinner: return to the start, erase what the shorter new text will not cover, print it.

### What Is Not

Everything else is **recognised in order to be removed, never obeyed**:

| Input | Treatment |
|-------|-----------|
| SGR (`ESC [ 31m`) and other presentation | Removed; the text under it survives |
| Cursor addressing (`ESC [ 3;5H`, `ESC [ 2A`) | Removed |
| Erase in display (`ESC [ 2J`) | Removed |
| Alternate screen, scroll regions, mode changes | Removed |
| OSC strings (`ESC ] … BEL`, `ESC ] … ESC \`) | Removed whole |
| Charset selection (`ESC ( B`) | Removed, with its parameter byte |
| Remaining C0 controls and DEL | Dropped — they address a device, not a reader |

A full-screen program that repaints by moving the cursor around therefore renders as every repaint concatenated, in emission order. Legible; not what a screen would have shown.

**Why stop there.** Obeying cursor addressing without modelling a screen is worse than ignoring it — `ESC [ H` followed by new text would silently discard everything printed before it, and the caller would have no way to tell a rendering artifact from a program that said nothing. Removal is wrong in a way that is visible and recoverable; half-emulation is wrong in a way that is neither. A caller who needs the bytes exactly as they arrived can always still have them: this function does not consume the raw stream, and rendering is the caller's own step. The boundary is a guarantee, not an unfinished emulator — see [invariant/002](../invariant/002_line_renderer_boundary.md).

### Trimming

Trailing whitespace is removed from every line, and blank lines are removed from the start and end of the result. Both are padding a screen needs and a transcript does not. Blank lines *between* content are kept — there they are the program's, not the terminal's.

### Bounds

`MAX_ESCAPE_PARAM_CHARS` (64) caps the parameter run inside one escape sequence. Past it, the sequence is abandoned and scanning returns to text. A scanner with no such cap treats the entire remainder of a desynchronised stream as parameters and returns nothing at all — the same reasoning as [`ipc::MAX_IPC_LINE_BYTES`](../../../claude_daemon_core/docs/feature/002_wire_protocol.md), one layer up.

### Why a Function, Not a Type

Rendering needs the whole stream: a `\r` at the boundary between two reads rewrites text that arrived in the previous one. Rather than carry cursor state across calls and require every caller to thread it correctly, a caller accumulates the raw text it read and renders once at the end. Cheap, and impossible to hold wrong.

### Verification

```bash
cd module/claude_terminal_core && ./verb/test
```

Or the single test binary, in-container:

```bash
cargo test -p claude_terminal_core --test render_test
```

`tests/render_test.rs` is pure — a literal in, an exact string out, sixteen cases. The ones that matter are where a naive implementation is *nearly* right: stripping escapes but not honouring the `\r` they accompany (rnd04), honouring `\r` but not the erase that follows it (rnd05), or trimming so eagerly that a blank line the program really printed disappears (rnd12).

To see the difference on a live session:

```bash
clr chat "say hello" --raw    # the bytes as they arrived
clr chat "say hello"          # the same bytes, rendered
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/render.rs` | `to_plain_text` and the scanner |
| doc | [invariant/002_line_renderer_boundary.md](../invariant/002_line_renderer_boundary.md) | Why the boundary is a guarantee |
| doc | [api/001_terminal_surface.md](../api/001_terminal_surface.md) | Full signature contract |
| doc | [`claude_daemon_core` feature/004](../../../claude_daemon_core/docs/feature/004_session_output.md) | Where the motivating raw stream comes from |
| test | `tests/render_test.rs` | Escape removal, cursor motion, trimming, and the parameter cap |
