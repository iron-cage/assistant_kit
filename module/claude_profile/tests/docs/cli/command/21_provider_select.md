# Command Tests :: `.provider.select`

### Scope

- **Purpose**: Integration test cases for the `.provider.select` global inference provider selection command.
- **Source**: `docs/cli/command/009_provider.md`, `docs/feature/072_inference_provider_selection.md`
- **Covers**: AC-07 through AC-13, AC-16

### Test Cases

| IT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| IT-01 | AC-07 | No `~/.clr/config.toml` → `provider.select: anthropic` | `t07_provider_select_get_default_anthropic` |
| IT-02 | AC-08 | `config.toml` has `provider` → prints value | `t08_provider_select_set_kimi_persists_and_confirms` (secondary assertion — not a dedicated test) |
| IT-03 | AC-08 | `id::kimi` → file written; stdout `(selected)` | `t08_provider_select_set_kimi_persists_and_confirms` |
| IT-04 | AC-08 | `id::moonshot` → file written | *(no test — coverage gap; "moonshot" appears nowhere in `tests/cli/`)* |
| IT-05 | AC-11 | `reset::1` with selection set → key removed; others preserved; reverts to `anthropic` | `t09_provider_select_reset_preserves_model_key` |
| IT-06 | AC-12 | `reset::1` with no `config.toml` → exits 0 idempotently | `t17_provider_select_reset_without_config_idempotent` |
| IT-07 | AC-08 | `id::VALUE` creates `config.toml` when absent | `t08_provider_select_set_kimi_persists_and_confirms` (implicit — fresh `TempDir`, not asserted as its own scenario) |
| IT-08 | AC-08 | `id::VALUE` on existing `config.toml` (seeded with `.model scope::subprocess`'s `model` key) → both keys preserved | *(no test — coverage gap; `t09` sets `id::kimi` over a pre-seeded `model` key but only asserts exit 0, never reads `config.toml` until after the subsequent `reset::1`)* |
| IT-09 | AC-09/AC-10 | `id::VALUE reset::1` → exits 1 with `mutually exclusive` in stderr | `t11_provider_select_id_and_reset_mutually_exclusive` |
| IT-10 | AC-13 | `format::json` with selection set → JSON output keyed `provider` | `t12_provider_select_json_format` |
| IT-11 | — | `.provider.select` appears in `clp .help` | `dot04_all_visible_commands_present` (`tests/cli/dot_test.rs` — asserts via `clp .`, not `clp .help` literally; equivalent, see `docs/cli/command_group/readme.md`'s Note on Group 15) |
| IT-12 | AC-09 | `id::` (empty) → exits 1 with non-empty required in stderr | `t10_provider_select_empty_id_exits_1` |

### Notes

- All IT cases use a temporary isolated `~/.clr/` directory to avoid touching the real user environment.
- Backing store is `~/.clr/config.toml`'s `provider` key, sharing the same tiered flat-TOML file as `.model scope::subprocess`'s `model` key via `claude_core::toml_io` — the two keys never interact. `t09_provider_select_reset_preserves_model_key` demonstrates non-interaction on the reset path (provider removed, model preserved); no test demonstrates it on the set path (IT-08, below, is a coverage gap).
- Unlike `.model scope::subprocess`'s get-mode `(unset)` sentinel, `.provider.select`'s get mode always resolves to an effective value (`anthropic` when never selected) — the global provider is a standing config scalar, never "nothing selected" (AC-07, AC-16).
- IT-05's real backing test, `t09_provider_select_reset_preserves_model_key`, seeds `config.toml` via `.model scope::subprocess model::claude-opus-4-8` (not a generic `other_key`) before calling `id::kimi` then `reset::1`; verifies `model` is preserved and `provider` is absent (subsequent get shows `anthropic`).
- **Coverage gaps (no backing test as of this writing):** IT-04 (`id::moonshot`) and IT-08 (both keys asserted present simultaneously after a set over a pre-existing `model` key) — see the Test Case Index above and each case's own note. IT-06's former gap was closed by Task 533 (`t17`, 2026-08-19).
- IT-09: does not require file existence — parameter validation fires first.
- IT-11: requires `clp .help` only conceptually — the real test (`dot04_all_visible_commands_present`) asserts via `clp .`, which rewrites to the same listing (no env setup needed either way).
- AC-14/AC-15 (Gate 10 rotation-exclusion behavior under a selected provider) are covered by `tests/docs/algorithm/004_eligibility_gates.md` (AC-08), not here — this file covers only `.provider.select`'s own command-level get/set/reset behavior.

---

### IT-01: Get with no `config.toml` returns `anthropic`

- **Given:** `~/.clr/config.toml` does not exist.
- **When:** `clp .provider.select`
- **Then:** Stdout is `provider.select: anthropic\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `t07_provider_select_get_default_anthropic`
- **Source:** [072_inference_provider_selection.md AC-07](../../../../docs/feature/072_inference_provider_selection.md)

---

### IT-02: Get returns selected provider value

> Covered as a secondary assertion within `t08_provider_select_set_kimi_persists_and_confirms`'s "second get" step (`tests/cli/account_provider_test.rs`), not by a dedicated test.

- **Given:** `~/.clr/config.toml` contains `provider = "kimi"`.
- **When:** `clp .provider.select`
- **Then:** Stdout is `provider.select: kimi\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `t08_provider_select_set_kimi_persists_and_confirms` (secondary assertion)
- **Source:** [072_inference_provider_selection.md AC-08](../../../../docs/feature/072_inference_provider_selection.md)

---

### IT-03: `id::kimi` writes to `config.toml`

> Primary assertion of `t08_provider_select_set_kimi_persists_and_confirms` — the same test also backs IT-02 (secondary assertion) and IT-07 (implicit).

- **Given:** Any state.
- **When:** `clp .provider.select id::kimi`
- **Then:** `~/.clr/config.toml` contains `provider = "kimi"`. Stdout contains `(selected)`. Exits 0.
- **Exit:** 0
- **Source fn:** `t08_provider_select_set_kimi_persists_and_confirms`
- **Source:** [072_inference_provider_selection.md AC-08](../../../../docs/feature/072_inference_provider_selection.md)

---

### IT-04: `id::moonshot` writes to `config.toml`

> **Coverage gap** — no test exercises this scenario. `moonshot` does not appear anywhere in `tests/cli/` (fresh grep, this session: zero hits). Only `id::kimi` is exercised elsewhere (`t08`); this specific value has no backing assertion.

- **Given:** Any state.
- **When:** `clp .provider.select id::moonshot`
- **Then:** `~/.clr/config.toml` contains `provider = "moonshot"`. Exits 0.
- **Exit:** 0
- **Source fn:** *(none — coverage gap)*
- **Source:** [072_inference_provider_selection.md AC-08](../../../../docs/feature/072_inference_provider_selection.md)

---

### IT-05: `reset::1` removes key and preserves others

> Real backing test seeds the "preserved other key" via `.model scope::subprocess model::claude-opus-4-8` (not a generic unrelated key) — see `tests/cli/account_provider_test.rs`'s `t09`.

- **Given:** `~/.clr/config.toml` contains `provider = "kimi"` and `other_key = "val"`.
- **When:** `clp .provider.select reset::1`
- **Then:** `~/.clr/config.toml` no longer contains the `provider` key. `other_key = "val"` is preserved. Stdout is `provider.select: anthropic (reset to default)\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `t09_provider_select_reset_preserves_model_key`
- **Source:** [072_inference_provider_selection.md AC-11](../../../../docs/feature/072_inference_provider_selection.md)

---

### IT-06: `reset::1` with no `config.toml` is idempotent

- **Given:** `~/.clr/config.toml` does not exist (never created in the sandbox).
- **When:** `clp .provider.select reset::1` — twice consecutively.
- **Then:** Both invocations exit 0 with stdout `provider.select: anthropic (reset to default)\n`; neither call creates `config.toml` (reset on an absent config is a no-op write per `toml_io::remove_user_tier`'s NotFound-as-empty semantics).
- **Exit:** 0
- **Source fn:** `t17_provider_select_reset_without_config_idempotent` (in `tests/cli/account_provider_test.rs`)
- **Source:** [072_inference_provider_selection.md AC-12](../../../../docs/feature/072_inference_provider_selection.md)

---

### IT-07: `id::VALUE` creates `config.toml` when absent

> Implicitly exercised by `t08_provider_select_set_kimi_persists_and_confirms` (its `TempDir` is always freshly empty, so the write is necessarily a from-absent creation) but not asserted as its own scenario — no explicit "file did not exist beforehand" check precedes the write.

- **Given:** `~/.clr/config.toml` does not exist.
- **When:** `clp .provider.select id::kimi`
- **Then:** `~/.clr/config.toml` is created; contains `provider = "kimi"`. Exits 0.
- **Exit:** 0
- **Source fn:** `t08_provider_select_set_kimi_persists_and_confirms` (implicit)
- **Source:** [072_inference_provider_selection.md AC-08](../../../../docs/feature/072_inference_provider_selection.md)

---

### IT-08: `id::VALUE` preserves `.model scope::subprocess`'s key

> **Coverage gap** — no test asserts this directly. `t09_provider_select_reset_preserves_model_key` sets `model` (via `.model scope::subprocess`) then `id::kimi` (asserting only exit 0, not file contents) before immediately testing `reset::1` — the "both keys present simultaneously" state this case describes is never read from `config.toml` or asserted.

- **Given:** `~/.clr/config.toml` contains `model = "claude-opus-4-8"`.
- **When:** `clp .provider.select id::kimi`
- **Then:** `~/.clr/config.toml` contains both `model = "claude-opus-4-8"` and `provider = "kimi"`. Exits 0.
- **Exit:** 0
- **Source fn:** *(none — coverage gap)*
- **Source:** [072_inference_provider_selection.md AC-08](../../../../docs/feature/072_inference_provider_selection.md)

---

### IT-09: `id::VALUE reset::1` exits 1 with mutual exclusion message

- **Given:** Any environment.
- **When:** `clp .provider.select id::kimi reset::1`
- **Then:** Exits 1. Stderr contains `mutually exclusive`.
- **Exit:** 1
- **Source fn:** `t11_provider_select_id_and_reset_mutually_exclusive`
- **Source:** [072_inference_provider_selection.md AC-09](../../../../docs/feature/072_inference_provider_selection.md), [AC-10](../../../../docs/feature/072_inference_provider_selection.md)

---

### IT-10: `format::json` returns JSON output keyed `provider`

- **Given:** `~/.clr/config.toml` contains `provider = "kimi"`.
- **When:** `clp .provider.select format::json`
- **Then:** Stdout is `{"provider":"kimi"}` (or valid JSON equivalent). Exits 0.
- **Variant:** When never selected → `{"provider":"anthropic"}` (never `null` — distinct from `.model scope::subprocess`'s unset-is-null behavior).
- **Exit:** 0
- **Source fn:** `t12_provider_select_json_format`
- **Source:** [072_inference_provider_selection.md AC-13](../../../../docs/feature/072_inference_provider_selection.md)

---

### IT-11: `.provider.select` appears in `clp .help`

> Real backing test (`tests/cli/dot_test.rs`) asserts via `clp .`, not `clp .help` literally — `dot01_dot_and_help_byte_identical` (same file) establishes the two are byte-identical, so the assertion carries over.

- **Given:** Any environment.
- **When:** `clp .help`
- **Then:** Output contains `.provider.select`. Exits 0.
- **Exit:** 0
- **Source fn:** `dot04_all_visible_commands_present`
- **Source:** [009_provider.md](../../../../docs/cli/command/009_provider.md)

---

### IT-12: Empty `id::` exits 1

- **Given:** Any environment.
- **When:** `clp .provider.select id::`
- **Then:** Exits 1. Stderr indicates `id::` must be non-empty.
- **Exit:** 1
- **Source fn:** `t10_provider_select_empty_id_exits_1`
- **Source:** [072_inference_provider_selection.md AC-09](../../../../docs/feature/072_inference_provider_selection.md)
