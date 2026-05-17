# Parameter: `extra_build_context`

- **Status:** ✅ Configured — via `runbox.yml`; default: `""` (disabled — no extra context passed)
- **Current State:** `wtools_cli_fmt=../../../wtools/dev/module/core/cli_fmt` — injects the `cli_fmt` crate from the wtools workspace; needed because `cli_fmt` is a path dependency (`path = "../wtools/dev/module/core/cli_fmt"` in `Cargo.toml`) that lives outside `WORKSPACE_ROOT`, so Docker cannot reach it via the primary build context; the Dockerfile's `COPY --from=wtools_cli_fmt . /wtools/dev/module/core/cli_fmt/` populates the path Cargo resolves to from `/workspace/`
- **Where It Flows:** `runbox.yml extra_build_context:` → `EXTRA_BUILD_CONTEXT` in `runbox-run` → parsed as `name=relpath`, `relpath` resolved to absolute path relative to `CONFIG_DIR` → `--build-context name=abspath` appended to `$CONTAINER_CMD build`

### Notes

Format is `name=relpath` — a single `name=value` pair. `name` is the context identifier used in `FROM name AS ...` lines inside the Dockerfile. `relpath` is a path to the directory relative to the directory containing `runbox.yml` (`CONFIG_DIR`), not relative to `WORKSPACE_ROOT`.

When absent (empty), no `--build-context` argument is passed and the build behaves as a normal single-context build using `WORKSPACE_ROOT` as the sole context.

Use this when a Dockerfile stage needs to copy files from a directory that lives outside `WORKSPACE_ROOT`. Without a named extra context, Docker/Podman cannot reach paths outside the primary build context.

Only one extra context is supported. If multiple external directories are needed, structure them under a common parent and pass the parent as the context.

**External path dep + named build context pattern:** When a workspace crate depends on a crate via a `path =` dependency that points outside `WORKSPACE_ROOT` (e.g., `path = "../wtools/dev/module/core/cli_fmt"`), Docker cannot reach that path via the primary build context. Use `extra_build_context` to inject the external directory under a name matching the `COPY --from=` reference in the Dockerfile. The Dockerfile then copies the source to the path Cargo resolves to inside the container (e.g., `COPY --from=wtools_cli_fmt . /wtools/dev/module/core/cli_fmt/` when WORKDIR is `/workspace/`).

### Example

Suppose a Dockerfile references a shared credentials helper that lives two levels above the module:

```dockerfile
# In run/runbox.dockerfile — references a named build context "shared"
FROM shared AS helper
COPY --from=helper /creds_tool /usr/local/bin/creds_tool
```

```yaml
# runbox.yml
extra_build_context: shared=../../shared
```

`runbox-run` resolves `../../shared` relative to the directory containing `runbox.yml`, then passes `--build-context shared=/absolute/path/to/shared` to the container build. The Dockerfile's `FROM shared` resolves to that directory.

Removing the key or setting it to `""` disables the extra context — the build falls back to a single-context invocation.
