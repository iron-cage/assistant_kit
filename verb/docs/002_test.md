# Verb: `test`

- **Kind:** canonical
- **Availability:** universal
- **`--dry-run`:** `w3 .test level::3`

### Command

```bash
w3 .test level::3
```

Delegates to `w3`, the workspace-level test runner. Level 3 runs: nextest (all features, warnings-as-errors) + doc tests + clippy (-D warnings). See `CLAUDE.md § Full Verification Commands` for the level breakdown.

### Notes

The `test` verb is **identical across all modules** — the command does not vary by module name because `w3 .test level::3` already scopes itself to the current workspace context. This makes `verb/test` the single source of truth for what "run tests" means for any module.

`verb/test` doubles as the runbox `test_script`: `cmd_test()` in `docker-run` mounts and executes `/workspace/module/<name>/verb/test` inside the container. The `_ensure_image()` probe checks for this file inside the image before running; a stale image (missing the script) triggers an automatic rebuild. See `run/docs/parameter/005_test_script.md`.

`--dry-run` prints `w3 .test level::3` and exits 0 — no tests run.

### Example

```bash
# Any module (command is identical)
./verb/test               # runs: w3 .test level::3
./verb/test --dry-run     # prints: w3 .test level::3
```

Runbox invocation inside Docker:
```bash
docker run --rm \
  -v claude_profile_test_plugin_targets:/tmp/will_test_targets \
  -v /usr/local/bin/w3:/usr/local/bin/w3:ro \
  -v /home/user/.claude:/workspace/.claude:rw \
  claude_profile_test \
  /workspace/module/claude_profile/verb/test
```

Script body:
```bash
#!/usr/bin/env bash
# do test — run the project test suite (cargo ecosystem)
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/.."
if [[ "${1:-}" == "--dry-run" ]]; then echo "w3 .test level::3"; exit 0; fi
exec w3 .test level::3
```
