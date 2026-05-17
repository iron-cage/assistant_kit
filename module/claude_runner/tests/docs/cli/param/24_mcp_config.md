# Parameter :: `--mcp-config`

Edge case coverage for the `--mcp-config` parameter. See [24_mcp_config.md](../../../../docs/cli/param/24_mcp_config.md) for specification.

## Test Case Index

| ID | Test Name | Category |
|----|-----------|----------|
| EC-1 | Single `--mcp-config` path → forwarded to assembled command | Behavioral Divergence |
| EC-2 | Default (no `--mcp-config`) → no `--mcp-config` in assembled command | Behavioral Divergence |
| EC-3 | Multiple `--mcp-config` flags → all forwarded individually | Edge Case |
| EC-4 | `--help` output contains `--mcp-config` | Documentation |
| EC-5 | `--mcp-config` + `--model` → both forwarded, no conflict | Interaction |
| EC-6 | `--mcp-config` without message → accepted; path in assembled command | Edge Case |

## Test Coverage Summary

- Behavioral Divergence: 2 tests (EC-1, EC-2)
- Edge Case: 2 tests
- Interaction: 1 test
- Documentation: 1 test

**Total:** 6 edge cases

---

### EC-1: Single `--mcp-config` forwarded

- **Given:** clean environment; a temp file exists at a known path
- **When:** `clr --dry-run --mcp-config /tmp/mcp.json "task"`
- **Then:** Assembled command contains `--mcp-config /tmp/mcp.json`
- **Exit:** 0
- **Source:** [--mcp-config](../../../../docs/cli/param/24_mcp_config.md)

---

### EC-2: Default → no `--mcp-config`

- **Given:** clean environment
- **When:** `clr --dry-run "task"`
- **Then:** Assembled command does NOT contain `--mcp-config`
- **Exit:** 0
- **Source:** [--mcp-config](../../../../docs/cli/param/24_mcp_config.md)

---

### EC-3: Multiple `--mcp-config` flags forwarded individually

- **Given:** clean environment
- **When:** `clr --dry-run --mcp-config /tmp/s1.json --mcp-config /tmp/s2.json "task"`
- **Then:** Assembled command contains `--mcp-config /tmp/s1.json` and `--mcp-config /tmp/s2.json` as separate occurrences
- **Exit:** 0
- **Source:** [--mcp-config](../../../../docs/cli/param/24_mcp_config.md)

---

### EC-4: `--help` lists `--mcp-config`

- **Given:** clean environment
- **When:** `clr --help`
- **Then:** Stdout contains `--mcp-config`
- **Exit:** 0
- **Source:** [command.md — help](../../../../docs/cli/command.md#command--2-help)

---

### EC-5: `--mcp-config` + `--model` → both forwarded

- **Given:** clean environment
- **When:** `clr --dry-run --mcp-config /tmp/mcp.json --model sonnet "task"`
- **Then:** Assembled command contains both `--mcp-config` and `--model sonnet`
- **Exit:** 0
- **Source:** [--mcp-config](../../../../docs/cli/param/24_mcp_config.md)

---

### EC-6: `--mcp-config` without message

- **Given:** clean environment
- **When:** `clr --dry-run --mcp-config /tmp/mcp.json`
- **Then:** Exit 0; assembled command contains `--mcp-config /tmp/mcp.json`
- **Exit:** 0
- **Source:** [--mcp-config](../../../../docs/cli/param/24_mcp_config.md)
