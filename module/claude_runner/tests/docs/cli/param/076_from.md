# Param :: `--from`

Edge case tests for the `--from <DIR>` parameter, which enables session cross-loading by reading the source session from a different directory's Claude session storage. `--from` defaults to the current working directory when omitted (same rule as `--to`/`--dir`).

**Source:** [param/076_from.md](../../../../docs/cli/param/076_from.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `--from` plans a transplant of the source session + bare `-c` when source has session | Behavioral Divergence |
| EC-2 | `--session-from` (pre-rename flag name) is no longer recognized | Rejection |
| EC-3 | Source dir with no `.jsonl` → no `-c` injected, no transplant; fresh session | Behavioral Divergence |
| EC-4 | `--session-dir` no longer suppresses `--from` (deprecated, inert) | Deprecation |
| EC-5 | `--new-session` takes precedence over `--from` | Precedence |
| EC-6 | `--to` + `--from`: Claude runs in target dir, loads from source | Behavioral |
| EC-7 | `CLR_FROM` env var equivalent to `--from` | EnvFallback |
| EC-8 | `--dry-run` output WYSIWYG-previews the transplant plan | Discovery |
| EC-9 | Relative source path resolves against cwd (physical canonicalization) | PathResolution |
| EC-10 | Empty source value ignored — no export, no `-unknown` fallback dir | Guard |
| EC-11 | Args-file JSON key `from` behaves like the CLI flag | ConfigRoute |
| EC-12 | Old `CLR_SESSION_FROM` env var is inert (renamed to `CLR_FROM`) | Rejection |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-3)
- Behavioral: 1 test (EC-6)
- Rejection: 2 tests (EC-2, EC-12)
- Precedence: 1 test (EC-5)
- Deprecation: 1 test (EC-4)
- EnvFallback: 1 test (EC-7)
- Discovery: 1 test (EC-8)
- PathResolution: 1 test (EC-9)
- Guard: 1 test (EC-10)
- ConfigRoute: 1 test (EC-11)

**Total:** 12 edge cases

**Mechanism note:** the runner injects bare `-c` (no UUID argument) and physically copies the source session file into the target's own storage before spawn — claude then continues the transplanted history in place. Under `--dry-run` the copy is previewed as `# session-transplant: <src_file> -> <target_storage_dir>` without being performed. The former mechanism — exporting `CLAUDE_CODE_SESSION_DIR=<source storage>` — is inert on claude 2.x and was deleted (BUG-490); Then-clauses below assert on the plan line and the bare `-c` flag, and negatively on the absence of that export.

## Test Cases

---

### EC-1: `--from` plans a transplant of the source session + bare `-c` when source has session

- **Given:** session storage for source dir `/tmp/076ec1-src` (under a temp `CLAUDE_HOME`) holds `aaa-111.jsonl`; target is CWD
- **When:** `clr --from /tmp/076ec1-src --dry-run "Continue"`
- **Then:** dry-run output contains the transplant plan line prefixed `# session-transplant: <claude_home>/projects/-tmp-076ec1-src/aaa-111.jsonl -> ` (destination = the target's own storage) and the bare continue flag ` -c "` before the quoted message (no UUID argument — claude continues the transplanted file in place); NO `CLAUDE_CODE_SESSION_DIR=` export appears (the env redirect is deleted — BUG-490)
- **Exit:** 0
- **Source:** [param/076_from.md](../../../../docs/cli/param/076_from.md)
- **Implemented by:** `session_from_test.rs::ec1_from_injects_continue`

---

### EC-2: `--session-from` (pre-rename flag name) is no longer recognized

- **Given:** session storage for source dir `/tmp/076ec2-src` holds `bbb-222.jsonl`
- **When:** `clr --session-from /tmp/076ec2-src --dry-run "Continue"`
- **Then:** the process exits non-zero and stderr contains `unknown option: --session-from` — the rename to `--from` is breaking, not an alias (CLAUDE.md "No Backward Compatibility Preservation"); no transplant is planned
- **Exit:** non-zero
- **Source:** [param/076_from.md](../../../../docs/cli/param/076_from.md)
- **Implemented by:** `session_from_test.rs::ec2_session_from_no_longer_recognized`

---

### EC-3: Source dir with no `.jsonl` → no `-c` injected, no transplant; fresh session

- **Given:** source dir `/tmp/076ec3-empty-src` has no session storage (no qualifying `.jsonl` files under `CLAUDE_HOME`)
- **When:** `clr --session-from /tmp/076ec3-empty-src --dry-run "Start fresh"`
- **Then:** dry-run output does NOT contain the continue flag ` -c "` and does NOT contain a `# session-transplant:` plan line; subprocess starts without session continuation
- **Exit:** 0
- **Source:** [param/076_from.md](../../../../docs/cli/param/076_from.md)
- **Implemented by:** `session_from_test.rs::ec3_empty_source_no_continue`

---

### EC-4: `--session-dir` no longer suppresses `--from`

- **Given:** source dir `/tmp/076ec4-src` storage holds `ccc-333.jsonl`; raw override dir (a temp dir) holds `xyz-789.jsonl`
- **When:** `clr --from /tmp/076ec4-src --session-dir <override> --dry-run "test"`
- **Then:** dry-run output contains the `# session-transplant: <source storage>/ccc-333.jsonl -> ` plan prefix exactly as if `--session-dir` were absent (Fix(BUG-493): the raw override is deprecated and inert — claude ignores the `CLAUDE_CODE_SESSION_DIR` export it used to trigger, so it must never suppress `--from`'s transplant); stdout contains NO `CLAUDE_CODE_SESSION_DIR=` export; stderr contains a deprecation warning naming the override value
- **Exit:** 0
- **Source:** [param/076_from.md](../../../../docs/cli/param/076_from.md)
- **Implemented by:** `session_from_test.rs::ec4_session_dir_no_longer_wins_over_from`

---

### EC-5: `--new-session` takes precedence over `--from`

- **Given:** source dir `/tmp/076ec5-src` storage holds `ddd-444.jsonl`
- **When:** `clr --from /tmp/076ec5-src --new-session --dry-run "fresh"`
- **Then:** dry-run output does NOT contain the continue flag ` -c "` (`--new-session` suppresses cross-loading), does NOT contain a `# session-transplant:` plan line, and contains NO `CLAUDE_CODE_SESSION_DIR=` export at all (the former always-exported source path went away with the env-redirect mechanism — BUG-490)
- **Exit:** 0
- **Source:** [param/076_from.md](../../../../docs/cli/param/076_from.md)
- **Implemented by:** `session_from_test.rs::ec5_new_session_suppresses_from`

---

### EC-6: `--to` + `--from`: Claude runs in target dir, loads from source

- **Given:** source dir `/tmp/076ec6-src` storage holds `eee-555.jsonl`; target dir (a temp dir) exists
- **When:** `clr --to <target> --from /tmp/076ec6-src --dry-run "Continue"`
- **Then:** dry-run output contains the full plan line `# session-transplant: <claude_home>/projects/-tmp-076ec6-src/eee-555.jsonl -> <claude_home>/projects/<Df(canonical target)>` (source file into target's own storage) and `cd <target>` (subprocess runs in target, not source)
- **Exit:** 0
- **Source:** [param/076_from.md](../../../../docs/cli/param/076_from.md)
- **Implemented by:** `session_from_test.rs::ec6_to_plus_from`

---

### EC-7: `CLR_FROM` env var equivalent to `--from`

- **Given:** source dir `/tmp/076ec7-src` storage holds `fff-666.jsonl`; `CLR_FROM` set to that path; no `--from` on CLI
- **When:** `CLR_FROM=/tmp/076ec7-src clr --dry-run "Continue"`
- **Then:** dry-run output contains the same `# session-transplant: <source storage>/fff-666.jsonl -> ` plan prefix as `--from /tmp/076ec7-src` would produce; no `CLAUDE_CODE_SESSION_DIR=` export appears
- **Exit:** 0
- **Source:** [param/076_from.md](../../../../docs/cli/param/076_from.md)
- **Implemented by:** `session_from_test.rs::ec7_clr_from_env_var`

---

### EC-8: `--dry-run` output WYSIWYG-previews the transplant plan

- **Given:** source dir `/tmp/076ec8-src` storage holds `ggg-777.jsonl` (highest mtime)
- **When:** `clr --from /tmp/076ec8-src --dry-run "task"`
- **Then:** dry-run output includes the exact `# session-transplant: <src_file> -> <target_storage_dir>` line describing the copy a real run would perform (no copy happens under `--dry-run`); WYSIWYG — dry-run accurately reflects the real invocation (`--trace` renders the identical block via the shared `describe_full()` source of truth)
- **Exit:** 0
- **Source:** [param/076_from.md](../../../../docs/cli/param/076_from.md)
- **Implemented by:** `session_from_test.rs::ec8_dry_run_wysiwyg_from`

---

### EC-9: Relative source path resolves against cwd (physical canonicalization)

- **Given:** a real source dir `relsrc/` inside a temp parent dir; session storage seeded with `rel-901.jsonl` for the **canonicalized absolute** form of that dir (`fs::canonicalize`); clr invoked with the temp parent as its working directory
- **When:** `clr --dry-run --from ./relsrc "Continue"` (relative value)
- **Then:** dry-run output contains the plan line `# session-transplant: <claude_home>/projects/<Df(canonical absolute path)>/rel-901.jsonl -> ` and ` -c "` — an unresolved relative value would instead encode literally (`./relsrc` → `---relsrc`), silently miss the storage dir, and start a fresh session
- **Exit:** 0
- **Source:** [param/076_from.md](../../../../docs/cli/param/076_from.md)
- **Implemented by:** `session_from_test.rs::ec9_relative_source_path_resolves_against_cwd`

---

### EC-10: Empty source value ignored — no export, no `-unknown` fallback dir

- **Given:** no session storage; `--from` given an empty string
- **When:** `clr --from "" --dry-run "task"`
- **Then:** dry-run output contains NO `CLAUDE_CODE_SESSION_DIR=` export, no `# session-transplant:` plan line, and no `-unknown` path — an unfiltered empty value would fall through `encode_path()`'s error path into the `-unknown` fallback storage name (same empty-is-identity rule as `--subdir ""`)
- **Exit:** 0
- **Source:** [param/076_from.md](../../../../docs/cli/param/076_from.md)
- **Implemented by:** `session_from_test.rs::ec10_empty_source_value_ignored`

---

### EC-11: Args-file JSON key `from` behaves like the CLI flag

- **Given:** source dir `/tmp/076ec11-src` storage holds `hhh-888.jsonl`; an args-file containing `{"from": "/tmp/076ec11-src"}`; no `--from` on CLI
- **When:** `clr --args-file <file> --dry-run "Continue"`
- **Then:** dry-run output contains the `# session-transplant: <source storage>/hhh-888.jsonl -> ` plan prefix and ` -c "` — the third input route (args-file JSON) matches the CLI flag and `CLR_FROM` env var routes
- **Exit:** 0
- **Source:** [param/076_from.md](../../../../docs/cli/param/076_from.md)
- **Implemented by:** `session_from_test.rs::ec11_json_config_from_key`

---

### EC-12: Old `CLR_SESSION_FROM` env var is inert (renamed to `CLR_FROM`)

- **Given:** source dir `/tmp/076ec12-old-env-src` storage holds `ec12-old-env.jsonl`; no `--from` on CLI
- **When:** `CLR_SESSION_FROM=/tmp/076ec12-old-env-src clr --dry-run "Continue"` (the pre-rename env var name)
- **Then:** dry-run output does NOT contain a `# session-transplant:` plan line for that source — the old `CLR_SESSION_FROM` name is inert now that the env var is renamed to `CLR_FROM` (a breaking rename, not an alias)
- **Exit:** 0
- **Source:** [param/076_from.md](../../../../docs/cli/param/076_from.md)
- **Implemented by:** `session_from_test.rs::ec12_old_clr_session_from_env_var_inert`
