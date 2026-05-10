# Verb: `clean`

- **Kind:** canonical
- **Availability:** universal
- **`--dry-run`:** `cargo clean -p <module>`

### Command

```bash
cargo clean -p <module>
```

Removes compiled artifacts for the specific crate. Scoped with `-p` — does not clean the entire workspace target directory.

### Notes

Scoped cleaning avoids evicting build cache for other workspace members. Useful before a forced rebuild of a single module without discarding compiled dependencies.

`--dry-run` emits the exact command and exits 0 — no artifacts are removed.

### Example

```bash
# claude_storage (binary module)
./verb/clean              # runs: cargo clean -p claude_storage
./verb/clean --dry-run    # prints: cargo clean -p claude_storage

# claude_assets_core (library module)
./verb/clean              # runs: cargo clean -p claude_assets_core
```

Script body:
```bash
#!/usr/bin/env bash
# do clean — remove generated artifacts and caches (cargo ecosystem)
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/.."
if [[ "${1:-}" == "--dry-run" ]]; then echo "cargo clean -p claude_storage"; exit 0; fi
exec cargo clean -p claude_storage
```
