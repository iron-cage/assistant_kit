# User Story :: Topic Discovery

Test case spec for [031_topic_discovery.md](../../../../docs/cli/user_story/031_topic_discovery.md).

## Test Case Index

| ID | Test Name | AC | Status |
|----|-----------|-----|-----|
| US-1 | Local topics are listed, sorted by name | AC-1 | ✅ |
| US-2 | `--global` lists the global topic home instead of cwd | AC-2 | ✅ |
| US-3 | `--dir` selects the base and outranks `--global` | AC-3 | ✅ |
| US-4 | SESSIONS reflects real session storage; never-entered reads 0 | AC-4 | ✅ |
| US-5 | An empty base reports on stderr and exits 0 | AC-5 | ✅ |
| US-6 | `--path NAME` resolves one name to its absolute path | AC-6 | ✅ |
| US-7 | Resolving is pure — works for a non-existent topic, creates nothing | AC-7 | ✅ |
| US-8 | The resolved path equals the directory `--topic` actually runs in | AC-8 | ✅ |
| US-9 | A `--path` value containing `/` is rejected | AC-9 | ✅ |
| US-10 | No subprocess is spawned and no directory is created by any form | AC-10 | ✅ |

---

### US-1: Local topics are listed, sorted by name

- **Given:** cwd holds topic directories `-zebra` and `-alpha`
- **When:** `clr topics`
- **Then:** a `NAME SESSIONS PATH` header is printed, followed by exactly 2 rows — `alpha` before `zebra`
- **Exit:** 0
- **Verifies:** AC-1
- **Implemented by:** `topics_command_test.rs::tp01_lists_topics_in_cwd_sorted`

---

### US-2: `--global` lists the global topic home instead of cwd

- **Given:** `CLR_TOPIC_HOME` points at a base holding `-global-only`; cwd holds `-local-only`
- **When:** `clr topics --global`
- **Then:** stdout lists `global-only` and not `local-only`
- **Exit:** 0
- **Verifies:** AC-2
- **Implemented by:** `topics_command_test.rs::tp06_global_lists_topic_home`

---

### US-3: `--dir` selects the base and outranks `--global`

- **Given:** an explicit base holds `-explicit-only`; `CLR_TOPIC_HOME` holds `-global-only`
- **When:** `clr topics --global --dir <base>`
- **Then:** stdout lists `explicit-only` and not `global-only` — an explicit path beats a named default
- **Exit:** 0
- **Verifies:** AC-3
- **Implemented by:** `topics_command_test.rs::tp05_explicit_dir_selects_base`, `topics_command_test.rs::tp07_dir_outranks_global`

---

### US-4: SESSIONS reflects real session storage

- **Given:** `-entered` has one real `<uuid>.jsonl` in its own encoded storage under `CLAUDE_HOME`; `-virgin` has none
- **When:** `clr topics`
- **Then:** `entered`'s SESSIONS column reads `1`; `virgin`'s reads `0`
- **Exit:** 0
- **Verifies:** AC-4
- **Implemented by:** `topics_command_test.rs::tp15_session_count_reflects_real_storage`

---

### US-5: An empty base reports on stderr and exits 0

- **Given:** a base with no topic directories
- **When:** `clr topics`
- **Then:** stdout is empty; stderr contains `no topics in <base>`; the command is safe under `set -e`
- **Exit:** 0
- **Verifies:** AC-5
- **Implemented by:** `topics_command_test.rs::tp02_empty_base_reports_on_stderr_exit_0`

---

### US-6: `--path NAME` resolves one name to its absolute path

- **Given:** cwd is `<base>`
- **When:** `clr topics --path auth-refactor`
- **Then:** stdout is exactly one line, `<base>/-auth-refactor`
- **Exit:** 0
- **Verifies:** AC-6
- **Implemented by:** `topics_command_test.rs::tp08_path_resolves_name_under_cwd`, `topics_command_test.rs::tp10_path_honors_global`

---

### US-7: Resolving is pure

- **Given:** an empty base with no topic of that name
- **When:** `clr topics --path never-created`
- **Then:** the path is printed; the directory does not exist afterwards; the base still holds zero entries
- **Exit:** 0
- **Verifies:** AC-7
- **Implemented by:** `topics_command_test.rs::tp09_path_is_pure_computation`

---

### US-8: The resolved path equals the directory `--topic` actually runs in

- **Given:** `CLR_TOPIC_HOME` points at a fixture home
- **When:** `clr topics --global --path cross-check`, then `clr --dry-run --global --topic cross-check "x"`
- **Then:** the dry-run output contains the exact path the resolver printed — both sides compute it through `claude_topic_core::topic_dir()`
- **Exit:** 0 for both
- **Verifies:** AC-8
- **Implemented by:** `topics_command_test.rs::tp16_path_matches_dry_run_effective_dir`

---

### US-9: A `--path` value containing `/` is rejected

- **Given:** any base
- **When:** `clr topics --path a/b`
- **Then:** stderr explains the single-topic-name constraint; nothing on stdout
- **Exit:** 1
- **Verifies:** AC-9
- **Implemented by:** `topics_command_test.rs::tp11_path_rejects_slash`

---

### US-10: No subprocess is spawned and no directory is created

- **Given:** an empty base
- **When:** `clr topics --path never-created`
- **Then:** the base still holds zero entries afterwards; the command returns without spawning `claude` (no `PATH` stub is needed for any case in this file)
- **Exit:** 0
- **Verifies:** AC-10
- **Implemented by:** `topics_command_test.rs::tp09_path_is_pure_computation`
