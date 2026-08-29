# Decision: Binary Named clr

**ID:** D10 · **Category:** Naming · **Status:** ✅ Adopted

### Scope

- **Purpose**: Record why the installed binary and the Rust crate carry different names.
- **Responsibility**: Rationale for the short binary name and for leaving the crate name untouched.
- **In Scope**: Why `clr` for the binary, why `claude_runner` for the crate, and what the split changes.
- **Out of Scope**: Command naming within the CLI (→ [013_commands_are_bare_words.md](013_commands_are_bare_words.md)); the public API surface consumers import (→ [`../api/001_public_api.md`](../api/001_public_api.md)).

### Decision

The installed binary is `clr`; the Rust crate/lib remains `claude_runner`.

### Rationale

`clr` is short and fast to type — the tool is used interactively many times per session, and it mirrors the `cm` convention of `claude_version`. The crate name stays `claude_runner` so existing `use claude_runner::COMMANDS_YAML` consumers are unaffected; only the `[[bin]] name` in `Cargo.toml` changes.

The two names answer to different audiences. A human types the binary name dozens of times a day and pays for every character; a `use` statement is written once and read as documentation, where the descriptive name is worth more than the brevity.

### Consequence

- `cargo install --path .` installs `clr`
- `CARGO_BIN_EXE_clr` is the env var integration tests use to locate the built binary
- All docs and help text show `clr`

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Decision collection index |
| api | [`../api/001_public_api.md`](../api/001_public_api.md) | The `claude_runner` library surface consumers import |
| decision | [013_commands_are_bare_words.md](013_commands_are_bare_words.md) | Naming convention *inside* the CLI |
