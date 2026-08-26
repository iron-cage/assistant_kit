# Invariant: Unsafe Containment

### Scope

- **Purpose**: Keep every `unsafe` block in this crate inside one auditable module, so reviewing the crate's soundness means reviewing one file rather than five.
- **Governs**: All modules under `src/`.
- **In Scope**: `unsafe` blocks, `unsafe fn`, `unsafe impl`, and the `#![ allow( unsafe_code ) ]` attribute.
- **Out of Scope**: Test code under `tests/`, which calls only the safe surface and needs no `unsafe` of its own.

### Rule

`src/ffi.rs` is the only module in this crate whose **code** may contain the token `unsafe`. Every other module — `pty.rs`, `session.rs`, `writer.rs`, `error.rs`, `env_scrub.rs`, `lib.rs` — MUST be free of it.

Comments and doc comments are exempt, and deliberately so: a crate that exists because it hand-rolls POSIX FFI cannot explain itself under a rule forbidding the word. `lib.rs` states this invariant in its own module doc, and that is the invariant being honoured rather than broken.

The workspace sets `unsafe-code = "deny"` at the lint level, so the allowance is granted once, module-wide, at the top of `ffi.rs`:

```rust
#![ allow( unsafe_code ) ]
```

A per-block `#[ allow ]` elsewhere would satisfy the compiler while defeating this invariant, which is why the check greps for the token rather than relying on the lint alone.

**Rationale:** This crate exists because the safe alternatives did not fit — `pty-process` requires edition 2024 against a workspace pinned to Rust 1.75, and `portable-pty` puts `anyhow` in its public API in a workspace that mandates a single error convention. Hand-rolling POSIX FFI was the price of not taking those dependencies. Containment is what keeps that price bounded: the unsafe surface is six `extern "C"` declarations and four wrapper functions, and it does not grow silently as the crate does.

Every `unsafe` block inside `ffi.rs` additionally carries a `// SAFETY:` comment stating why the call is sound — the workspace sets `undocumented_unsafe_blocks = "deny"`, so an undocumented block fails the build rather than the review.

### Verification

```bash
cd module/claude_pty_core && \
  for f in src/*.rs; do
    [ "$f" = src/ffi.rs ] && continue
    sed 's|//.*||' "$f" | grep -q 'unsafe' && echo "$f"
  done
```

Prints nothing when the invariant holds. `sed` drops each line's comment first, so the scan sees code only — these sources use no block comments and no `//` inside a string literal, which is what makes cutting at the first `//` exact rather than approximate here. `tests/unsafe_containment_test.rs` performs the same scan, asserts both of those preconditions instead of assuming them, and fails with the offending modules listed.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/ffi.rs` | The only module permitted to contain `unsafe` |
| doc | [feature/001_pty_allocation.md](../feature/001_pty_allocation.md) | What the FFI calls are for |
| doc | [002_zero_dependencies.md](002_zero_dependencies.md) | Why the FFI exists at all |
| test | `tests/unsafe_containment_test.rs` | Mechanical enforcement |
