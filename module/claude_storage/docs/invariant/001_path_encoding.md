# Invariant: Path Encoding

### Scope

- **Purpose**: Define the path encode/decode behavioral contract.
- **Responsibility**: Encoding rule, decode disambiguation procedure, round-trip guarantee.
- **In Scope**: Which characters encode to `-`, disambiguation via filesystem DFS.
- **Out of Scope**: URL encoding, filesystem permissions, storage root selection.

### Statement

Every non-alphanumeric character — `/`, `_`, `.`, literal `-`, and any other special
character — encodes to `-` (or `--` when it is the first character of a path component).
The encoding is therefore **lossy**: the stored key does not uniquely identify the
original path. Decoding is non-deterministic without external context.

### Encoding Rule

| Input character | Encoded character |
|-----------------|-------------------|
| `/` | `-` |
| `_` | `-` |
| `.`, and any other non-alphanumeric character | `-` |
| a component's first character, when it is `.`, `_`, or literal `-` | `--` (escaped, to stay distinguishable from an ordinary path-separator `-`) |
| all other (alphanumeric) characters | unchanged |

**Example**:
- Input path: `/home/alice/projects/my_app/module`
- Encoded key: `-home-alice-projects-my-app-module`

- Input path: `/home/alice/.config/my_app`
- Encoded key: `-home-alice--config-my-app`

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

`decode_path_via_fs()` (`../../src/cli/scope.rs`) resolves the encoding ambiguity against the
real filesystem — not a simple linear DFS over a flat candidate list, but a component-by-
component walk (`walk_fs()`) that enumerates each level's ACTUAL `read_dir()` entries and
forward-matches each entry's own re-encoding (via `claude_storage_core::encode_component_piece`)
against the remaining unconsumed portion of the encoded key. Each recursive step resolves to one
of four outcomes:

- **Full** — the remaining string was consumed by exactly ONE real candidate: an unambiguous
  complete resolution.
- **AmbiguousFull** — the remaining string was consumed COMPLETELY by 2+ DIFFERENT real
  candidates at the same level (a genuine `encode_component_piece` collision spanning a
  component boundary). The full candidate set is preserved rather than collapsed to a common
  ancestor; each caller (`matches_under`/`matches_relevant`/`matches_local`) checks its own
  directional relationship against every candidate individually and conservatively includes
  when at least one qualifies.
- **Partial** — the best (longest-consumed) real prefix found, tie-broken by consumed length
  (not raw byte length) when 2+ candidates tie, with an extension-sibling check promoting a
  real sibling whose name textually extends the winner whenever the winner's own piece was too
  short to have forward-matched past the 200-character truncation boundary (see below).
- **NotFound** — no real filesystem entry corresponds to even the first component.

**200-character truncation boundary**: once the fully-assembled encoded key exceeds 200
characters, `encode_path()` truncates it and appends a hash-of-the-original-string suffix (see
`claude_storage_core`'s own
[`algorithm/001_path_encoding.md`](../../../claude_storage_core/docs/algorithm/001_path_encoding.md)
for the exact encode-side mechanism). On the decode side this means: (a) a `Partial` result whose
own consumed length already exceeds 200 characters falls back to `search_encoded_subtree`,
directly re-encoding real subtree entries to find the truncation-hidden target; (b) fast-reject
string checks that assume a literal prefix relationship between a shorter candidate's encoded
key and a longer one's (`is_relevant_encoded`, `matches_under`) must special-case the situation
where BOTH sides of a genuine ancestor/descendant pair independently exceed 200 characters —
each then carries a DIFFERENT hash suffix after an identical shared body, breaking the literal-
prefix assumption even though the real relationship is genuine; the fast-reject conservatively
defers to real filesystem verification in that case rather than false-excluding.

The algorithm assumes the caller's working environment matches the storage origin. This decode
algorithm's full evolution — 10 defects found and fixed via this session's MAAV
adversarial-verification process — is recorded in `task/claude_storage/bug/completed/509_*.md`
through `518_*.md` (sibling directory to this repo root). Read those for detailed root-cause
narratives; this section states only the CURRENT resulting contract.

### Contract

- **Encode**: replace every `/` and `_` with `-` (see `claude_storage_core`'s own encoding
  algorithm doc, linked above, for the full character-substitution and 200-char-truncation rule)
- **Decode**: real-filesystem-guided, component-by-component candidate enumeration (see
  Disambiguation above) — NOT a simple "first match wins" search; a genuine, irreducible
  ambiguity (`AmbiguousFull`) is preserved as a candidate SET rather than collapsed to an
  arbitrary or common-ancestor winner, and each caller applies its own relationship check
  per-candidate
- **Round-trip guarantee**: `encode(decode(k)) == k` always holds
- **Inverse guarantee**: `decode(encode(p)) == p` holds only when the filesystem contains
  exactly one candidate matching the encoded key; when 2+ real candidates exist, decode
  correctly reports the ambiguity (`AmbiguousFull`) rather than silently picking one

### Violation Conditions

- Storing a decoded path as a storage key (bypasses encoding; produces unmatchable keys)
- Assuming encode is injective (it is not — `/home/foo/bar` and `/home/foo_bar` both encode to `-home-foo-bar`)
- Decoding without filesystem access (ambiguity is unresolvable without on-disk confirmation)
- Calling `decode_path_via_fs()` from a machine that does not share the original filesystem
- Assuming `decode_path_via_fs()` always resolves to a single unambiguous real path — a genuine
  `encode_component_piece` collision across 2+ real candidates is a real, supported outcome
  (`AmbiguousFull`), not an error condition; code consuming this API must handle it explicitly
  rather than assuming a `Full` result is the only success case

### Referenced Commands

| # | Command | Context |
|---|---------|---------|
| 8 | [`.project.path`](../cli/command/08_project_path.md) | Returns encoded storage key for a project directory |
| 10 | [`.session.dir`](../cli/command/10_session_dir.md) | Uses encoded path to locate session directory |
| 11 | [`.session.ensure`](../cli/command/11_session_ensure.md) | Creates directory using encoded path |

### Sources

| File | Relationship |
|------|--------------|
| `../../src/cli/scope.rs` | `decode_path_via_fs()`/`walk_fs()` real-filesystem-guided decode implementation (`claude_storage`, not `claude_storage_core`) |
| `../../../claude_storage_core/src/path.rs` | `encode_path()`/`encode_component_piece()` encoding implementation |
| [`algorithm/001_agent_session_tracking.md`](../algorithm/001_agent_session_tracking.md) | Layout examples and detection algorithms |
