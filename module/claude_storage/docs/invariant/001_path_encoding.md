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
of five outcomes:

- **Full** — the remaining string was consumed by exactly ONE real candidate: an unambiguous
  complete resolution.
- **AmbiguousFull** — the remaining string was consumed COMPLETELY by 2+ DIFFERENT real
  candidates at the same level (a genuine `encode_component_piece` collision spanning a
  component boundary). The full candidate set is preserved rather than collapsed to a common
  ancestor; each caller (`matches_under`/`matches_relevant`/`matches_local`) checks its own
  directional relationship against every candidate individually and conservatively includes
  when at least one qualifies.
- **AmbiguousPartial** — 2+ real candidates tie as the best (longest-consumed) Partial match
  when the unconsumed remainder (e.g. a synthetic `--topic` suffix) prevents every candidate
  from reaching Full. As with `AmbiguousFull`, the full candidate set is preserved rather than
  collapsed to the candidates' common ancestor — collapsing reintroduces the shared-parent
  false-inclusion shape one confidence level down (BUG-526) — and each caller checks its own
  directional relationship against every candidate individually, conservatively including when
  at least one qualifies. The consumed length credited to a PARENT level's own competition when
  this result is itself a nested (non-outermost) recursive step is the tied candidates' own
  verified, shared consumed length — never a hardcoded placeholder — since which candidate wins
  the tie is ambiguous but how much every tied candidate consumed reaching it is not; crediting
  it as though it were unresolved starves a genuinely deeper tied subtree against an unrelated,
  less-verified sibling one level up (BUG-528).
- **Partial** — the best (longest-consumed) real prefix found, tie-broken by consumed length
  (not raw byte length) when 2+ candidates tie. When the winning candidate is a single best
  match (no tie) AND `total_len >= 200` (the total encoded key's own length reaching the
  truncation boundary at all, identically the condition that opens the rescue search itself,
  `encoded.len() > 200` — so the defer can never fire on a short key where truncation is
  impossible by construction and the retreat would go unrescued, BUG-525) AND some OTHER,
  non-winning candidate's own on-disk piece agrees with the unconsumed remainder ITSELF
  (`common_prefix_len(piece, remaining)` — never the winning candidate's own piece length or
  verified depth) out to the fixed 199-character real-content boundary, `walk_fs()` does NOT
  guess which candidate is correct — guessing via an in-memory `piece.len() > remaining.len()`
  comparison was found unsound (`remaining`'s own tail is inflated by `encode_path()`'s appended
  hash-plus-topic bytes once truncation has actually occurred, silently defeating the comparison,
  BUG-524), and keying the defer on the WINNING candidate's own verified depth was found unable
  to distinguish genuine truncation ambiguity from an unrelated long synthetic `--topic` tag
  riding on an otherwise short, already-complete winning encoding — appended entirely AFTER
  `encode_path()` already ran, inflating `total_len` with no real truncation in play (BUG-529) —
  it instead defers unconditionally to the current level (`Partial(base)`), letting the single
  `search_encoded_subtree` fallback (see below) make the final, filesystem-verified
  determination.
- **NotFound** — no real filesystem entry corresponds to even the first component.

**200-character truncation boundary**: once the fully-assembled encoded key exceeds 200
characters, `encode_path()` truncates it and appends a hash-of-the-original-string suffix (see
`claude_storage_core`'s own
[`algorithm/001_path_encoding.md`](../../../claude_storage_core/docs/algorithm/001_path_encoding.md)
for the exact encode-side mechanism). On the decode side this means: (a) a `Partial` result whose
own consumed length already exceeds 200 characters falls back to `search_encoded_subtree`,
directly re-encoding real subtree entries — recursing into children BEFORE checking the current
level's own directory, and matching via topic-boundary-aware prefix
(`target.starts_with("{encoded}--")`) in addition to exact equality, so a real descendant past
the truncation boundary is still found when it additionally carries a synthetic topic suffix on
top of its own already-truncated key (BUG-522) — to find the truncation-hidden target. The loose
prefix match fires only at proper-descendant levels of the search, never at any of the search's
own **stall points** — every directory the search's own entry function already stalled at
without further filesystem descent: for the single-candidate rescue (`search_encoded_subtree`)
this is the one candidate itself; for the tied-candidate rescue (`search_encoded_subtree_tied`,
invoked when `walk_fs()` found 2+ tied best candidates) this is BOTH the tied candidates
themselves AND their shared common ancestor. At a stall point the candidate is merely where the
search itself stalled — nothing deeper was verified — so "candidate + `--`" is stringwise
indistinguishable from a DELETED deeper path's own special-leading component, and promoting it
to a confident `Full` would strip the `Partial`/tie conservative-include disjunct and falsely
EXCLUDE a session genuinely nested under a deleted query anchor (BUG-527, single-candidate case;
BUG-530, tied-candidate case — including the tied candidates' own common ancestor, which a
special-leading-character-escape collision between the tied siblings can otherwise expose to the
identical false-promotion risk, since that escape's own `--` separator sits immediately after the
ancestor's own encoding by construction); exact equality stays enabled at every level including
every stall point. This
subtree search examines every sibling to completion at each level rather than returning on the
first match found (an early return silently depends on `read_dir()`'s platform-unspecified
enumeration order to decide a genuine tie, BUG-523); when 2+ real candidates remain after the
full search, `rank_subtree_candidates()` ranks them by specificity — an exact `encode_path()`
match always outranks a merely-loose topic-boundary-prefix match, and among same-tier candidates
the longest `encode_path()` output wins (mirroring the `Partial`-arm's own consumed-length
tie-break) — so a real, specific match is never wrongly reported as ambiguous alongside an
unrelated sibling whose name merely, coincidentally, textually extends it; only candidates tied
on BOTH the exactness tier AND the encoded length produce `AmbiguousFull` (BUG-523); (b) fast-reject
string checks that assume a literal prefix relationship between a shorter candidate's encoded
key and a longer one's (`is_relevant_encoded`, `matches_under`) must special-case the situation
where BOTH sides of a genuine ancestor/descendant pair independently exceed 200 characters —
each then carries a DIFFERENT hash suffix after an identical shared body, breaking the literal-
prefix assumption even though the real relationship is genuine.

Bypassing the fast-reject in that situation is necessary but not, by itself, sufficient: "both
sides exceed 200 characters" alone does not distinguish a genuine ancestor/descendant pair from
two UNRELATED paths that merely share a shallow real filesystem ancestor (any two sufficiently
long, independently-truncated paths under the same tree qualify). The bypass additionally
requires the two encodings' literal first-200-character bodies to match
(`double_truncated_and_related()` in `scope.rs`, compared via the char-boundary-safe
`common_prefix_len()` helper rather than a raw byte-index slice — a real, filesystem-sourced
directory name is not guaranteed ASCII the way `encode_path()`'s own output is, so a raw slice
can panic when a multi-byte character straddles the comparison boundary, BUG-521) before
deferring to real filesystem verification — a genuine ancestor/descendant pair is guaranteed to
share this literal body, since `encode_path()` concatenates path components strictly additively
before truncating, while unrelated siblings diverge well before the 200th character. Without
this precondition, the `Partial`-arm conservative-include check (`base_path.starts_with(&p)`)
can false-include an unrelated candidate whenever its own deeper subtree isn't materialized on
disk and `walk_fs()` can only confirm the shared, shallow, real ancestor.

This body-match precondition does not prove soundness against every possible truncation shape,
and this residual gap is now formally resolved (not merely flagged) as an **accepted,
documented limitation, not a defect**. Two confirmed causes produce an identical first-200-
character body without a genuine ancestor/descendant relationship, and neither is closeable by
any finite-prefix-length string comparison: once the first 200 characters of two paths'
pre-truncation encodings are already forced identical (for whatever reason), the entire
comparison window any such check could inspect is exhausted before either side's own
distinguishing structure ever appears, leaving zero budget to observe anything past that
boundary. (1) **Deep shared ancestor** (BUG-520): if the paths' shared real ancestor is itself
deep enough that its OWN pre-truncation encoding already exceeds 200 characters, two genuinely
unrelated siblings under that ancestor inherit an identical first-200-character body from it,
despite neither being nested in the other (`it_91`/`it_92`). (2) **Shallow ancestor plus a
component-boundary escape collision** (BUG-520 scope broadened by BUG-533): even with a SHORT
shared ancestor, one side's own diverging tail can independently exceed 200 characters and
collide, byte-for-byte, with an unrelated real multi-component chain on the other side, because
the encoding step maps a literal hyphen WITHIN one component to the same output byte as the
separator BETWEEN two components — the same non-injectivity already documented below for
untruncated paths, now compounded by truncation (`it_117`). Ancestor depth is therefore not the
operative condition; a shared 200-character encoded body is, however it was produced. Closing
either cause would require storing full, untruncated paths — a storage-format (encode-side)
change outside this decode-side algorithm's scope. This disposition, its unclosability proof, and
its pinning regression tests (`it_91`/`it_92`/`it_117`, each asserting the CURRENT, tolerated
inclusion rather than exclusion) are recorded in BUG-520 and BUG-533.

The algorithm assumes the caller's working environment matches the storage origin. This decode
algorithm's full evolution — 25 defects found (23 fixed; 2, BUG-520 and BUG-533, resolved as
accepted architectural limitations rather than fixes) via this session's MAAV adversarial-
verification process — is recorded in `task/claude_storage/bug/completed/509_*.md` through
`533_*.md` (sibling directory to this repo root). Read those for detailed root-cause narratives;
this section states only the CURRENT resulting contract.

### Contract

- **Encode**: replace every `/` and `_` with `-` (see `claude_storage_core`'s own encoding
  algorithm doc, linked above, for the full character-substitution and 200-char-truncation rule)
- **Decode**: real-filesystem-guided, component-by-component candidate enumeration (see
  Disambiguation above) — NOT a simple "first match wins" search; a genuine, irreducible
  ambiguity (`AmbiguousFull`, or `AmbiguousPartial` when the unconsumed remainder itself
  prevents every candidate from reaching Full) is preserved as a candidate SET rather than
  collapsed to an arbitrary or common-ancestor winner, and each caller applies its own
  relationship check per-candidate
- **Round-trip guarantee**: `encode(decode(k)) == k` always holds
- **Inverse guarantee**: `decode(encode(p)) == p` holds only when the filesystem contains
  exactly one candidate matching the encoded key; when 2+ real candidates exist, decode
  correctly reports the ambiguity (`AmbiguousFull`) rather than silently picking one
- **Known accepted imprecision**: when two unrelated paths share an identical first-200-character
  encoded body — whether because a real shared ancestor's OWN pre-truncation encoding
  independently exceeds 200 characters, or because a shallow ancestor's diverging tail collides
  byte-for-byte with an unrelated real chain via a component-boundary escape — decode may
  conservatively over-include the unrelated candidate — proven unclosable by any finite-prefix
  string comparison on the decode side (see Disambiguation above, and BUG-520/BUG-533). This is a
  documented, accepted limitation of the contract, not a violation of it.

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
