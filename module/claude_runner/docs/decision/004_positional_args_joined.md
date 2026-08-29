# Decision: Positional Args Joined

**ID:** D4 · **Category:** Syntax · **Status:** ✅ Adopted

### Scope

- **Purpose**: Record why multiple positional arguments are joined into a single message rather than rejected or treated as distinct operands.
- **Responsibility**: Rationale for space-joining positionals, and the convention it follows.
- **In Scope**: The joining rule and its effect on quoting.
- **Out of Scope**: `[MESSAGE]` reference semantics and its role as a print-mode trigger (→ [`../cli/param/001_message.md`](../cli/param/001_message.md)).

### Decision

Multiple positional arguments are joined with spaces: `clr Fix the bug` becomes message `"Fix the bug"`.

### Rationale

Standard CLI convention, like `git commit -m`. Eliminates the need to quote simple messages — the common interactive case becomes the shortest one to type.

### Consequence

No invocation form needs quoting for a plain message. Quoting still matters where the shell itself would mangle the text (globs, `$`, `;`), because the join happens after the shell has already split the words.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Decision collection index |
| cli | [`../cli/param/001_message.md`](../cli/param/001_message.md) | `[MESSAGE]` positional parameter reference |
| test | `../../tests/cli_args_test.rs` | Flag and positional parsing coverage |
