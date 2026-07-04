# User Story: MCP Config Injection

- **Source:** [docs/cli/user_story/019_mcp_config_injection.md](../../../../docs/cli/user_story/019_mcp_config_injection.md)
- **Primary flags:** `--mcp-config`
- **Command:** `run`

## Test Case Index

| ID | Category | Summary |
|----|----------|---------|
| US-1 | Happy path | `--mcp-config` path appears in assembled command |
| US-2 | Multi-config | Multiple `--mcp-config` flags forwarded individually |
| US-3 | Env var | `CLR_MCP_CONFIG` sets config when flag absent |
| US-4 | Boundary | No `--mcp-config` by default |

---

### US-1: mcp-config appears in assembled command

- **Given:** No prior configuration
- **When:** `clr --mcp-config /tmp/mcp.json --dry-run "Fix bug"`
- **Then:** Assembled command contains `--mcp-config /tmp/mcp.json`
- **Exit:** 0

### US-2: multiple configs forwarded individually

- **Given:** Two MCP config file paths provided
- **When:** `clr --mcp-config /tmp/us19a.json --mcp-config /tmp/us19b.json --dry-run "Fix bug"`
- **Then:** Assembled command contains both `--mcp-config /tmp/us19a.json` and `--mcp-config /tmp/us19b.json` as separate flags
- **Exit:** 0

### US-3: CLR_MCP_CONFIG env var sets single config

- **Given:** `CLR_MCP_CONFIG=/tmp/us19env.json` set; no `--mcp-config` CLI flag
- **When:** `CLR_MCP_CONFIG=/tmp/us19env.json clr --dry-run "Fix bug"`
- **Then:** Assembled command contains `--mcp-config /tmp/us19env.json`
- **Exit:** 0

### US-4: no --mcp-config by default

- **Given:** No `--mcp-config` flag and no `CLR_MCP_CONFIG` env var
- **When:** `clr --dry-run "Fix bug"`
- **Then:** Assembled command does NOT contain `--mcp-config`
- **Exit:** 0
