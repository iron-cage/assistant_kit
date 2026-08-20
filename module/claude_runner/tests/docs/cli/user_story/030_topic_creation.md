# User Story :: Topic Creation

Test case spec for [030_topic_creation.md](../../../../docs/cli/user_story/030_topic_creation.md).

## Test Case Index

| ID | Test Name | AC | Status |
|----|-----------|-----|-----|
| US-1 | Auto-named topic generates a slug from the message | AC-001 | ✅ |
| US-2 | Auto-naming disambiguates repeated calls via counter | AC-001 | ✅ |
| US-3 | Explicit `--topic` bypasses slug generation | AC-002 | ✅ |
| US-4 | First use of a topic name clones the source session | AC-003 | ✅ |
| US-5 | Repeat use of the same topic name continues its conversation | AC-004 | ✅ |
| US-6 | `topic` accepts every `run`/`ask` parameter unchanged | AC-006 | ✅ |

---

### US-1: Auto-named topic generates a slug from the message

- **Given:** cwd has no `-investigate-the-flaky` topic directory yet
- **When:** `clr topic --dry-run "Investigate the flaky concurrency-gate test"`
- **Then:** dry-run output includes a path ending in `/-investigate-the-flaky` (or equivalent truncated slug)
- **Exit:** 0
- **Verifies:** AC-001
- **Implemented by:** `topic_command_test.rs::us1_auto_named_topic_generates_slug`

---

### US-2: Auto-naming disambiguates repeated calls via counter

- **Given:** a topic directory matching the message's slug already exists (from a prior real invocation)
- **When:** `clr topic --dry-run "Investigate the flaky concurrency-gate test"` (same message again)
- **Then:** dry-run output includes a path ending in `/-investigate-the-flaky-2`, distinct from the first call's path
- **Exit:** 0
- **Verifies:** AC-001
- **Implemented by:** `topic_command_test.rs::us2_auto_naming_disambiguates_via_counter`

---

### US-3: Explicit `--topic` bypasses slug generation

- **Given:** clean cwd
- **When:** `clr topic --dry-run --topic auth-refactor "Start refactoring the auth module"`
- **Then:** dry-run output includes a path ending in `/-auth-refactor` exactly (no counter suffix, not derived from message text)
- **Exit:** 0
- **Verifies:** AC-002
- **Implemented by:** `topic_command_test.rs::us3_explicit_topic_bypasses_slug_generation`

---

### US-4: First use of a topic name clones the source session

- **Given:** cwd has a qualifying `.jsonl` session file; `-fresh-topic` topic directory does not yet exist
- **When:** `clr topic --dry-run --topic fresh-topic "Start this"`
- **Then:** dry-run output includes a `# session-transplant:` plan line copying cwd's session into the new topic directory's storage
- **Exit:** 0
- **Verifies:** AC-003
- **Implemented by:** `topic_command_test.rs::us4_first_use_clones_source_session`

---

### US-5: Repeat use of the same topic name continues its conversation

- **Given:** `-fresh-topic` topic directory already has its own session file from a prior real (non-dry-run) `clr topic --topic fresh-topic` call
- **When:** `clr topic --dry-run --topic fresh-topic "Continue this"`
- **Then:** dry-run output does NOT include a `# session-transplant:` plan line; output DOES include `-c "`
- **Exit:** 0
- **Verifies:** AC-004
- **Implemented by:** `topic_command_test.rs::us5_repeat_use_continues_conversation`

---

### US-6: `topic` accepts every `run`/`ask` parameter unchanged

- **Given:** clean cwd
- **When:** `clr topic --dry-run --effort high --model sonnet "message"`
- **Then:** dry-run output contains `--effort high` and `--model sonnet`, identical to the equivalent `clr ask` invocation
- **Exit:** 0
- **Verifies:** AC-006
- **Implemented by:** `topic_command_test.rs::us6_full_parameter_inheritance`
