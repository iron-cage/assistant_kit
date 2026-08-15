# Format 001: text

FM test cases for `docs/cli/format/001_text.md`. Verifies the `format::text` (default)
output contract: labeled key-value structure, default-when-omitted behavior, scope across
all format-capable commands, and column alignment for `.usage` table output.

**Source:** [docs/cli/format/001_text.md](../../../../docs/cli/format/001_text.md)

### FM Case Index

| ID | Short Name | Category | Status |
|----|------------|----------|--------|
| FM-1 | `format::text` is the default — omitting `format::` produces text output | Default Behavior | ✅ |
| FM-2 | `.paths` text output: labeled key-value pairs, one field per line | Structure | ✅ |
| FM-3 | `.usage` text footer shows the session model label (`· sonnet` / `· opus`) | Structure | ✅ |
| FM-4 | Text output differs from JSON output (behavioral divergence) | Behavioral Divergence | ✅ |

**Behavioral Divergence Pair:** FM-1 (format::text, omitted) ↔ FM-4 (format::json) — text output produces human-readable labeled key-value lines; JSON output produces a single-line machine-parseable object. Structurally incompatible outputs from the same command.

---

### FM-1: `format::text` is the default — omitting `format::` produces labeled text output

- **Given:** A clp command that supports format selection (`.paths`, `.accounts`, `.usage`, etc.) invoked without any `format::` parameter
- **When:** The command runs
- **Then:** Output is human-readable labeled key-value text (e.g., `Label:   value`), NOT JSON or table — the `format::text` mode is active by default
- **Source fn:** `p02_paths_text_v1_labeled` (cli/token_paths_test.rs)
- **Source:** [docs/cli/format/001_text.md §Trigger](../../../../docs/cli/format/001_text.md)

---

### FM-2: `.paths` text output shows one labeled field per line

- **Given:** `.paths` invoked without format parameter
- **When:** Output is captured
- **Then:** Each path appears on its own line with a label (e.g., `Claude JSON:     /home/user/.claude.json`), padded to align values — the `data_fmt` text renderer is used
- **Source fn:** `p02_paths_text_v1_labeled` (cli/token_paths_test.rs)
- **Source:** [docs/cli/format/001_text.md §Structure](../../../../docs/cli/format/001_text.md)

---

### FM-3: `.usage` text footer shows the session model label (`· sonnet` / `· opus`)

- **Given:** Two accounts built directly via `render_text()` (no CLI): one `is_current`, one candidate whose `seven_day_sonnet.utilization` is set to 50.0 (scenario 1: sonnet_left = 50% ≥ 10%) or 91.0 (scenario 2: sonnet_left = 9% < 10%, override fires)
- **When:** `render_text()` output is captured for each scenario
- **Then:** Output contains `· sonnet` in scenario 1 and `· opus` in scenario 2 — only the footer's model-label substring is asserted; the test makes no header-row, data-row-count, or `Next` recommendation-line assertion
- **Source fn:** `test_ft28_009_footer_model_label` (usage/mod_tests.rs)
- **Source:** [docs/cli/format/001_text.md §Structure](../../../../docs/cli/format/001_text.md)

---

### FM-4: Text output is structurally different from JSON output (behavioral divergence)

- **Given:** `.accounts` invoked without `format::` (text) in one test, and separately with `format::json` in another test
- **When:** Each output is captured
- **Then:** Text output contains labeled lines `Active:  yes` / `Active:  no`; JSON output starts with `[`, confirming a single-line JSON array — the two formats are structurally incompatible, proving `format::` controls output structure meaningfully
- **Source fn:** `acc02_active_shows_yes_inactive_shows_no` (text half) and `acc09_json_format_array` (JSON half) (both cli/accounts_list_test.rs)
- **Source:** [docs/cli/format/001_text.md §Scope](../../../../docs/cli/format/001_text.md)
