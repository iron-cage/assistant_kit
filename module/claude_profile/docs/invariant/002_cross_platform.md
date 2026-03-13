# Invariant: Cross-Platform Compatibility

### Scope

- **Purpose**: Ensure `claude_profile` works correctly on all platforms where Claude Code runs.
- **Responsibility**: Documents the cross-platform path operation requirement (NFR-3).
- **In Scope**: Path construction, filesystem operations, environment variable resolution.
- **Out of Scope**: Platform-specific installation procedures, Claude Code binary availability.

### Invariant Statement

All path operations in `claude_profile` must work correctly on Linux, macOS, and Windows.

**Measurable threshold:** All tests pass on Linux, macOS, and Windows CI runners without platform-specific workarounds.

**Required practices:**
- Use `std::path::PathBuf` and `Path::join()` — never string concatenation for path construction
- Use `dirs` crate or `$HOME`/`$USERPROFILE` env vars for home directory resolution (stdlib only per NFR-1)
- Use `std::fs` for all filesystem operations — no POSIX-only syscalls
- `~` shorthand must not appear as a literal — always expand from `HOME` env var

### Enforcement Mechanism

- Code review: reject any use of string concatenation for path construction
- Stdlib-only home dir resolution: `$HOME` (Linux/macOS) or `$USERPROFILE` (Windows)
- CI: run tests on at least Linux in CI; design for macOS/Windows portability

### Violation Consequences

- Paths fail to resolve correctly on Windows (e.g., `~/.claude` literal instead of `C:\Users\...`)
- Breaks tooling integration on macOS or Windows users
- Creates hidden platform-specific bugs that only surface when deployed to a different OS

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/paths.rs` | `ClaudePaths` — all path construction via `PathBuf::from(home).join(...)` |
| source | `src/persist.rs` | `PersistPaths` — `$HOME`/`$USERPROFILE`/`$PRO` resolution chain |
| doc | [010_persistent_storage.md](../feature/010_persistent_storage.md) | `$USERPROFILE` fallback for Windows |
