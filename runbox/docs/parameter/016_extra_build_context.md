# Parameter: `extra_build_context`

- **Status:** ⬜ Not Configured — default: `""` (disabled — no extra context passed); not currently in use by any workspace
- **Current State:** Not set. (Historical: previously injected the co-developed `cli_fmt` companion crate before it was migrated to a crates.io registry dependency; the Docker build-context plumbing that supported the path dependency was removed once the migration made it dead code.)
- **Where It Flows:** `runbox.yml extra_build_context:` → `EXTRA_BUILD_CONTEXT` in `runbox-run` → parsed as `name=relpath`, `relpath` resolved to absolute path relative to `CONFIG_DIR` → `--build-context name=abspath` appended to `$CONTAINER_CMD build`

### Notes

Format is `name=relpath` — a single `name=value` pair. `name` is the context identifier used in `FROM name AS ...` lines inside the Dockerfile. `relpath` is a path to the directory relative to the directory containing `runbox.yml` (`CONFIG_DIR`), not relative to `WORKSPACE_ROOT`.

When absent (empty), no `--build-context` argument is passed and the build behaves as a normal single-context build using `WORKSPACE_ROOT` as the sole context.

Use this when a Dockerfile stage needs to copy files from a directory that lives outside `WORKSPACE_ROOT`. Without a named extra context, Docker/Podman cannot reach paths outside the primary build context.

Only one extra context is supported. If multiple external directories are needed, structure them under a common parent and pass the parent as the context.

**External path dep + named build context pattern:** When a workspace crate depends on a crate via a `path =` dependency that points outside `WORKSPACE_ROOT`, Docker cannot reach that path via the primary build context. Use `extra_build_context` to inject the external directory under a name matching the `COPY --from=` reference in the Dockerfile. The Dockerfile then copies the source to the path Cargo resolves to inside the container. (Historical example: this project used to do exactly this for the `cli_fmt` crate — `COPY --from=cli_fmt . /wtools/dev/module/core/cli_fmt/` with `path = "../../wtools/dev/module/core/cli_fmt"` — until `cli_fmt` was migrated to a crates.io registry dependency and the plumbing was removed as dead code. The generic `shared`/`helper` example below demonstrates the pattern for a hypothetical live case.)

### Example

Suppose a Dockerfile references a shared credentials helper that lives two levels above the module:

```dockerfile
# In runbox/runbox.dockerfile — references a named build context "shared"
FROM shared AS helper
COPY --from=helper /creds_tool /usr/local/bin/creds_tool
```

```yaml
# runbox.yml
extra_build_context: shared=../../shared
```

`runbox-run` resolves `../../shared` relative to the directory containing `runbox.yml`, then passes `--build-context shared=/absolute/path/to/shared` to the container build. The Dockerfile's `FROM shared` resolves to that directory.

Removing the key or setting it to `""` disables the extra context — the build falls back to a single-context invocation.
