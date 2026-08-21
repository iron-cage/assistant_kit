# Test: `--topic-mode` (fork-mode topics)

Integration test planning for the `--topic-mode` parameter and the fork-mode topic
mechanism it governs. See [param/088_topic_mode.md](../../../../docs/cli/param/088_topic_mode.md)
for the parameter specification and [param/028_topic.md](../../../../docs/cli/param/028_topic.md)
§ Mode selection for the fork/dir decision rules.

This file plans the whole fork-topic suite — mode selection, fork/resume argv shapes,
contradiction guards, registry side effects, and the `topics --file`/listing surface —
because those behaviors are one mechanism switched by this parameter. F15–F17 exercise
the `topics` command surface; [command/12_topics.md](../command/12_topics.md) keeps its
own TP-1..TP-17 plan for the pre-fork behaviors and cross-links here.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| F01 | first use, empty storage → `--session-id <UUIDv5>` alone; no dir created | Arg Shape |
| F02 | first use with base session → `--resume <src> --fork-session --session-id <topic>` | Arg Shape |
| F03 | repeat use → `--resume <topic>` alone; `# topic-resume:` preview | Arg Shape |
| F04 | pre-existing `-<name>` dir → legacy dir mode wins, no fork args | Coexistence |
| F05 | explicit `--topic-mode dir` on fresh topic → dir mode | Mode Selection |
| F06 | `--topic-mode fork` + `--from` → exit 1 contradiction error | Mode Selection |
| F07 | `--topic-mode fork` + `--global` → exit 1 contradiction error | Mode Selection |
| F08 | `--new-session` on repeat topic → exit 1 naming `topics --file` | New Session |
| F09 | `--new-session` on fresh topic → fork source suppressed (`source=fresh`) | New Session |
| F10 | `CLR_TOPIC_MODE=dir` env → dir mode | Mode Selection |
| F11 | `CLR_TOPIC_MODE=fork` overrides a pre-existing `-<name>` dir | Mode Selection |
| F12 | dry-run writes no registry entry and creates no dir | Side Effects |
| F13 | real run passes `--session-id` through argv and records the registry | Registry |
| F14 | print-gated invocation (no message, non-TTY) injects no fork args, no preview | Gating |
| F15 | `topics --file NAME` output == core `topic_session_file` (parity contract) | Parity |
| F16 | `topics --file` guards: slash name, missing value, `--path` exclusivity | Guards |
| F17 | `topics` listing shows fork (registry) and dir (scan) rows with MODE column | Listing |
| F18 | auto-naming skips a candidate whose fork session file already exists | Auto-naming |

## Test Coverage Summary

- Arg Shape: 3 tests (F01–F03)
- Coexistence: 1 test (F04)
- Mode Selection: 5 tests (F05–F07, F10, F11)
- New Session: 2 tests (F08, F09)
- Side Effects: 1 test (F12)
- Registry: 1 test (F13)
- Gating: 1 test (F14)
- Parity: 1 test (F15)
- Guards: 1 test (F16)
- Listing: 1 test (F17)
- Auto-naming: 1 test (F18)

**Total:** 18 tests

**Implemented by:** `tests/topic_fork_test.rs::fork_f01`–`fork_f18`

**Isolation contract:** every case runs via `run_cli_in_dir_isolated` — cwd pinned to a
canonicalized tempdir (the fork rule hashes the CANONICAL physical base, so a symlinked
`/tmp` would silently change every expected UUIDv5), all topic-affecting env scrubbed,
only the vars under test re-added (`CLAUDE_HOME` for storage, `CLR_TOPIC_REGISTRY_DIR`
for the registry). Expected paths are assembled from parts in the test process — never
by calling env-reading helpers whose env differs from the subprocess's.

---

### F01: First use on empty storage creates the deterministic session

- **Given:** empty `CLAUDE_HOME` storage for the base dir; no `-x` dir
- **Command:** `clr --dry-run --topic x "hello"` (cwd = base)
- **Expected behavior:** argv contains `--session-id <UUIDv5(base, "x")>` and none of `--resume`, `--fork-session`, legacy ` -c `; preview line `# topic-fork: topic=x session=<uuid> source=fresh`; no `-x` dir created
- **Exit:** 0
- **Source:** [param/028_topic.md](../../../../docs/cli/param/028_topic.md), [param/088_topic_mode.md](../../../../docs/cli/param/088_topic_mode.md)

---

### F02: First use forks from the latest base session

- **Given:** one qualifying session in the base dir's storage
- **Command:** `clr --dry-run --topic x "hello"`
- **Expected behavior:** argv contains `--resume <src> --fork-session --session-id <topic-uuid>` — the cache-preserving shape (same cwd, forked history); preview names `source=<src>`
- **Exit:** 0
- **Source:** [param/028_topic.md](../../../../docs/cli/param/028_topic.md)

---

### F03: Repeat use resumes the topic session

- **Given:** the topic's own `<uuid>.jsonl` exists non-empty (plus a base session, to prove the repeat check outranks source selection)
- **Command:** `clr --dry-run --topic x "hello"`
- **Expected behavior:** argv is plain `--resume <topic-uuid>` — no `--fork-session`, no `--session-id`; preview switches to `# topic-resume:`
- **Exit:** 0
- **Source:** [param/028_topic.md](../../../../docs/cli/param/028_topic.md)

---

### F04: Pre-existing `-<name>` dir keeps legacy dir mode

- **Given:** `<base>/-x` exists
- **Command:** `clr --dry-run --topic x "hello"`
- **Expected behavior:** no fork args, no `# topic-fork` preview; the working dir moves into `/-x`
- **Exit:** 0
- **Source:** [param/088_topic_mode.md](../../../../docs/cli/param/088_topic_mode.md) (mode rule 4)

---

### F05: Explicit `--topic-mode dir` forces dir mode on a fresh topic

- **Command:** `clr --dry-run --topic x --topic-mode dir "hello"`
- **Expected behavior:** no `# topic-fork` preview; output uses the `/-x` topic dir
- **Exit:** 0
- **Source:** [param/088_topic_mode.md](../../../../docs/cli/param/088_topic_mode.md)

---

### F06: `--topic-mode fork` contradicts `--from`

- **Command:** `clr --dry-run --topic x --topic-mode fork --from <src> "hello"`
- **Expected behavior:** stderr contains `--topic-mode fork cannot be combined with --from` — fork mode stays in the base dir and forks its own storage; a transplant source contradicts it
- **Exit:** 1
- **Source:** [param/088_topic_mode.md](../../../../docs/cli/param/088_topic_mode.md) (contradictions table)

---

### F07: `--topic-mode fork` contradicts `--global`

- **Command:** `clr --dry-run --topic x --topic-mode fork --global "hello"`
- **Expected behavior:** stderr contains `--topic-mode fork cannot be combined with --global`
- **Exit:** 1
- **Source:** [param/088_topic_mode.md](../../../../docs/cli/param/088_topic_mode.md) (contradictions table)

---

### F08: `--new-session` on a repeat fork topic errors

- **Given:** the topic's session file exists non-empty
- **Command:** `clr --dry-run --topic x --new-session "hello"`
- **Expected behavior:** stderr contains `--new-session cannot restart fork-mode topic 'x'` and points at `topics --file` — the topic IS its deterministic session; restarting means deleting that file
- **Exit:** 1
- **Source:** [param/088_topic_mode.md](../../../../docs/cli/param/088_topic_mode.md), [command/12_topics.md](../../../../docs/cli/command/12_topics.md)

---

### F09: `--new-session` on a fresh fork topic suppresses the fork source

- **Given:** a base session exists, the topic's own file does not
- **Command:** `clr --dry-run --topic x --new-session "hello"`
- **Expected behavior:** argv is `--session-id <uuid>` with no `--resume` — "start clean" wins over forking history; preview shows `source=fresh`
- **Exit:** 0
- **Source:** [param/088_topic_mode.md](../../../../docs/cli/param/088_topic_mode.md)

---

### F10: `CLR_TOPIC_MODE=dir` selects dir mode like the CLI flag

- **Command:** `CLR_TOPIC_MODE=dir clr --dry-run --topic x "hello"` (no `--topic-mode` flag)
- **Expected behavior:** no `# topic-fork` preview; output uses the `/-x` topic dir
- **Exit:** 0
- **Source:** [003_env_param.md](../../../../docs/cli/003_env_param.md) (row 66)

---

### F11: Explicit `CLR_TOPIC_MODE=fork` overrides a pre-existing `-<name>` dir

- **Given:** `<base>/-x` exists
- **Command:** `CLR_TOPIC_MODE=fork clr --dry-run --topic x "hello"`
- **Expected behavior:** argv contains `--session-id <uuid>` and a `# topic-fork` preview — explicit mode beats the dir-exists heuristic; the escape hatch for moving an old dir topic to fork mode without deleting the dir first
- **Exit:** 0
- **Source:** [param/088_topic_mode.md](../../../../docs/cli/param/088_topic_mode.md) (precedence chain)

---

### F12: Dry-run is side-effect-free

- **Command:** `clr --dry-run --topic x "hello"` with `CLR_TOPIC_REGISTRY_DIR` pinned to an empty tempdir
- **Expected behavior:** the registry dir stays empty and no `-x` dir is created — the registry write is a run-path effect (BUG-231/319 dry-run purity rule)
- **Exit:** 0
- **Source:** [003_env_param.md](../../../../docs/cli/003_env_param.md) (Env Param 13)

---

### F13: Real run passes `--session-id` through argv and records the registry

- **Given:** a fake `claude` on `PATH` that writes `<storage>/$id.jsonl` for whatever `--session-id` it receives
- **Command:** `clr --max-sessions 0 --topic x "hello"` (real run, no `--dry-run`)
- **Expected behavior:** `<storage>/<topic-uuid>.jsonl` appears (argv wiring proven end-to-end, not just the preview); the registry file `<registry>/<encode_path(base)>` contains the line `x`
- **Exit:** 0
- **Source:** [param/028_topic.md](../../../../docs/cli/param/028_topic.md), [003_env_param.md](../../../../docs/cli/003_env_param.md) (Env Param 13)

---

### F14: Print-gated invocation injects no fork args

- **Command:** `clr --dry-run --topic x` (no message, non-TTY stdin)
- **Expected behavior:** no `--session-id`, no `--resume`, no `# topic-fork` preview — the BUG-426/435 print gate suppresses resume/fork/create for a bare run with nothing to say, and the preview/registry plan is dropped with it
- **Exit:** 0
- **Source:** [param/088_topic_mode.md](../../../../docs/cli/param/088_topic_mode.md)

---

### F15: `topics --file NAME` matches the core rule (parity contract)

- **Command:** `clr topics --file x` (cwd = base)
- **Expected behavior:** stdout is exactly `<CLAUDE_HOME>/projects/<encode_path(base)>/<UUIDv5(base, "x")>.jsonl` + newline — the value `claude_storage_core::topic_session_file` computes. The claude_storage side pins `.session.path path::<base> topic::x` to the same core value (SP-6 in `cli_cmd_session_path_test.rs`), so the two CLIs are byte-identical by transitivity
- **Exit:** 0
- **Source:** [command/12_topics.md](../../../../docs/cli/command/12_topics.md)

---

### F16: `topics --file` guards

- **Commands:** `clr topics --file a/b`; `clr topics --file` (missing value); `clr topics --path x --file x`
- **Expected behavior:** each exits 1 — stderr respectively contains `--file must be a single topic name`, `--file requires a value`, and `mutually exclusive`
- **Exit:** 1 for all three
- **Source:** [command/12_topics.md](../../../../docs/cli/command/12_topics.md)

---

### F17: Listing merges fork (registry) and dir (scan) rows with a MODE column

- **Given:** registry file for the base contains `x`; `<base>/-y` dir exists
- **Command:** `clr topics`
- **Expected behavior:** header shows the MODE column; row `x` lists as `fork` (sessions 0 — a registry entry whose session file was never created still lists; the name stays reserved), row `y` lists as `dir`
- **Exit:** 0
- **Source:** [command/12_topics.md](../../../../docs/cli/command/12_topics.md)

---

### F18: Auto-naming skips a candidate whose fork session file already exists

- **Given:** the base's storage holds a non-empty session file at the `UUIDv5` path for name `orphan-topic` (as a prior `clr --topic orphan-topic` run would leave); no `-orphan-topic` dir, no dir-mode storage
- **Command:** `clr topic --dry-run "orphan topic"`
- **Expected behavior:** the auto-generated slug `orphan-topic` is judged taken by the fork-session freshness signal (the third probe — the only one that can see a fork topic, which creates no directory); preview plans `topic=orphan-topic-2 ` instead. Companion signals: directory existence (`topic_command_test.rs` T02), dir-mode storage (T10/T11)
- **Exit:** 0
- **Source:** [param/028_topic.md](../../../../docs/cli/param/028_topic.md)
