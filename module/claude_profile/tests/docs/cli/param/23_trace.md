# Parameter :: `trace::`

Edge case tests for the `trace::` parameter. Tests validate boolean enforcement, default-off behavior, stderr output routing (stdout unchanged), and the `[trace]` line format. Used by `.usage` to expose internal fetch and refresh mechanics for diagnostics.

**Source:** [params.md#parameter--23-trace](../../../../docs/cli/params.md#parameter--23-trace)

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | `trace::1` accepted — `[trace]` lines appear on stderr | Trace Enabled |
| EC-2 | `trace::0` accepted — no trace output on stderr | Default Off |
| EC-3 | `trace::2` rejected (out of range) | Boundary Values |
| EC-4 | `trace::yes` rejected (type validation) | Type Validation |
| EC-5 | Default value is `0` (trace disabled) | Default |
| EC-6 | `trace::1` — trace goes to stderr; stdout output unchanged | Output Routing |

## Test Coverage Summary

- Trace Enabled: 1 test (EC-1)
- Default Off: 1 test (EC-2)
- Boundary Values: 1 test (EC-3)
- Type Validation: 1 test (EC-4)
- Default: 1 test (EC-5)
- Output Routing: 1 test (EC-6)

**Total:** 6 edge cases

**Behavioral Divergence Pair:** EC-1 (trace enabled — diagnostics on stderr) ↔ EC-5 (absent by default — no diagnostic output)

## Test Cases
---

### EC-1: `trace::1` — `[trace]` lines appear on stderr

- **Given:** `.usage` environment with valid credentials and at least one saved account.
- **When:** `clp .usage trace::1`
- **Then:** stderr contains at least one line beginning with `[trace]`; exit 0.
- **Exit:** 0
- **Source fn:** `it34_trace_param_writes_to_stderr`
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/params.md#parameter--23-trace)
---

### EC-2: `trace::0` — explicit disable; no trace output

- **Given:** `.usage` environment with valid credentials.
- **When:** `clp .usage trace::0`
- **Then:** stderr contains no `[trace]` lines; behavior identical to default; exit 0.
- **Exit:** 0
- **Source fn:** `it49_trace_0_no_trace_on_stderr`
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/params.md#parameter--23-trace)
---

### EC-3: `trace::2` rejected

- **Given:** clean environment with valid credentials.
- **When:** `clp .usage trace::2`
- **Then:** Exit 1 with error referencing `trace::`; must be 0 or 1.
- **Exit:** 1
- **Source fn:** `it50_trace_2_rejected`
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/params.md#parameter--23-trace)
---

### EC-4: `trace::yes` rejected

- **Given:** clean environment with valid credentials.
- **When:** `clp .usage trace::yes`
- **Then:** Exit 1 with type validation error referencing `trace::`.
- **Exit:** 1
- **Source fn:** `it51_trace_yes_rejected`
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/params.md#parameter--23-trace)
---

### EC-5: Default value is `0` (trace disabled)

- **Given:** `.usage` environment with valid credentials.
- **When:** `clp .usage` (no `trace::` param)
- **Then:** stderr contains no `[trace]` lines; behavior identical to `trace::0`; exit 0.
- **Exit:** 0
- **Source fn:** `it52_trace_default_off`
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/params.md#parameter--23-trace)
---

### EC-6: `trace::1` — trace output on stderr does not appear on stdout

- **Given:** `.usage` environment with valid credentials and at least one saved account.
- **When:** `clp .usage trace::1`
- **Then:** stdout contains the normal quota table output only (no `[trace]` lines); stderr contains `[trace]` lines; the two streams are independent; exit 0.
- **Exit:** 0
- **Source fn:** `it34_trace_param_writes_to_stderr`
- **Source:** [params.md#parameter--23-trace](../../../../docs/cli/params.md#parameter--23-trace)
