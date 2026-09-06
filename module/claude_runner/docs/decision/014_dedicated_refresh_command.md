# Decision: Dedicated refresh Command

**ID:** D14 · **Category:** Behavior · **Status:** ✅ Adopted

### Scope

- **Purpose**: Record why credential refresh got its own `clr refresh` command instead of reusing `isolated` with a throwaway invocation.
- **Responsibility**: Rationale for the dedicated command, its fixed-argument wrapping of `run_isolated()`, and the timeout/exit-code differences from `isolated`.
- **In Scope**: Why `refresh` exists as a distinct command; what it wraps; its timeout and exit-code semantics.
- **Out of Scope**: `isolated`'s own subprocess defaults (→ [`../invariant/005_isolated_subprocess_defaults.md`](../invariant/005_isolated_subprocess_defaults.md)); credential store paths (→ contract docs).

### Decision

`clr refresh --creds <FILE>` wraps `run_isolated()` with fixed args `["--print", "."]`.

### Rationale

`clr isolated` is designed for running real tasks in credential isolation. Credential refresh is a distinct operational intent: no user task, no output, just token renewal. Encoding the `["--print", "."]` invocation trick inside `isolated` forces users to know the implementation detail — that a trivial print-mode prompt is what triggers a token exchange without doing real work. A dedicated `clr refresh` makes intent self-documenting: the command name states the purpose, and the trick lives behind it rather than in every caller's script.

### Consequence

- Default timeout is 45s (vs 30s for `isolated`) to accommodate slow OAuth token exchange
- Exit 0 means credentials were refreshed
- Exit 1 means error or no refresh
- Exit 2 means timeout without refresh

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Decision collection index |
| invariant | [`../invariant/005_isolated_subprocess_defaults.md`](../invariant/005_isolated_subprocess_defaults.md) | `isolated`'s own subprocess defaults, timeout 30s |
| invariant | [`../invariant/006_exit_codes.md`](../invariant/006_exit_codes.md) | Full exit code contract including `refresh`'s 0/1/2 mapping |
| source | `../../src/cli/credential.rs` | `run_isolated_command()`, `run_refresh_command()` |
| test | `../../tests/refresh_test.rs` | `clr refresh` command behavior |
