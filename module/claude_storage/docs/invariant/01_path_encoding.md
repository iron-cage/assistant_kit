# Invariant: Path Encoding

### Scope

- **Purpose**: Define the path encode/decode behavioral contract.
- **Responsibility**: Encoding rule, decode disambiguation procedure, round-trip guarantee.
- **In Scope**: Which characters encode to `-`, disambiguation via filesystem DFS.
- **Out of Scope**: URL encoding, filesystem permissions, storage root selection.

### Statement

Both `/` and `_` encode to `-`. The encoding is therefore **lossy**: the stored key does not
uniquely identify the original path. Decoding is non-deterministic without external context.

### Encoding Rule

| Input character | Encoded character |
|-----------------|-------------------|
| `/` | `-` |
| `_` | `-` |
| all others | unchanged |

**Example**:
- Input path: `/home/alice/projects/my_app/module`
- Encoded key: `-home-alice-projects-my-app-module`

### Project Path Format

Claude Code project paths are always absolute, e.g.:
- `/home/alice/projects/my-app/module/reasoner`
- `/home/alice/projects/my-project`
- `/home/alice/projects/project-a`

**Mapping to storage**:
- Path: `/home/alice/projects/my-app/module/reasoner`
- Encoded: `-home-alice-projects-my-app-module-reasoner`
- Storage: `~/.claude/projects/-home-alice-projects-my-app-module-reasoner/`

### Disambiguation

`decode_path_via_fs()` resolves the encoding ambiguity by DFS-walking the filesystem,
matching the stored key against all candidate project directories at the storage root,
and returning the first filesystem-confirmed match.

This means two different project paths that encode identically are disambiguated by
checking which one actually exists on disk. The algorithm assumes the caller's working
environment matches the storage origin.

### Contract

- **Encode**: replace every `/` and `_` with `-`
- **Decode**: DFS over storage root candidates; first filesystem-confirmed match wins
- **Round-trip guarantee**: `encode(decode(k)) == k` always holds
- **Inverse guarantee**: `decode(encode(p)) == p` holds only when the filesystem contains
  exactly one candidate matching the encoded key

### Violation Conditions

- Storing a decoded path as a storage key (bypasses encoding; produces unmatchable keys)
- Assuming encode is injective (it is not — `/home/foo/bar` and `/home/foo_bar` both encode to `-home-foo-bar`)
- Decoding without filesystem access (ambiguity is unresolvable without on-disk confirmation)
- Calling `decode_path_via_fs()` from a machine that does not share the original filesystem

### Referenced Commands

| # | Command | Context |
|---|---------|---------|
| 8 | [`.project.path`](../cli/command/08_project_path.md) | Returns encoded storage key for a project directory |
| 10 | [`.session.dir`](../cli/command/10_session_dir.md) | Uses encoded path to locate session directory |
| 11 | [`.session.ensure`](../cli/command/11_session_ensure.md) | Creates directory using encoded path |

### Sources

| File | Relationship |
|------|--------------|
| `src/` | `decode_path_via_fs()` implementation in `claude_storage_core` |
| [`algorithm/001_agent_session_tracking.md`](../algorithm/001_agent_session_tracking.md) | Layout examples and detection algorithms |
