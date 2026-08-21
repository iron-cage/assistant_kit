# Parameter :: `--keep-clone`

Edge case coverage for the `--keep-clone` flag. See [089_keep_clone.md](../../../../docs/cli/param/089_keep_clone.md) for specification.

Implemented KC-1/KC-2/KC-3 live in `tests/topic_command_test.rs` (T05/T12/T13) — they ride the
`topic --from` transplant fixture because that is the only code path where the collision the flag
governs can occur. KC-5 is pinned by the same file's T04 (first-clone case).

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| KC-1 | Default (no `--keep-clone`) + non-empty same-uuid destination → overwritten with fresh source copy, announced | Behavioral Divergence |
| KC-2 | `--keep-clone` + non-empty same-uuid destination → preserved byte-for-byte (mtime refresh only), announced | Behavioral Divergence |
| KC-3 | `CLR_KEEP_CLONE=1` → same preserve outcome as KC-2 with no CLI flag | Env Fallback |
| KC-4 | `--quiet` suppresses both announcement messages (behavior unchanged) | Interaction |
| KC-5 | Destination missing or empty → plain copy, no announcement, flag irrelevant | Edge Case |
| KC-6 | `--help` output contains `--keep-clone` | Documentation |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (KC-1, KC-2)
- Env Fallback: 1 test (KC-3)
- Interaction: 1 test (KC-4)
- Edge Case: 1 test (KC-5)
- Documentation: 1 test (KC-6)

**Total:** 6 edge cases

---

### KC-1: Default → stale destination copy re-cloned from source

- **Given:** source storage has session `<uuid>.jsonl` with seed content; target storage already
  holds a DIVERGED `<uuid>.jsonl` (seed + extra turn) from a prior clone
- **When:** `clr topic --topic NAME --from <SRC> "msg"` (real run, stubbed `claude`), no `--keep-clone`
- **Then:** destination file content equals the SOURCE content (diverged turn gone); stderr contains
  `re-cloning over existing session copy`; continuation still injected (` -c "` present)
- **Exit:** 0
- **Source:** [--keep-clone](../../../../docs/cli/param/089_keep_clone.md), [--from](../../../../docs/cli/param/076_from.md) step 7
- **Commands:** run, ask, topic
- **Implemented by:** `t05_second_explicit_from_call_recopies_existing_destination` (`tests/topic_command_test.rs`)

---

### KC-2: `--keep-clone` → diverged destination preserved

- **Given:** same fixture as KC-1
- **When:** same invocation plus `--keep-clone`
- **Then:** destination file content is byte-identical to the pre-run diverged content; stderr
  contains `kept existing session copy`; continuation still injected (` -c "` present)
- **Exit:** 0
- **Source:** [--keep-clone](../../../../docs/cli/param/089_keep_clone.md)
- **Commands:** run, ask, topic
- **Implemented by:** `t12_keep_clone_flag_preserves_existing_destination` (`tests/topic_command_test.rs`)

---

### KC-3: `CLR_KEEP_CLONE=1` env fallback

- **Given:** same fixture as KC-1; env `CLR_KEEP_CLONE=1`; no `--keep-clone` on CLI
- **When:** same invocation as KC-1
- **Then:** same outcome as KC-2 — destination preserved, `kept existing session copy` announced
- **Exit:** 0
- **Source:** [--keep-clone](../../../../docs/cli/param/089_keep_clone.md), [003_env_param.md](../../../../docs/cli/003_env_param.md) row 67
- **Commands:** run, ask, topic
- **Implemented by:** `t13_keep_clone_env_preserves_existing_destination` (`tests/topic_command_test.rs`)

---

### KC-4: `--quiet` suppresses the announcements

- **Given:** same fixture as KC-1
- **When:** KC-1's invocation (and separately KC-2's) with `--quiet` added
- **Then:** file outcome unchanged from KC-1/KC-2 respectively; neither
  `re-cloning over existing session copy` nor `kept existing session copy` appears on stderr
- **Exit:** 0
- **Source:** [--keep-clone](../../../../docs/cli/param/089_keep_clone.md), [--quiet](../../../../docs/cli/param/074_quiet.md)
- **Commands:** run, ask, topic

---

### KC-5: Missing/empty destination → plain copy, flag irrelevant

- **Given:** source storage has a qualifying session; target storage has NO file (or an empty file)
  under that uuid
- **When:** `clr topic --topic NAME --from <SRC> "msg"` with or without `--keep-clone`
- **Then:** source copied into target storage; no collision announcement on stderr
- **Exit:** 0
- **Source:** [--keep-clone](../../../../docs/cli/param/089_keep_clone.md)
- **Commands:** run, ask, topic
- **Implemented by:** `t04_first_explicit_topic_call_clones_session` (`tests/topic_command_test.rs`, missing-file case)

---

### KC-6: `--help` lists `--keep-clone`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** stdout contains `--keep-clone` and names `CLR_KEEP_CLONE`
- **Exit:** 0
- **Source:** [command/02_help.md](../../../../docs/cli/command/02_help.md)
- **Commands:** run, ask, topic
