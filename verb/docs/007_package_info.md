# Verb: `package_info`

- **Kind:** meta
- **Availability:** universal
- **`--dry-run`:** not supported (output is the result)

### Command

```bash
python3 - <<'PYEOF'
import json, re, os
# reads Cargo.toml; falls back to workspace root for inherited fields
print("\n" + json.dumps({...}, indent=2) + "\n")
PYEOF
```

Deterministic flat JSON — output is identical on any machine for the same package version.

### Output Fields

| Field | Source | Deterministic? |
|-------|--------|---------------|
| `name` | `Cargo.toml` `name` | yes |
| `version` | `Cargo.toml` `version`; inherits from `[workspace.package]` if `.workspace = true` | yes |
| `edition` | `Cargo.toml` `edition`; inherits from `[workspace.package]` if `.workspace = true` | yes |
| `language` | static | yes |
| `package_manager` | static | yes |
| `signal` | static | yes |
| `confidence` | static | yes |

### Notes

Protocol discovery verb — reports package identity and ecosystem for tooling integration. Output is machine-readable JSON.

All fields are **package-level**: the output is fully deterministic and identical across machines. Runtime environment (OS, architecture, toolchain versions) is intentionally excluded — those are environment properties, not package properties.

`version` and `edition` resolve workspace inheritance: if the module's `Cargo.toml` uses `version.workspace = true`, the script reads the value from `[workspace.package]` in the workspace root `Cargo.toml`.

The script body is **identical across all workspace modules** — module identity comes from reading `Cargo.toml` at runtime, not from hardcoded values.

No `--dry-run` flag: the output IS the result — no side effect to preview.

### Example

```bash
./verb/package_info
```

Output:

```json

{
  "name": "claude_profile",
  "version": "1.1.0",
  "edition": "2021",
  "language": "rust",
  "package_manager": "cargo",
  "signal": "Cargo.toml",
  "confidence": "inferred"
}

```

### Script Body

```bash
#!/usr/bin/env bash
# do package_info — report package metadata as JSON (deterministic, package-level only)
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/.."
python3 - <<'PYEOF'
import json, re, os

with open("Cargo.toml") as f:
  toml = f.read()

ws_toml = ""
ws_path = os.path.join("..", "..", "Cargo.toml")
if os.path.exists(ws_path):
  with open(ws_path) as f:
    ws_toml = f.read()

def direct(key, src):
  m = re.search(rf'^{key}\s*=\s*"([^"]+)"', src, re.MULTILINE)
  return m.group(1) if m else None

def resolve(key):
  v = direct(key, toml)
  if v:
    return v
  if re.search(rf'^{key}\.workspace\s*=\s*true', toml, re.MULTILINE) and ws_toml:
    section = re.search(r'\[workspace\.package\](.*?)(?=\n\[|\Z)', ws_toml, re.DOTALL)
    if section:
      v = direct(key, section.group(1))
      if v:
        return v
  return "unknown"

print("\n" + json.dumps({
  "name":            direct("name", toml) or "unknown",
  "version":         resolve("version"),
  "edition":         resolve("edition"),
  "language":        "rust",
  "package_manager": "cargo",
  "signal":          "Cargo.toml",
  "confidence":      "inferred",
}, indent=2) + "\n")
PYEOF
```
