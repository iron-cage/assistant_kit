# Param :: `--session-from`

Edge case tests for the `--session-from <DIR>` parameter (alias `--from`), which enables session cross-loading by reading the source session from a different directory's Claude session storage.

**Source:** [param/076_session_from.md](../../../../docs/cli/param/076_session_from.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--session-from` exports source `CLAUDE_CODE_SESSION_DIR` + bare `-c` when source has session | Behavioral Divergence |
| EC-2 | `--from` alias behaves identically to `--session-from` | Alias |
| EC-3 | Source dir with no `.jsonl` → no `-c` injected; fresh session | Behavioral Divergence |
| EC-4 | `--session-dir` takes precedence over `--session-from` | Precedence |
| EC-5 | `--new-session` takes precedence over `--session-from` | Precedence |
| EC-6 | `--to` + `--session-from`: Claude runs in target dir, loads from source | Behavioral |
| EC-7 | `CLR_SESSION_FROM` env var equivalent to `--session-from` | EnvFallback |
| EC-8 | `--dry-run` output WYSIWYG-reflects the source `CLAUDE_CODE_SESSION_DIR` | Discovery |
| EC-9 | Relative source path resolves against cwd (physical canonicalization) | PathResolution |
| EC-10 | Empty source value ignored — no export, no `-unknown` fallback dir | Guard |
| EC-11 | Args-file JSON key `session-from` behaves like the CLI flag | ConfigRoute |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-3)
- Behavioral: 1 test (EC-6)
- Alias: 1 test (EC-2)
- Precedence: 2 tests (EC-4, EC-5)
- EnvFallback: 1 test (EC-7)
- Discovery: 1 test (EC-8)
- PathResolution: 1 test (EC-9)
- Guard: 1 test (EC-10)
- ConfigRoute: 1 test (EC-11)

**Total:** 11 edge cases

**Mechanism note:** the runner injects bare `-c` (no UUID argument) and exports `CLAUDE_CODE_SESSION_DIR=<source storage>` to the subprocess — session selection is delegated to claude via that redirect, not via a `-c <uuid>` argument. All Then-clauses below assert on the export line and the bare `-c` flag accordingly.

## Test Cases

---

### EC-1: `--session-from` exports source `CLAUDE_CODE_SESSION_DIR` + bare `-c` when source has session

- **Given:** session storage for source dir `/tmp/076ec1-src` (under a temp `CLAUDE_HOME`) holds `aaa-111.jsonl`; target is CWD
- **When:** `clr --session-from /tmp/076ec1-src --dry-run "Continue"`
- **Then:** dry-run output contains `export CLAUDE_CODE_SESSION_DIR=<claude_home>/projects/-tmp-076ec1-src` and the bare continue flag ` -c "` before the quoted message (no UUID argument — selection is delegated to claude via the env redirect)
- **Exit:** 0
- **Source:** [param/076_session_from.md](../../../../docs/cli/param/076_session_from.md)
- **Implemented by:** `session_from_test.rs::ec1_session_from_injects_continue`

---

### EC-2: `--from` alias behaves identically to `--session-from`

- **Given:** same setup as EC-1 (source dir `/tmp/076ec2-src` storage holds `bbb-222.jsonl`)
- **When:** `clr --from /tmp/076ec2-src --dry-run "Continue"`
- **Then:** dry-run output contains the same `CLAUDE_CODE_SESSION_DIR=<claude_home>/projects/-tmp-076ec2-src` export that `--session-from` would produce
- **Exit:** 0
- **Source:** [param/076_session_from.md](../../../../docs/cli/param/076_session_from.md)
- **Implemented by:** `session_from_test.rs::ec2_from_alias_identical_to_session_from`

---

### EC-3: Source dir with no `.jsonl` → no `-c` injected; fresh session

- **Given:** source dir `/tmp/076ec3-empty-src` has no session storage (no qualifying `.jsonl` files under `CLAUDE_HOME`)
- **When:** `clr --session-from /tmp/076ec3-empty-src --dry-run "Start fresh"`
- **Then:** dry-run output does NOT contain the continue flag ` -c "`; subprocess starts without session continuation
- **Exit:** 0
- **Source:** [param/076_session_from.md](../../../../docs/cli/param/076_session_from.md)
- **Implemented by:** `session_from_test.rs::ec3_empty_source_no_continue`

---

### EC-4: `--session-dir` takes precedence over `--session-from`

- **Given:** source dir `/tmp/076ec4-src` storage holds `ccc-333.jsonl`; raw override dir (a temp dir) holds `xyz-789.jsonl`
- **When:** `clr --session-from /tmp/076ec4-src --session-dir <override> --dry-run "test"`
- **Then:** dry-run output contains `CLAUDE_CODE_SESSION_DIR=<override>` (the raw path verbatim); the computed source storage path does NOT appear anywhere in the output
- **Exit:** 0
- **Source:** [param/076_session_from.md](../../../../docs/cli/param/076_session_from.md)
- **Implemented by:** `session_from_test.rs::ec4_session_dir_wins_over_session_from`

---

### EC-5: `--new-session` takes precedence over `--session-from`

- **Given:** source dir `/tmp/076ec5-src` storage holds `ddd-444.jsonl`
- **When:** `clr --session-from /tmp/076ec5-src --new-session --dry-run "fresh"`
- **Then:** dry-run output does NOT contain the continue flag ` -c "` (`--new-session` suppresses cross-loading); the `CLAUDE_CODE_SESSION_DIR` export is still present (the source path is computed regardless — only continuation is suppressed)
- **Exit:** 0
- **Source:** [param/076_session_from.md](../../../../docs/cli/param/076_session_from.md)
- **Implemented by:** `session_from_test.rs::ec5_new_session_suppresses_session_from`

---

### EC-6: `--to` + `--session-from`: Claude runs in target dir, loads from source

- **Given:** source dir `/tmp/076ec6-src` storage holds `eee-555.jsonl`; target dir (a temp dir) exists
- **When:** `clr --to <target> --session-from /tmp/076ec6-src --dry-run "Continue"`
- **Then:** dry-run output contains `CLAUDE_CODE_SESSION_DIR=<claude_home>/projects/-tmp-076ec6-src` (source storage) and `cd <target>` (subprocess runs in target, not source)
- **Exit:** 0
- **Source:** [param/076_session_from.md](../../../../docs/cli/param/076_session_from.md)
- **Implemented by:** `session_from_test.rs::ec6_to_plus_session_from`

---

### EC-7: `CLR_SESSION_FROM` env var equivalent to `--session-from`

- **Given:** source dir `/tmp/076ec7-src` storage holds `fff-666.jsonl`; `CLR_SESSION_FROM` set to that path; no `--session-from` on CLI
- **When:** `CLR_SESSION_FROM=/tmp/076ec7-src clr --dry-run "Continue"`
- **Then:** dry-run output contains the same `CLAUDE_CODE_SESSION_DIR` export as `--session-from /tmp/076ec7-src` would produce
- **Exit:** 0
- **Source:** [param/076_session_from.md](../../../../docs/cli/param/076_session_from.md)
- **Implemented by:** `session_from_test.rs::ec7_clr_session_from_env_var`

---

### EC-8: `--dry-run` output WYSIWYG-reflects the source `CLAUDE_CODE_SESSION_DIR`

- **Given:** source dir `/tmp/076ec8-src` storage holds `ggg-777.jsonl` (highest mtime)
- **When:** `clr --session-from /tmp/076ec8-src --dry-run "task"`
- **Then:** dry-run output includes the exact `CLAUDE_CODE_SESSION_DIR=<computed source storage>` export the subprocess would receive; WYSIWYG — dry-run accurately reflects the real invocation (`--trace` renders the identical block via the shared `describe_full()` source of truth)
- **Exit:** 0
- **Source:** [param/076_session_from.md](../../../../docs/cli/param/076_session_from.md)
- **Implemented by:** `session_from_test.rs::ec8_dry_run_wysiwyg_session_from`

---

### EC-9: Relative source path resolves against cwd (physical canonicalization)

- **Given:** a real source dir `relsrc/` inside a temp parent dir; session storage seeded for the **canonicalized absolute** form of that dir (`fs::canonicalize`); clr invoked with the temp parent as its working directory
- **When:** `clr --dry-run --session-from ./relsrc "Continue"` (relative value)
- **Then:** dry-run output contains `CLAUDE_CODE_SESSION_DIR=<claude_home>/projects/<Df(canonical absolute path)>` and ` -c "` — an unresolved relative value would instead encode literally (`./relsrc` → `---relsrc`), silently miss the storage dir, and start a fresh session
- **Exit:** 0
- **Source:** [param/076_session_from.md](../../../../docs/cli/param/076_session_from.md)
- **Implemented by:** `session_from_test.rs::ec9_relative_source_path_resolves_against_cwd`

---

### EC-10: Empty source value ignored — no export, no `-unknown` fallback dir

- **Given:** no session storage; `--session-from` given an empty string
- **When:** `clr --session-from "" --dry-run "task"`
- **Then:** dry-run output contains NO `CLAUDE_CODE_SESSION_DIR=` export and no `-unknown` path — an unfiltered empty value would fall through `encode_path()`'s error path into the `-unknown` fallback storage name and actively redirect subprocess session storage there (same empty-is-identity rule as `--subdir ""`)
- **Exit:** 0
- **Source:** [param/076_session_from.md](../../../../docs/cli/param/076_session_from.md)
- **Implemented by:** `session_from_test.rs::ec10_empty_source_value_ignored`

---

### EC-11: Args-file JSON key `session-from` behaves like the CLI flag

- **Given:** source dir `/tmp/076ec11-src` storage holds `hhh-888.jsonl`; an args-file containing `{"session-from": "/tmp/076ec11-src"}`; no `--session-from` on CLI
- **When:** `clr --args-file <file> --dry-run "Continue"`
- **Then:** dry-run output contains the computed source `CLAUDE_CODE_SESSION_DIR` export and ` -c "` — the third input route (args-file JSON) matches the CLI flag and `CLR_SESSION_FROM` env var routes
- **Exit:** 0
- **Source:** [param/076_session_from.md](../../../../docs/cli/param/076_session_from.md)
- **Implemented by:** `session_from_test.rs::ec11_json_config_session_from_key`
