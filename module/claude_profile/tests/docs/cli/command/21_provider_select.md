# Command Tests :: `.provider.select`

### Scope

- **Purpose**: Integration test cases for the `.provider.select` global inference provider selection command.
- **Source**: `docs/cli/command/009_provider.md`, `docs/feature/072_inference_provider_selection.md`
- **Covers**: AC-07 through AC-13, AC-16

### Test Cases

| IT | AC | Scenario | Source fn |
|----|----|----------|-----------|
| IT-01 | AC-07 | No `~/.clr/config.toml` → `provider.select: anthropic` | `it01_get_default_no_file` |
| IT-02 | AC-08 | `config.toml` has `provider` → prints value | `it02_get_shows_selected_value` |
| IT-03 | AC-08 | `id::kimi` → file written; stdout `(selected)` | `it03_set_kimi_selects_provider` |
| IT-04 | AC-08 | `id::moonshot` → file written | `it04_set_moonshot_selects_provider` |
| IT-05 | AC-11 | `reset::1` with selection set → key removed; others preserved; reverts to `anthropic` | `it05_reset_removes_key_preserves_others` |
| IT-06 | AC-12 | `reset::1` with no `config.toml` → exits 0 idempotently | `it06_reset_no_file_is_idempotent` |
| IT-07 | AC-08 | `id::VALUE` creates `config.toml` when absent | `it07_set_creates_file_when_absent` |
| IT-08 | AC-08 | `id::VALUE` on existing `config.toml` (seeded with `.model scope::subprocess`'s `model` key) → both keys preserved | `it08_set_preserves_model_select_key` |
| IT-09 | AC-09/AC-10 | `id::VALUE reset::1` → exits 1 with `mutually exclusive` in stderr | `it09_id_and_reset_mutual_exclusive` |
| IT-10 | AC-13 | `format::json` with selection set → JSON output keyed `provider` | `it10_get_json_format` |
| IT-11 | — | `.provider.select` appears in `clp .help` | `it11_provider_select_in_help` |
| IT-12 | AC-09 | `id::` (empty) → exits 1 with non-empty required in stderr | `it12_empty_id_exits_1` |

### Notes

- All IT cases use a temporary isolated `~/.clr/` directory to avoid touching the real user environment.
- Backing store is `~/.clr/config.toml`'s `provider` key, sharing the same tiered flat-TOML file as `.model scope::subprocess`'s `model` key via `claude_core::toml_io` — the two keys never interact (IT-08 asserts this directly).
- Unlike `.model scope::subprocess`'s get-mode `(unset)` sentinel, `.provider.select`'s get mode always resolves to an effective value (`anthropic` when never selected) — the global provider is a standing config scalar, never "nothing selected" (AC-07, AC-16).
- IT-05: seed `config.toml` with `provider = "kimi"` and `other_key = "val"` before calling `reset::1`; verify `other_key` is preserved and `provider` is absent (subsequent get shows `anthropic`).
- IT-08: seed `config.toml` with `model = "claude-opus-4-8"` (from `.model scope::subprocess`); call `id::kimi`; verify both `model` and `provider` keys present with their respective values.
- IT-09: does not require file existence — parameter validation fires first.
- IT-11: requires `clp .help` only (no env setup needed).
- AC-14/AC-15 (Gate 10 rotation-exclusion behavior under a selected provider) are covered by `tests/docs/algorithm/004_eligibility_gates.md` (AC-08), not here — this file covers only `.provider.select`'s own command-level get/set/reset behavior.

---

### IT-01: Get with no `config.toml` returns `anthropic`

- **Given:** `~/.clr/config.toml` does not exist.
- **When:** `clp .provider.select`
- **Then:** Stdout is `provider.select: anthropic\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `it01_get_default_no_file`
- **Source:** [072_inference_provider_selection.md AC-07](../../../../docs/feature/072_inference_provider_selection.md)

---

### IT-02: Get returns selected provider value

- **Given:** `~/.clr/config.toml` contains `provider = "kimi"`.
- **When:** `clp .provider.select`
- **Then:** Stdout is `provider.select: kimi\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `it02_get_shows_selected_value`
- **Source:** [072_inference_provider_selection.md AC-08](../../../../docs/feature/072_inference_provider_selection.md)

---

### IT-03: `id::kimi` writes to `config.toml`

- **Given:** Any state.
- **When:** `clp .provider.select id::kimi`
- **Then:** `~/.clr/config.toml` contains `provider = "kimi"`. Stdout contains `(selected)`. Exits 0.
- **Exit:** 0
- **Source fn:** `it03_set_kimi_selects_provider`
- **Source:** [072_inference_provider_selection.md AC-08](../../../../docs/feature/072_inference_provider_selection.md)

---

### IT-04: `id::moonshot` writes to `config.toml`

- **Given:** Any state.
- **When:** `clp .provider.select id::moonshot`
- **Then:** `~/.clr/config.toml` contains `provider = "moonshot"`. Exits 0.
- **Exit:** 0
- **Source fn:** `it04_set_moonshot_selects_provider`
- **Source:** [072_inference_provider_selection.md AC-08](../../../../docs/feature/072_inference_provider_selection.md)

---

### IT-05: `reset::1` removes key and preserves others

- **Given:** `~/.clr/config.toml` contains `provider = "kimi"` and `other_key = "val"`.
- **When:** `clp .provider.select reset::1`
- **Then:** `~/.clr/config.toml` no longer contains the `provider` key. `other_key = "val"` is preserved. Stdout is `provider.select: anthropic (reset to default)\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `it05_reset_removes_key_preserves_others`
- **Source:** [072_inference_provider_selection.md AC-11](../../../../docs/feature/072_inference_provider_selection.md)

---

### IT-06: `reset::1` with no `config.toml` is idempotent

- **Given:** `~/.clr/config.toml` does not exist.
- **When:** `clp .provider.select reset::1`
- **Then:** Stdout is `provider.select: anthropic (reset to default)\n`. Exits 0.
- **Exit:** 0
- **Source fn:** `it06_reset_no_file_is_idempotent`
- **Source:** [072_inference_provider_selection.md AC-12](../../../../docs/feature/072_inference_provider_selection.md)

---

### IT-07: `id::VALUE` creates `config.toml` when absent

- **Given:** `~/.clr/config.toml` does not exist.
- **When:** `clp .provider.select id::kimi`
- **Then:** `~/.clr/config.toml` is created; contains `provider = "kimi"`. Exits 0.
- **Exit:** 0
- **Source fn:** `it07_set_creates_file_when_absent`
- **Source:** [072_inference_provider_selection.md AC-08](../../../../docs/feature/072_inference_provider_selection.md)

---

### IT-08: `id::VALUE` preserves `.model scope::subprocess`'s key

- **Given:** `~/.clr/config.toml` contains `model = "claude-opus-4-8"`.
- **When:** `clp .provider.select id::kimi`
- **Then:** `~/.clr/config.toml` contains both `model = "claude-opus-4-8"` and `provider = "kimi"`. Exits 0.
- **Exit:** 0
- **Source fn:** `it08_set_preserves_model_select_key`
- **Source:** [072_inference_provider_selection.md AC-08](../../../../docs/feature/072_inference_provider_selection.md)

---

### IT-09: `id::VALUE reset::1` exits 1 with mutual exclusion message

- **Given:** Any environment.
- **When:** `clp .provider.select id::kimi reset::1`
- **Then:** Exits 1. Stderr contains `mutually exclusive`.
- **Exit:** 1
- **Source fn:** `it09_id_and_reset_mutual_exclusive`
- **Source:** [072_inference_provider_selection.md AC-09](../../../../docs/feature/072_inference_provider_selection.md), [AC-10](../../../../docs/feature/072_inference_provider_selection.md)

---

### IT-10: `format::json` returns JSON output keyed `provider`

- **Given:** `~/.clr/config.toml` contains `provider = "kimi"`.
- **When:** `clp .provider.select format::json`
- **Then:** Stdout is `{"provider":"kimi"}` (or valid JSON equivalent). Exits 0.
- **Variant:** When never selected → `{"provider":"anthropic"}` (never `null` — distinct from `.model scope::subprocess`'s unset-is-null behavior).
- **Exit:** 0
- **Source fn:** `it10_get_json_format`
- **Source:** [072_inference_provider_selection.md AC-13](../../../../docs/feature/072_inference_provider_selection.md)

---

### IT-11: `.provider.select` appears in `clp .help`

- **Given:** Any environment.
- **When:** `clp .help`
- **Then:** Output contains `.provider.select`. Exits 0.
- **Exit:** 0
- **Source fn:** `it11_provider_select_in_help`
- **Source:** [009_provider.md](../../../../docs/cli/command/009_provider.md)

---

### IT-12: Empty `id::` exits 1

- **Given:** Any environment.
- **When:** `clp .provider.select id::`
- **Then:** Exits 1. Stderr indicates `id::` must be non-empty.
- **Exit:** 1
- **Source fn:** `it12_empty_id_exits_1`
- **Source:** [072_inference_provider_selection.md AC-09](../../../../docs/feature/072_inference_provider_selection.md)
