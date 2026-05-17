# Env Param :: CLR_* Input Variables

Edge cases for the 24 `CLR_*` input environment variable fallbacks.
Source: [`env_param.md`](../../../../docs/cli/env_param.md)
Implementation: `apply_env_vars()` and `apply_isolated_env_vars()` in `src/lib.rs`
Test file: `tests/env_var_test.rs`

## Test Case Index

| ID | Test | Variable | Primary Assertion |
|----|------|----------|-------------------|
| E01 | `CLR_MESSAGE` supplies prompt text | `CLR_MESSAGE` | stdout contains message text from env |
| E02 | `CLR_PRINT` enables print mode | `CLR_PRINT` | `--print` appears in assembled command |
| E03 | `CLR_MODEL` sets model | `CLR_MODEL` | `--model sonnet` appears; CLI wins over env |
| E04 | `CLR_VERBOSE` enables verbose | `CLR_VERBOSE` | `--verbose` appears; `yes` does NOT activate |
| E05 | `CLR_NO_SKIP_PERMISSIONS` suppresses default | `CLR_NO_SKIP_PERMISSIONS` | `--dangerously-skip-permissions` absent |
| E06 | `CLR_INTERACTIVE` suppresses auto-print | `CLR_INTERACTIVE` | `--print` absent despite message present |
| E07 | `CLR_NEW_SESSION` suppresses `-c` | `CLR_NEW_SESSION` | ` -c` absent in assembled command |
| E08 | `CLR_DIR` sets working directory | `CLR_DIR` | dir path appears in assembled command |
| E09 | `CLR_MAX_TOKENS` sets limit | `CLR_MAX_TOKENS` | token value appears in assembled command |
| E10 | `CLR_SESSION_DIR` sets session dir | `CLR_SESSION_DIR` | session dir path appears in assembled command |
| E11 | `CLR_DRY_RUN` enables preview | `CLR_DRY_RUN` | exit 0 and command printed without execution |
| E12 | `CLR_VERBOSITY=5` triggers verbose detail | `CLR_VERBOSITY` | assembled command preview appears in stderr |
| E13 | `CLR_TRACE` prints command to stderr | `CLR_TRACE` | assembled command preview appears in stderr |
| E14 | `CLR_NO_ULTRATHINK` suppresses suffix | `CLR_NO_ULTRATHINK` | `ultrathink` absent from assembled command |
| E15 | `CLR_SYSTEM_PROMPT` sets system prompt | `CLR_SYSTEM_PROMPT` | `--system-prompt` appears in assembled command |
| E16 | `CLR_APPEND_SYSTEM_PROMPT` appends | `CLR_APPEND_SYSTEM_PROMPT` | `--append-system-prompt` appears |
| E17 | `CLR_EFFORT=low` sets effort level | `CLR_EFFORT` | `low` (not `max`) in assembled command |
| E18 | `CLR_NO_EFFORT_MAX` suppresses default | `CLR_NO_EFFORT_MAX` | `--effort` absent from assembled command |
| E19 | `CLR_NO_CHROME` suppresses default | `CLR_NO_CHROME` | `--chrome` absent from assembled command |
| E20 | `CLR_NO_PERSIST` adds no-session-persistence | `CLR_NO_PERSIST` | `--no-session-persistence` in assembled command |
| E21 | `CLR_JSON_SCHEMA` sets schema | `CLR_JSON_SCHEMA` | `--json-schema` appears in assembled command |
| E22 | `CLR_MCP_CONFIG` adds config path | `CLR_MCP_CONFIG` | `--mcp-config` and path appear in assembled command |
| E23 | `CLR_CREDS` supplies isolated creds path | `CLR_CREDS` | "missing required argument: --creds" NOT in stderr |
| E24 | `CLR_TIMEOUT` sets isolated timeout | `CLR_TIMEOUT` | argument parsing succeeds with `CLR_CREDS+CLR_TIMEOUT` |

## Test Coverage Summary

- Bool vars (truthy only): E02, E04, E05, E06, E07, E11, E13, E14, E18, E19 (10 tests)
- String vars: E01, E03, E08, E10, E15, E16, E21, E22, E23 (9 tests)
- Parsed vars (with silent-ignore): E09, E12, E17 (3 tests)
- Negation suppression (suppress default injection): E05, E06, E07, E18, E19 (5 tests)
- CLI-wins verification: E01, E03 (2 tests)
- Isolated subcommand: E23, E24 (2 tests)

**Total:** 24 edge cases (E01–E24)

## Test Cases

---

### E01: CLR_MESSAGE supplies prompt text

- **Given:** no positional arg; `CLR_MESSAGE=hello world`
- **When:** `clr --dry-run`
- **Then:** stdout contains `hello world`
- **Exit:** 0
- **CLI-wins:** `clr --dry-run cli_msg` with `CLR_MESSAGE=env_msg` → stdout contains `cli_msg`, NOT `env_msg`
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E02: CLR_PRINT enables print mode

- **Given:** `--interactive` on CLI (suppresses auto-print); `CLR_PRINT=1`
- **When:** `clr --dry-run --interactive x`
- **Then:** stdout contains `--print`
- **Exit:** 0
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E03: CLR_MODEL sets model; CLI wins

- **Given:** `CLR_MODEL=sonnet`; no `--model` on CLI
- **When:** `clr --dry-run task`
- **Then:** stdout contains `--model` and `sonnet`
- **Exit:** 0
- **CLI-wins:** `clr --dry-run --model opus task` with `CLR_MODEL=sonnet` → stdout contains `opus`, NOT `sonnet`
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E04: CLR_VERBOSE enables verbose; "yes" does not

- **Given:** `CLR_VERBOSE=1`
- **When:** `clr --dry-run task`
- **Then:** stdout contains `--verbose`
- **Bool negative:** `CLR_VERBOSE=yes` → `--verbose` must NOT appear (only `1`/`true` are truthy)
- **Exit:** 0
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E05: CLR_NO_SKIP_PERMISSIONS suppresses default

- **Given:** `CLR_NO_SKIP_PERMISSIONS=1`
- **When:** `clr --dry-run task`
- **Then:** stdout does NOT contain `--dangerously-skip-permissions`
- **Exit:** 0
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E06: CLR_INTERACTIVE suppresses auto-print injection

- **Given:** `CLR_INTERACTIVE=1`; positional message present
- **When:** `clr --dry-run task`
- **Then:** stdout does NOT contain `--print`
- **Exit:** 0
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E07: CLR_NEW_SESSION suppresses default -c

- **Given:** `CLR_NEW_SESSION=1`
- **When:** `clr --dry-run task`
- **Then:** stdout does NOT contain ` -c`
- **Exit:** 0
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E08: CLR_DIR sets working directory

- **Given:** `CLR_DIR=/tmp/e08dir`
- **When:** `clr --dry-run task`
- **Then:** stdout contains `/tmp/e08dir`
- **Exit:** 0
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E09: CLR_MAX_TOKENS sets token limit

- **Given:** `CLR_MAX_TOKENS=3000`
- **When:** `clr --dry-run task`
- **Then:** stdout contains `3000`
- **Exit:** 0
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E10: CLR_SESSION_DIR sets session directory

- **Given:** `CLR_SESSION_DIR=/tmp/e10sess`
- **When:** `clr --dry-run task`
- **Then:** stdout contains `/tmp/e10sess`
- **Exit:** 0
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E11: CLR_DRY_RUN enables dry-run without CLI flag

- **Given:** `CLR_DRY_RUN=1`; no `--dry-run` on CLI
- **When:** `clr task`
- **Then:** exit 0; stdout contains assembled command (contains `--effort`)
- **Rationale:** without env var the process attempts claude execution (non-0 in test env)
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E12: CLR_VERBOSITY=5 triggers verbose-detail preview in stderr

- **Given:** `CLR_VERBOSITY=5`; no `--verbosity` on CLI
- **When:** `clr task`
- **Then:** stderr contains assembled command preview (contains `--effort`)
- **Note:** `shows_verbose_detail()` returns true for level ≥ 4; default level 3 does not
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E13: CLR_TRACE prints command to stderr before execution

- **Given:** `CLR_TRACE=1`
- **When:** `clr task`
- **Then:** stderr contains assembled command preview (contains `--effort`)
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E14: CLR_NO_ULTRATHINK suppresses ultrathink suffix

- **Given:** `CLR_NO_ULTRATHINK=1`
- **When:** `clr --dry-run task`
- **Then:** stdout does NOT contain `ultrathink`
- **Exit:** 0
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E15: CLR_SYSTEM_PROMPT sets system prompt

- **Given:** `CLR_SYSTEM_PROMPT=Be concise.`
- **When:** `clr --dry-run task`
- **Then:** stdout contains `--system-prompt`
- **Exit:** 0
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E16: CLR_APPEND_SYSTEM_PROMPT appends to system prompt

- **Given:** `CLR_APPEND_SYSTEM_PROMPT=Always JSON.`
- **When:** `clr --dry-run task`
- **Then:** stdout contains `--append-system-prompt`
- **Exit:** 0
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E17: CLR_EFFORT=low sets effort level

- **Given:** `CLR_EFFORT=low`
- **When:** `clr --dry-run task`
- **Then:** stdout contains `low` (default is `max`; env var overrides)
- **Exit:** 0
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E18: CLR_NO_EFFORT_MAX suppresses default --effort injection

- **Given:** `CLR_NO_EFFORT_MAX=1`
- **When:** `clr --dry-run task`
- **Then:** stdout does NOT contain `--effort`
- **Exit:** 0
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E19: CLR_NO_CHROME suppresses default --chrome injection

- **Given:** `CLR_NO_CHROME=1`
- **When:** `clr --dry-run task`
- **Then:** stdout does NOT contain `--chrome`
- **Exit:** 0
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E20: CLR_NO_PERSIST adds --no-session-persistence

- **Given:** `CLR_NO_PERSIST=1`
- **When:** `clr --dry-run task`
- **Then:** stdout contains `--no-session-persistence`
- **Exit:** 0
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E21: CLR_JSON_SCHEMA sets JSON schema

- **Given:** `CLR_JSON_SCHEMA={"type":"string"}`
- **When:** `clr --dry-run task`
- **Then:** stdout contains `--json-schema`
- **Exit:** 0
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E22: CLR_MCP_CONFIG adds single MCP config path

- **Given:** `CLR_MCP_CONFIG=/tmp/mcp.json`
- **When:** `clr --dry-run task`
- **Then:** stdout contains `--mcp-config` and `/tmp/mcp.json`
- **Exit:** 0
- **Note:** env var supplies at most one path; multiple paths require CLI repeats
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E23: CLR_CREDS supplies credentials path for isolated subcommand

- **Given:** `CLR_CREDS=/tmp/e23.creds.json`; no `--creds` on CLI
- **When:** `clr isolated`
- **Then:** stderr does NOT contain `missing required argument: --creds`
- **Note:** error shifts to file-not-found, confirming `creds_path` was populated from env
- **Source:** [env_param.md §2](../../../../docs/cli/env_param.md)

---

### E24: CLR_TIMEOUT sets isolated subprocess timeout

- **Given:** `CLR_CREDS=/tmp/e24.creds.json` + `CLR_TIMEOUT=5`; no `--creds`/`--timeout` on CLI
- **When:** `clr isolated`
- **Then:** stderr does NOT contain `missing required argument: --creds`
- **Note:** combined with CLR_CREDS to pass argument validation; confirms both vars applied
- **Source:** [env_param.md §2](../../../../docs/cli/env_param.md)
