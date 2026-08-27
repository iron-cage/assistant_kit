# API: Topic Surface

### Scope

- **Purpose**: Pin the signature and behavioral contract of every item `claude_topic_core` exports, so a consumer can depend on it without reading the source.
- **In Scope**: All items re-exported from `lib.rs`, plus the `identity`, `registry`, `enumerate`, `select`, `pool`, and `lock` module paths they are also reachable through.
- **Out of Scope**: Private helpers, which are implementation.

### Errors

The crate exports one error type, `LockDenied`, from one fallible function,
`try_lock`. Everything else is either total or reports absence as `Option::None`.

`validate_prefix` returns `Result< (), String >` — a human-readable reason rather
than a typed error, because its only consumer is a CLI layer printing it back to the
person who typed the prefix.

### `identity`

| Signature | Contract |
|-----------|----------|
| `enum TopicMode { Fork, Dir }` | `Copy`, `Eq`, `Hash`. `as_str()` → `"fork"` / `"dir"`; `Display` and `FromStr` are exact inverses of it |
| `effective_topic_mode( explicit : Option< TopicMode >, global : bool, from : Option< &str >, dir : Option< &str >, topic : &str ) -> TopicMode` | Total. Performs exactly one filesystem probe (rule 4). Precedence in [feature/001](../feature/001_topic_identity.md) |
| `topic_home() -> PathBuf` | Total. `$CLR_TOPIC_HOME` when non-empty, else `<system temp dir>/clr-topic`. Used verbatim — nothing appended |
| `topic_base( dir : Option< &str >, global : bool ) -> PathBuf` | Total. `dir` > `global` > current directory. Falls back to `.` if the current directory is unreadable |
| `topic_dir( base : &Path, name : &str ) -> PathBuf` | Pure. `<base>/-<name>`. The file need not exist |
| `topic_name_of( entry_name : &str ) -> Option< &str >` | Pure. Strips one leading `-`; `None` for a bare `-` or an unprefixed name |
| `fork_session_file( base : &Path, name : &str ) -> Option< PathBuf >` | Canonicalises `base` itself. `None` only when the storage root cannot be resolved or the path is not UTF-8. Pure otherwise — the file need not exist |

**Guarantee — determinism.** `fork_session_file( base, name )` returns the same path
for the same `( base, name )` on every call and in every process, including before
the file exists. Two spellings of one base (a symlink, a `..`) resolve to one path.

### `registry`

| Signature | Contract |
|-----------|----------|
| `record( canonical_base : &Path, topic : &str )` | Append-if-missing. Never panics, never fails: every error path warns on stderr and returns. A `topic` containing `\n` is refused |
| `list( canonical_base : &Path ) -> Vec< String >` | Total. First-recorded order, blank lines skipped. Empty on a missing, unreadable, or unencodable base |

**Caller obligation.** `canonical_base` must already be in canonical physical
absolute form — pass `claude_storage_core::physical_abs( base )`. Neither function
canonicalises, and both key on the bytes they are given.

**Guarantee — non-authoritative.** A listed name asserts nothing about whether a
session exists. See [invariant/001](../invariant/001_registry_non_authoritative.md).

### `enumerate`

| Signature | Contract |
|-----------|----------|
| `struct Topic { name : String, mode : TopicMode, path : PathBuf, sessions : usize }` | `Clone`, `Eq`. `path` is the working directory for `Dir`, the session file for `Fork` |
| `Topic::session_id( &self ) -> Option< String >` | `Some` for `Fork` (the file stem — the id `--resume` takes), `None` for `Dir` |
| `session_count( dir : &Path ) -> usize` | Total. `*.jsonl` files in `dir`'s own session storage; 0 when that storage does not exist |
| `enumerate( base : &Path ) -> Vec< Topic >` | Total. Both mechanisms, sorted by name then by `mode.as_str()`. Empty for a missing base |
| `enumerate_live( base : &Path ) -> Vec< Topic >` | `enumerate` retaining `sessions > 0` — the addressable subset |

**Guarantee — one row per `( name, mode )`.** Never deduped by name. See
[invariant/002](../invariant/002_mode_travels_with_name.md).

### `select`

| Signature | Contract |
|-----------|----------|
| `enum Pick { Idle, Random }` | `Copy`, `Eq`, `Default` (= `Idle`). `as_str()` → `"idle"` / `"random"`; `Display` and `FromStr` are exact inverses |
| `struct Selection< 't > { topic : &'t Topic, all_busy : bool }` | Borrows from the slice passed to `select`/`select_with` |
| `is_busy( topic : &Topic, processes : &[ ProcessInfo ] ) -> bool` | Pure. Fork: the session id appears in some process's `args`. Dir: some process's `cwd` canonicalises to the topic's path |
| `default_seed() -> u64` | Reads the wall clock and this process's id, then mixes. Not cryptographic; not reproducible across calls |
| `select( topics : &[ Topic ], pick : Pick, seed : u64 ) -> Option< Selection< '_ > >` | Scans `/proc` once under `Idle`; no scan under `Random`. `None` only when `topics` is empty |
| `select_with( topics : &'t [ Topic ], pick : Pick, seed : u64, processes : &[ ProcessInfo ] ) -> Option< Selection< 't > >` | Pure. The whole of the selection logic; `select` only supplies the sweep |

**Guarantee — reproducible.** For a fixed `( topics, pick, seed, processes )`,
`select_with` returns the same topic every time. The draw is `seed % candidates.len()`
over the candidate list in `topics` order.

**Guarantee — never empty-handed.** Under `Idle`, a fully-busy candidate set falls
back to the full set with `all_busy = true` rather than returning `None`. `None`
means "no topics", never "no idle topics".

### `pool`

| Signature | Contract |
|-----------|----------|
| `const DEFAULT_PREFIX : &str` | `"t"` |
| `validate_prefix( prefix : &str ) -> Result< (), String >` | Rejects empty, `/`, `\n`, a leading `-`, and a trailing digit. `Err` carries the reason |
| `pool_index( name : &str, prefix : &str ) -> Option< u32 >` | Pure, exact inverse of `format!( "{prefix}{index}" )`. `None` for a leading zero, for `0`, and for any non-matching name |
| `missing_names( existing : &[ Topic ], target : usize, prefix : &str ) -> Vec< String >` | Pure. The names absent from `existing` needed to reach `target`; empty when already met. Gaps before extension; one name per index across both modes |

**Guarantee — idempotent.** Feeding `missing_names`' own output back in as `existing`
(with any modes) yields an empty vector. It reports absences and never deletions.

### `lock`

| Signature | Contract |
|-----------|----------|
| `const LOCK_ENV : &str` | `"CLR_TOPIC_LOCK"` — the run-path opt-in |
| `const LOCK_DIR_ENV : &str` | `"CLR_TOPIC_LOCK_DIR"` — overrides the lock directory |
| `enum LockDenied { Held( u32 ), Unavailable( String ) }` | `Eq`, `Display`. `Held` carries the owning pid |
| `struct TopicLock` | Released on `Drop`; `path()` exposes the backing file. Drop does not run on `SIGKILL` — hence reclaim |
| `enabled_for_run_path() -> bool` | `true` only when `CLR_TOPIC_LOCK` is `"1"` or `"true"` |
| `lock_file( topic : &Topic ) -> Option< PathBuf >` | Pure given the environment. `None` when the topic path cannot be encoded |
| `try_lock( topic : &Topic ) -> Result< TopicLock, LockDenied >` | Never waits, never blocks. Reclaims a lock whose owner is not running, by compare-and-delete |

**Guarantee — advisory only.** Holding a `TopicLock` does not prevent a process that
never called `try_lock` from writing the same session. Two racing reclaims of the
same stale lock can both succeed; that degrades to unlocked behaviour for that
invocation, never to something worse. See [feature/005](../feature/005_topic_lock.md).

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/lib.rs` | The flat re-export surface |
| doc | [feature/001_topic_identity.md](../feature/001_topic_identity.md) | Precedence rationale |
| doc | [feature/003_topic_selection.md](../feature/003_topic_selection.md) | Policy rationale |
| doc | [invariant/002_mode_travels_with_name.md](../invariant/002_mode_travels_with_name.md) | Why `Topic` carries a mode |
| test | `tests/` | One file per module, named for it |
