# API: File I/O

### Scope

- **Purpose**: Document the programmatic interface of the claude_core `file_io` module.
- **Responsibility**: Specify the atomic file-replacement and secret-redacting trace-format contracts shared by the crate's settings stores and sibling crates.
- **In Scope**: `atomic_write`, `atomic_write_secret`, `redact_for_trace`.
- **Out of Scope**: The pair-list upsert helper (`upsert_pair` is `pub(crate)` — internal to `settings_io`/`toml_io`), flat-JSON KV I/O (→ `001_settings_io.md`), flat-TOML KV I/O (→ `002_toml_io.md`).

### Abstract

`claude_core::file_io` is the single authority for atomic file replacement — write to a unique sibling temp file, then rename — and for the secret-redacting parameter-trace formatter. Both `settings_io` (JSON) and `toml_io` (TOML) delegate their writes to it, and it is exported for sibling crates that persist credential files (`claude_profile_core`'s store writes use `atomic_write_secret`). Temp names embed pid + per-process sequence + subsecond nanos (audit-unique-tmp-race), so concurrent writers to the same path never truncate each other's in-flight temp file; the rename is the commit point, so a crash mid-write leaves the original file untouched.

### Operations

#### `atomic_write(path: &Path, content: &str) -> Result<(), io::Error>`

Atomically replaces `path`'s content. The temp file is opened with `create_new` (which also refuses to follow a pre-planted symlink at the temp path); on any error the temp file is removed best-effort.

#### `atomic_write_secret(path: &Path, content: &str) -> Result<(), io::Error>`

Like `atomic_write`, but the file is created owner-read/write only (`0o600`) — for credential-bearing files (audit-credential-file-perms). The mode is applied to the temp file's `OpenOptions` before the first byte of content is written and travels through the rename, so an existing world-readable file at `path` is replaced by the `0o600` one with no readable window. On non-Unix platforms the mode request is ignored and this behaves exactly like `atomic_write`.

#### `redact_for_trace(key: &str, value: &str) -> String`

Formats a parameter value for a mutation trace line (audit-trace-token-leak). Values under secret-bearing key names (case-insensitive substring match on `token`, `password`, `passwd`, `pwd`, `secret`, `auth`, `bearer`, `key`, `credential`) or values shaped like credentials (`sk-ant-…` prefixes, `eyJ…` JWTs) are replaced with a length-only `<redacted N chars>` placeholder; everything else is debug-quoted verbatim. Keeps the Task-313 parameter-trace directive (every mutating call traces all its parameters to stderr) compatible with never exposing credential bytes.

### Error Handling

`atomic_write`/`atomic_write_secret` return `Err` if `path` has no filename component, or if the temp-file write or the final rename fails; on error the temp file is removed best-effort so no stale temp file is left next to the store. `redact_for_trace` is infallible.

### Compatibility Guarantees

- The rename is the commit point: a crash at any earlier moment leaves the previous file content fully intact.
- Temp names are unique per writer (pid + sequence + nanos) — concurrent writers to the same path never interleave.
- `atomic_write_secret` never exposes content under a mode wider than `0o600` at any point of the write (Unix).
- `redact_for_trace` never prints a value verbatim when either the key-atom or the value-shape check fires; the placeholder reveals only the character count.

### Sources

| File | Relationship |
|------|--------------|
| `../../src/file_io.rs` | All operations, unique-temp-name generation, sensitive-key atom list |

### Tests

| File | Relationship |
|------|--------------|
| `../../tests/file_io_test.rs` | Create/replace behavior, no-temp-left-behind, no-filename rejection, `0o600` mode, redaction key-atom/value-shape/verbatim/over-redaction coverage |
