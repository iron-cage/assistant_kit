# Verb: `build`

- **Kind:** canonical
- **Availability:** universal
- **`--dry-run`:** `cargo build -p <module>`

### Command

```bash
cargo build -p <module>
```

Scoped to the target module with `-p`. Binary modules produce their binary artifact; library modules compile the lib crate.

### Notes

`cd "$SCRIPT_DIR/.."` positions the shell at the module root before invoking cargo, ensuring the correct workspace context. The `-p` scope prevents cargo from building unrelated workspace members.

`--dry-run` emits the exact command and exits 0 — no compilation occurs. Used by tooling (CI, runbox, `verbs`) to discover what `build` would do without side effects.

### Example

```bash
# claude_profile (binary module)
./verb/build              # runs: cargo build -p claude_profile
./verb/build --dry-run    # prints: cargo build -p claude_profile

# claude_storage_core (library module)
./verb/build              # runs: cargo build -p claude_storage_core
```

Script body:
```bash
#!/usr/bin/env bash
# do build — compile project artifacts (cargo ecosystem)
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/.."
if [[ "${1:-}" == "--dry-run" ]]; then echo "cargo build -p claude_profile"; exit 0; fi
exec cargo build -p claude_profile
```
