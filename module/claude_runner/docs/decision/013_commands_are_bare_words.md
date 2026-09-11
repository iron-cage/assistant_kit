# Decision: Commands Are Bare Words

**ID:** D13 · **Category:** Syntax · **Status:** ✅ Adopted

### Scope

- **Purpose**: Record why `clr` commands are bare words rather than `--` flags, and why that namespace is kept separate from parameters.
- **Responsibility**: Rationale for the bare-word convention and the consequence for `help` specifically.
- **In Scope**: The command/parameter namespace split and its effect on shell completion, typo detection, and user mental models.
- **Out of Scope**: The full command naming specification (→ [`../invariant/003_command_naming.md`](../invariant/003_command_naming.md)); the crate/binary naming split (→ [010_binary_named_clr.md](010_binary_named_clr.md)).

### Decision

Commands select a mode of operation (`run`, `isolated`, `refresh`, `help`). Parameters modify that mode (`--model`, `--creds`, `--trace`). Behavioral specification: [`../invariant/003_command_naming.md`](../invariant/003_command_naming.md).

### Rationale

Mixing these namespaces — e.g. `--help` as the only way to invoke help — breaks the lexical distinction and confuses shell completers, subcommand typo detection, and user mental models. A bare word answers "what mode am I entering"; a `--flag` answers "how do I adjust the current mode." Collapsing both into `--` syntax erases that distinction for both the parser and the person reading the invocation.

### Consequence

`help` is now invocable as `clr help` (bare word subcommand). `--help` and `-h` remain as parameter aliases for POSIX compliance — the alias exists for convention, not as the primary form. `KNOWN_SUBCOMMANDS` includes `"help"` alongside `"isolated"` and `"refresh"`.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Decision collection index |
| invariant | [`../invariant/003_command_naming.md`](../invariant/003_command_naming.md) | Full command naming specification |
| decision | [010_binary_named_clr.md](010_binary_named_clr.md) | Naming convention *outside* the CLI (binary vs crate) |
