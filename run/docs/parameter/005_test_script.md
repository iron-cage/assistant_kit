# Parameter: `test_script`

- **Status:** ✅ Configured — present in `runbox.yml`
- **Current State:** `verb/test` for standalone projects; module-relative path (e.g., `module/claude_profile/verb/test`) for workspace modules
- **Where It Flows:** `docker run /workspace/$TEST_SCRIPT` — executed after plugin mounts are wired

### Notes

Full-test entrypoint. May invoke bin plugins (e.g., `w3`) and assumes plugin mounts are present. Used by `runbox-run`'s `.test` command path. `_ensure_image()` probes for `test_script`, `lint_script`, and `run_script` in a single container run before executing any command; any missing file triggers an automatic rebuild rather than emitting a cryptic OCI "not found" error.

Use `$SCRIPT_DIR`-relative paths in the layer script body — inside the container `SCRIPT_DIR` resolves to `$WORKSPACE_DIR/verb/test.d`, so `$SCRIPT_DIR/../..` is `$WORKSPACE_DIR`. The dispatcher (`verb/test`) itself does not use SCRIPT_DIR for execution paths.

Module-level runboxes point at `verb/test` (the canonical `do`-protocol test verb) rather than a bespoke `run/test` script. This makes `verb/test` the single source of truth for what "run tests" means for a module.

### Multi-Layer Verbs

`test_script` may point to a verb dispatcher — a plain executable file that reads `VERB_LAYER` and self-dispatches to `test.d/l0` (host-native, default when no `VERB_LAYER`) or `test.d/l1` (container invocation, `VERB_LAYER=l1` set by `runbox-run`). `verb/` has no knowledge of `run/` — the dispatcher never calls `run/runbox`. `verb/test` is always a file; no directory detection is needed in `runbox-run`.

`runbox-run` passes `-e VERB_LAYER=l1` to the container run. The dispatcher inside `verb/test` routes to `test.d/l1` for direct execution.

See `onboarding.md § Multi-Layer Verbs` for the complete protocol, layer naming, dispatch rules, and `VERB_LAYER` convention.

### Example

Standalone project `runbox.yml`:
```yaml
test_script: verb/test
```
Module `runbox.yml` (`claude_profile`):
```yaml
test_script: module/claude_profile/verb/test
```
`cmd_test()` builds the full docker invocation:
```bash
docker run --rm \
  --user $(id -u):$(id -g) \
  -v claude_profile_test_plugin_targets:/tmp/will_test_targets \
  -v /usr/local/bin/w3:/usr/local/bin/w3:ro \
  -v /home/user/.claude:/workspace/.claude:rw \
  claude_profile_test \
  /workspace/module/claude_profile/verb/test
```
The script runs inside the container after all plugin mounts are wired, so it can invoke `w3` and read credentials from `/workspace/.claude`. This is the full-test path; `cmd_test_offline()` skips the script entirely and runs the baked CMD directly.
