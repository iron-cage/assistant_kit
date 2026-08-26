# API: Session Surface

### Scope

- **Purpose**: Pin the signature and error contract of every item `claude_session_core` exports.
- **In Scope**: All items re-exported from `lib.rs`, plus the `liveness`, `registry`, and `turn` modules.
- **Out of Scope**: Private helpers (`proc_tgid`, `starttime_from_stat`, `json_u64`).

### Errors

`Result< T > = core::result::Result< T, Error >`.

| Variant | Meaning |
|---------|---------|
| `Error::ReadDir { path, source }` | The sessions directory exists but could not be enumerated |

There is deliberately no variant for a missing directory or an unparseable file — both are ordinary states, not failures. See [feature/001_registry_scan.md](../feature/001_registry_scan.md).

### `registry`

| Signature | Contract |
|-----------|----------|
| `scan( sessions_dir : &Path ) -> Result< Vec< SessionRecord > >` | Every parseable record, sorted by PID. A missing directory yields `Ok( vec![] )`. Fails `ReadDir` only if the directory exists and cannot be read. |
| `scan_live( sessions_dir : &Path ) -> Result< Vec< SessionRecord > >` | `scan` filtered by `is_alive`. |
| `SessionRecord::parse( text : &str ) -> Option< Self >` | `None` for unparseable JSON or a record lacking `pid` or `sessionId`. |
| `SessionRecord::is_alive( &self ) -> bool` | `pid_alive( self.pid, self.proc_start )`. |
| `SessionStatus::from_raw( raw : &str ) -> Self` | `Busy` for `"busy"`, `Idle` for `"idle"`, `Other( raw )` otherwise. |

`SessionRecord` is `#[ non_exhaustive ]` — construct it via `parse`, match it with `..`. Fields:

| Field | Type | Note |
|-------|------|------|
| `pid` | `u32` | Required |
| `session_id` | `String` | Required — the conversation id |
| `cwd` | `PathBuf` | Empty path when absent |
| `proc_start` | `Option< u64 >` | Read from a JSON **string**, then parsed |
| `version`, `kind`, `entrypoint`, `name` | `Option< String >` | |
| `status` | `SessionStatus` | Defaults to `Idle` when absent |
| `updated_at` | `Option< u64 >` | Milliseconds since the Unix epoch |

**Caller obligation:** the sessions directory is a parameter, not resolved internally. This crate does not depend on `claude_core`, so the caller supplies the path — `claude_daemon_core::DaemonPaths::sessions_dir` is the intended source.

### `liveness`

| Signature | Contract |
|-----------|----------|
| `pid_alive( pid : u32, recorded_starttime : Option< u64 > ) -> bool` | All four clauses must hold — see [invariant/001_liveness_four_clauses.md](../invariant/001_liveness_four_clauses.md). `false` on any platform without `/proc`. |
| `proc_starttime( pid : u32 ) -> Option< u64 >` | Field 22 of `/proc/{pid}/stat`, in clock ticks since boot. `None` if unreadable. |

Passing `None` for `recorded_starttime` deliberately weakens the check to clauses (a)–(c); it is not a mismatch.

### `turn`

| Signature | Contract |
|-----------|----------|
| `BG_TASKS_REPORT_RUNNING_ENV : &str` | `"CLAUDE_CODE_BG_TASKS_REPORT_RUNNING"` — set to `"1"` on every session to be observed. |
| `TurnWatcher::new( reporting : BackgroundReporting ) -> Self` | `const`. The caller must state whether the guarantee was applied; it cannot be recovered later. |
| `.observe( &mut self, status : &SessionStatus ) -> Option< TurnEvent >` | Edge-triggered. First call always `None` — see [invariant/002_first_sighting_never_settles.md](../invariant/002_first_sighting_never_settles.md). |
| `.last( &self ) -> Option< &SessionStatus >` | `const`. The most recently observed status. |

`TurnEvent` is `Started`, `Settled`, or `SettledUnverified`. `SettledUnverified` means the transition looked like a boundary but the session was spawned without background-task reporting, so an outstanding background task would look identical — treat as advisory.

### Verification

```bash
cd module/claude_session_core && cargo doc --no-deps --all-features
```

`#![ deny( missing_docs ) ]` is set, so an undocumented public item fails the build.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/lib.rs` | The re-export list this documents |
| doc | [feature/001_registry_scan.md](../feature/001_registry_scan.md) | Behavior behind `scan` |
| doc | [feature/002_turn_detection.md](../feature/002_turn_detection.md) | Behavior behind `TurnWatcher` |
| test | `tests/registry_test.rs` | Parsing and scan semantics |
| test | `tests/liveness_test.rs` | The four clauses |
| test | `tests/turn_test.rs` | Transition table |
