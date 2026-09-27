# Type :: `ModelName`

Validation tests for the `ModelName` semantic type (any non-empty string). Tests validate pass-through and missing-value handling.

**Source:** [type/04_model_name.md](../../../../docs/cli/type/04_model_name.md)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| TC-1 | Valid model name → forwarded to claude | Valid Input |
| TC-2 | Model name with hyphens → accepted | Valid Input |
| TC-3 | `--model` without value → exit 1 | Missing Value |
| TC-4 | `--model` absent → built-in `claude-opus-5-5` in assembled command | Default |

## Test Coverage Summary

- Valid Input: 2 tests (TC-1, TC-2)
- Missing Value: 1 test (TC-3)
- Default: 1 test (TC-4)

**Total:** 4 test cases

## Test Cases

---

### TC-1: Valid model name → forwarded

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug" --model sonnet`
- **Then:** Assembled command contains `--model sonnet`; value forwarded verbatim
- **Exit:** 0
- **Source:** [type/04_model_name.md](../../../../docs/cli/type/04_model_name.md)

---

### TC-2: Model name with hyphens → accepted

- **Given:** clean environment
- **When:** `clr --dry-run "Fix bug" --model claude-sonnet-5`
- **Then:** Exit 0; hyphenated model name accepted and forwarded intact
- **Exit:** 0
- **Source:** [type/04_model_name.md](../../../../docs/cli/type/04_model_name.md)

---

### TC-3: `--model` without value → exit 1

- **Given:** clean environment
- **When:** `clr --model`
- **Then:** Exit 1; error indicating `--model` requires a value
- **Exit:** 1
- **Source:** [type/04_model_name.md](../../../../docs/cli/type/04_model_name.md)

---

### TC-4: `--model` absent → built-in `claude-opus-5-5` in assembled command

- **Given:** clean environment (no `CLR_MODEL`, no config-file `model`, no `env.ANTHROPIC_MODEL` in `~/.claude/settings.json`)
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--model claude-opus-5-5` (`DEFAULT_MODEL`); on a non-anthropic seat no `--model` is emitted at all (Provider Gate)
- **Exit:** 0
- **Source:** [type/04_model_name.md](../../../../docs/cli/type/04_model_name.md)
