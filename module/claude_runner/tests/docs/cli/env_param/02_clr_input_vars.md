# Env Param :: CLR_* Input Variables

Edge cases for the 37 `CLR_*` input environment variable fallbacks (34 for `run` + 3 for `isolated`/`refresh`).
Source: [`env_param.md`](../../../../docs/cli/env_param.md)
Implementation: `apply_env_vars()` in `src/cli/parse.rs`; `apply_isolated_env_vars()` and `apply_refresh_env_vars()` in `src/cli/cred_parse.rs`
Test files: `tests/env_var_test.rs` (E01–E17), `tests/env_var_ext_test.rs` (E18–E37)

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
| E23 | `CLR_CREDS` supplies isolated creds path | `CLR_CREDS` | file-not-found error (creds path populated from env); no HOME-resolution error |
| E24 | `CLR_TIMEOUT` sets isolated timeout | `CLR_TIMEOUT` | argument parsing succeeds with `CLR_CREDS+CLR_TIMEOUT` |
| E25 | `CLR_FILE` supplies file path | `CLR_FILE` | describe output includes the file path (same as `--file`) |
| E26 | `CLR_STRIP_FENCES=1` strips fences | `CLR_STRIP_FENCES` | captured stdout has fences removed (same as `--strip-fences`) |
| E27 | `CLAUDECODE=1 CLR_KEEP_CLAUDECODE=1` preserves env var | `CLR_KEEP_CLAUDECODE` | subprocess env contains `CLAUDECODE` (same as `--keep-claudecode`) |
| E28 | `CLR_TRACE` enables trace for `isolated`/`refresh` | `CLR_TRACE` | trace output appears in stderr for credential ops (cross-command) |
| E29 | `CLR_SUBDIR=NAME` appends subdirectory to base dir | `CLR_SUBDIR` | dry-run output contains effective dir ending in `/-NAME` |
| E30 | `CLR_MAX_SESSIONS=N` sets session limit; invalid value silently ignored | `CLR_MAX_SESSIONS` | gate uses N as limit; invalid value → default 20 used; CLI wins |
| E31 | `CLR_OUTPUT_FILE=<path>` sets output file path | `CLR_OUTPUT_FILE` | dry-run exits 0; CLI flag wins over env var |
| E32 | `CLR_EXPECT=val1\|val2` sets expect pattern | `CLR_EXPECT` | dry-run exits 0; CLI flag wins; same `|`-separated syntax |
| E33 | `CLR_EXPECT_STRATEGY=<strategy>` sets mismatch handler | `CLR_EXPECT_STRATEGY` | dry-run exits 0; CLI flag wins; invalid value rejected |
| E34 | `CLR_EXPECT_RETRIES=N` sets retry cap | `CLR_EXPECT_RETRIES` | dry-run exits 0; CLI flag wins; out-of-range rejected |
| E35 | `CLR_RETRY_ON_RATE_LIMIT=N` sets retry count | `CLR_RETRY_ON_RATE_LIMIT` | dry-run exits 0; CLI flag wins; invalid value silently ignored |
| E36 | `CLR_RETRY_DELAY=N` sets retry delay (secs) | `CLR_RETRY_DELAY` | dry-run exits 0; CLI flag wins; invalid value silently ignored |
| E37 | `CLR_TIMEOUT=N` sets run/ask subprocess timeout | `CLR_TIMEOUT` | dry-run exits 0; CLI flag wins; `0` = unlimited; invalid silently ignored |

## Test Coverage Summary

- Bool vars (truthy only): E02, E04, E05, E06, E07, E11, E13, E14, E18, E19, E28 (11 tests)
- String vars: E01, E03, E08, E10, E15, E16, E21, E22, E23, E29, E31, E32, E33 (13 tests)
- Parsed vars (with silent-ignore): E09, E12, E17, E30, E35, E36, E37 (7 tests)
- Parsed vars (with rejection): E34 (1 test)
- Negation suppression (suppress default injection): E05, E06, E07, E18, E19 (5 tests)
- CLI-wins verification: E01, E03, E29, E30, E31, E32, E33, E34, E35, E36, E37 (11 tests)
- Isolated subcommand: E23, E24 (2 tests)
- Credential ops (cross-command): E28 (1 test)

**Total:** 37 edge cases (E01–E37)

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
- **Then:** stderr contains a file-not-found error referencing the CLR_CREDS path (not a HOME-resolution error); confirms `creds_path` was populated from tier-2 env var
- **Source:** [env_param.md §2](../../../../docs/cli/env_param.md)

---

### E24: CLR_TIMEOUT sets isolated subprocess timeout

- **Given:** `CLR_CREDS=/tmp/e24.creds.json` + `CLR_TIMEOUT=5`; no `--creds`/`--timeout` on CLI
- **When:** `clr isolated`
- **Then:** stderr contains a file-not-found error (creds path populated from CLR_CREDS); no HOME-resolution error; confirms both tier-2 vars applied
- **Source:** [env_param.md §2](../../../../docs/cli/env_param.md)

---

### E25: CLR_FILE supplies file path

- **Given:** `CLR_FILE=/tmp/x.txt`; no `--file` on CLI; `/tmp/x.txt` exists
- **When:** `clr --dry-run task`
- **Then:** describe output includes the file path (same as `clr --dry-run --file /tmp/x.txt task`)
- **Exit:** 0
- **CLI-wins:** `clr --dry-run --file /tmp/other.txt task` with `CLR_FILE=/tmp/x.txt` → `/tmp/other.txt` used, NOT `/tmp/x.txt`
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E26: CLR_STRIP_FENCES=1 strips fences from captured stdout

- **Given:** `CLR_STRIP_FENCES=1`; no `--strip-fences` on CLI; fake claude emits fenced output
- **When:** `clr -p task`
- **Then:** captured stdout has fence lines removed (same as `clr -p --strip-fences task`)
- **Exit:** 0
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E27: CLR_KEEP_CLAUDECODE=1 preserves CLAUDECODE in subprocess env

- **Given:** parent env has `CLAUDECODE=1`; `CLR_KEEP_CLAUDECODE=1`; no `--keep-claudecode` on CLI
- **When:** `clr task` (via fake claude that prints its env)
- **Then:** subprocess env contains `CLAUDECODE` (same as `clr --keep-claudecode task`)
- **Exit:** 0
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E28: CLR_TRACE enables trace for isolated/refresh subcommands

- **Given:** `CLR_TRACE=1`; `CLR_CREDS=/tmp/e28.creds.json`; no `--trace` on CLI
- **When:** `clr isolated` (dry-run or parse-only path)
- **Then:** stderr contains creds path and temp HOME details (cross-command trace output)
- **Note:** `CLR_TRACE` is shared with `run` (E13); this case validates `apply_isolated_env_vars()` and `apply_refresh_env_vars()` apply it independently
- **Source:** [env_param.md §2](../../../../docs/cli/env_param.md)

---

### E29: CLR_SUBDIR=NAME appends named subdirectory

- **Given:** `CLR_SUBDIR=feature`; no `--subdir` on CLI
- **When:** `clr --dry-run task`
- **Then:** dry-run output contains the effective dir ending in `/-feature`
- **Exit:** 0
- **CLI-wins:** `clr --dry-run --subdir cliname task` with `CLR_SUBDIR=envname` → effective dir ends in `/-cliname`, NOT `/-envname`
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E30: CLR_MAX_SESSIONS=N sets session limit; invalid value silently ignored

- **Given:** `CLR_MAX_SESSIONS=3`; no `--max-sessions` on CLI; `--dry-run` set
- **When:** `CLR_MAX_SESSIONS=3 clr --dry-run task`
- **Then:** exit 0; env var applied (gate uses 3 as limit in a live run); dry-run skips gate and produces output immediately
- **Exit:** 0
- **Invalid-ignored:** `CLR_MAX_SESSIONS=notanumber` → parse failure silently ignored; default 20 used; `--dry-run` exits 0 normally
- **CLI-wins:** `clr --max-sessions 5 --dry-run task` with `CLR_MAX_SESSIONS=2` → CLI value 5 used; env var 2 ignored
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E31: CLR_OUTPUT_FILE sets output file path

- **Given:** `CLR_OUTPUT_FILE=/tmp/e31_out.txt`; no `--output-file` on CLI; `--dry-run` set
- **When:** `CLR_OUTPUT_FILE=/tmp/e31_out.txt clr --dry-run task`
- **Then:** exit 0; env var applied (output would tee to `/tmp/e31_out.txt` in a live run); dry-run exits 0 normally without creating the file
- **Exit:** 0
- **CLI-wins:** `clr --output-file /tmp/cli.txt --dry-run task` with `CLR_OUTPUT_FILE=/tmp/env.txt` → CLI value `/tmp/cli.txt` used; env var `/tmp/env.txt` ignored
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E32: CLR_EXPECT sets expect pattern

- **Given:** `CLR_EXPECT=yes|no`; no `--expect` on CLI; `--dry-run` set
- **When:** `CLR_EXPECT=yes|no clr --dry-run task`
- **Then:** exit 0; env var applied (validation would check stdout against `yes|no` in a live run); dry-run exits 0 normally
- **Exit:** 0
- **CLI-wins:** `clr --expect "ok|fail" --dry-run task` with `CLR_EXPECT=yes|no` → CLI value `ok|fail` used; env var ignored
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E33: CLR_EXPECT_STRATEGY sets mismatch handler

- **Given:** `CLR_EXPECT_STRATEGY=retry`; no `--expect-strategy` on CLI; `--dry-run` set
- **When:** `CLR_EXPECT_STRATEGY=retry clr --dry-run task`
- **Then:** exit 0; env var applied; dry-run exits 0 normally
- **Exit:** 0
- **CLI-wins:** `clr --expect-strategy fail --dry-run task` with `CLR_EXPECT_STRATEGY=retry` → CLI value `fail` used; env var `retry` ignored
- **Invalid:** `CLR_EXPECT_STRATEGY=bogus clr --dry-run task` → parse failure; exit 1 with error message about invalid strategy value
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E34: CLR_EXPECT_RETRIES sets retry cap

- **Given:** `CLR_EXPECT_RETRIES=3`; no `--expect-retries` on CLI; `--dry-run` set
- **When:** `CLR_EXPECT_RETRIES=3 clr --dry-run task`
- **Then:** exit 0; env var applied (retry cap would be 3 in a live run with `--expect-strategy retry`); dry-run exits 0 normally
- **Exit:** 0
- **CLI-wins:** `clr --expect-retries 5 --dry-run task` with `CLR_EXPECT_RETRIES=3` → CLI value 5 used; env var 3 ignored
- **Out-of-range:** `CLR_EXPECT_RETRIES=256 clr --dry-run task` → exit 1; stderr contains error about value exceeding u8 range (max 255)
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E35: CLR_RETRY_ON_RATE_LIMIT sets rate-limit retry count

- **Given:** `CLR_RETRY_ON_RATE_LIMIT=3`; no `--retry-on-rate-limit` on CLI; `--dry-run` set
- **When:** `CLR_RETRY_ON_RATE_LIMIT=3 clr --dry-run task`
- **Then:** exit 0; env var applied (retry count would be 3 on a rate-limit hit in a live run); dry-run exits 0 normally
- **Exit:** 0
- **CLI-wins:** `clr --retry-on-rate-limit 0 --dry-run task` with `CLR_RETRY_ON_RATE_LIMIT=3` → CLI value 0 used; env var 3 ignored
- **Invalid-ignored:** `CLR_RETRY_ON_RATE_LIMIT=notanumber` → parse failure silently ignored; default 1 used; dry-run exits 0 normally
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E36: CLR_RETRY_DELAY sets retry delay in seconds

- **Given:** `CLR_RETRY_DELAY=60`; no `--retry-delay` on CLI; `--dry-run` set
- **When:** `CLR_RETRY_DELAY=60 clr --dry-run task`
- **Then:** exit 0; env var applied (delay between rate-limit retries would be 60s in a live run); dry-run exits 0 normally
- **Exit:** 0
- **CLI-wins:** `clr --retry-delay 5 --dry-run task` with `CLR_RETRY_DELAY=60` → CLI value 5 used; env var 60 ignored
- **Invalid-ignored:** `CLR_RETRY_DELAY=notanumber` → parse failure silently ignored; default 30 used; dry-run exits 0 normally
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)

---

### E37: CLR_TIMEOUT sets run/ask subprocess timeout

- **Given:** `CLR_TIMEOUT=30`; no `--timeout` on CLI; `--dry-run` set
- **When:** `CLR_TIMEOUT=30 clr --dry-run task`
- **Then:** exit 0; env var applied (subprocess watchdog would fire after 30s in a live run); dry-run exits 0 normally
- **Exit:** 0
- **CLI-wins:** `clr --timeout 60 --dry-run task` with `CLR_TIMEOUT=30` → CLI value 60 used; env var 30 ignored
- **Zero:** `CLR_TIMEOUT=0` → unlimited (no watchdog); dry-run exits 0 normally
- **Invalid-ignored:** `CLR_TIMEOUT=notanumber` → parse failure silently ignored; default 0 (unlimited) used; dry-run exits 0 normally
- **Cross-command:** `CLR_TIMEOUT` also applies to `isolated`/`refresh` (same semantics: 0 = unlimited); tested separately in E24
- **Source:** [env_param.md §1](../../../../docs/cli/env_param.md)
