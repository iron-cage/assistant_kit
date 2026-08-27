# Invariant: Line Renderer Boundary

### Scope

- **Purpose**: Fix the renderer's modelling boundary as a guarantee — exactly one line of cursor state, never a screen — so that consumers can rely on what it will and will not do, and so that "add just one more sequence" is a decision with a stated cost rather than a drift.
- **Governs**: `src/render.rs` — the `State` machine and `apply_csi`.
- **In Scope**: Which control sequences change the output text.
- **Out of Scope**: Which sequences are *recognised* (all of them, in order to be removed), trimming rules (→ [feature/001](../feature/001_readable_output.md)).

### Rule

`to_plain_text` MUST model exactly one thing: **a cursor moving within the current line.** Precisely these inputs may change the text:

| Input | Effect on text |
|-------|----------------|
| printable character | Written at the cursor |
| `\n` | Ends the line |
| `\r` | Cursor to column zero of the same line |
| `\t` | Cursor to the next multiple of 8 |
| `\b` | Cursor back one column |
| `ESC [ 0K` / `ESC [ 1K` / `ESC [ 2K` | Erase within the current line |

Every other recognised sequence MUST be removed without effect. In particular, cursor addressing (`ESC [ H`, `ESC [ A`–`ESC [ D`), erase-in-display (`ESC [ J`), scroll regions, and alternate-screen switches MUST NOT move the cursor between lines or discard previously emitted lines.

**Rationale.** Half-emulation is worse than none. `ESC [ H` obeyed without a screen model would silently discard everything printed before it, and a caller could not distinguish a rendering artifact from a program that said nothing. Removal is wrong in a way that is visible and recoverable — the output contains every repaint, concatenated in emission order — while half-emulation is wrong in a way that is neither.

**Rationale — why a guarantee and not a TODO.** A consumer choosing this crate is choosing a transcript, not a screen snapshot. If the boundary later moved, output that currently shows a program's whole progression would start showing only its final frame, silently, with no signature change. That is a behavioural break dressed as an improvement, so the boundary is recorded here as something a change must argue against rather than something a change may quietly extend.

### Escape Hatch

A caller that genuinely needs screen semantics should not get them by growing this function. It should keep the raw stream — which this crate never consumes — and hand it to a real emulator. `claude_runner`'s `chat --raw` is exactly this: the accumulated bytes as they arrived, no rendering applied.

### Verification

`rnd10` in `tests/render_test.rs` is the direct check:

```rust
assert_eq!( to_plain_text( "first\u{1b}[2J\u{1b}[3;5Hsecond" ), "firstsecond" );
```

Erase-in-display and cursor addressing both removed; `first` survives. A renderer that started obeying either would fail this case rather than silently changing what consumers see.

```bash
cd module/claude_terminal_core && ./verb/test
```

A structural check that no new final byte has been given an effect — `apply_csi` handles `K` and nothing else:

```bash
cd module/claude_terminal_core && grep -n "final_byte" src/render.rs
```

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/render.rs` | `State` machine and `apply_csi` |
| doc | [feature/001_readable_output.md](../feature/001_readable_output.md) | The full modelled/removed tables |
| doc | [001_zero_dependencies.md](001_zero_dependencies.md) | Why an emulator crate is not adopted instead |
| test | `../../tests/render_test.rs` | rnd10 (removed, never obeyed), rnd04–rnd06 (what *is* obeyed) |
