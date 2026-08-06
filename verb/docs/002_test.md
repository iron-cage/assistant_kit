# Verb: `test`

- **Kind:** canonical
- **Availability:** universal
- **`--dry-run`:** `runbox .live` (workspace) / `runbox .live -- ./module/<name>/verb/test.d/l1` (module)

### Command

```bash
runbox .live
```

Delegates to the globally-installed `runbox` engine (ships with `family_dev`; the workspace's owning config is `runbox/runbox.yml`, discovered by walking up from the current directory). The engine bakes the shared image (`claude_storage_core_test`) if needed, mounts the workspace read-only at its real path, and runs the payload inside the container. With no explicit payload the config's `script:` runs — `verb/test.d/l1`, the full workspace suite: nextest (all features, warnings-as-errors) + doc tests + clippy (`-D warnings`), all `--workspace`.

Module `verb/test` passes the module's own layer as payload instead:

```bash
runbox .live -- ./module/<name>/verb/test.d/l1
```

The module l1 `cd`s to the module directory first, so the same trio runs package-scoped by cwd — no `-p` flags.

### Layers

| Layer | Context | Container | `CARGO_NET_OFFLINE` | Default |
|-------|---------|-----------|---------------------|---------|
| engine | host → container via `runbox .live` | yes | yes (config env + l1 export) | yes — no `VERB_LAYER` set |
| `l0` | host-native | no | — | disabled — hard-error stub |
| `l1` | container-internal | n/a | yes | payload executed by `.live` |

### Notes

`verb/test` rejects any `VERB_LAYER` set on the host side — container execution is the only path (see `module/claude_profile/docs/invariant/009_container_only_test_execution.md`). The authorized host escape hatch is `VERB_LAYER=l0 cargo nextest run` (bypasses `verb/test` entirely; honored by the nextest setup script).

`verb/test.d/l1` is the container-internal implementation: exports `RUNBOX_CONTAINER=1`, `NO_COLOR=1`, `CARGO_NET_OFFLINE=true`, `RUSTFLAGS="-D warnings"`, then runs nextest + doc tests + clippy. The engine supplies `CARGO_TARGET_DIR` (the `claude_storage_core_targets` working volume), so compilation artifacts land outside the read-only workspace mount.

`verb/test.d/l0` is a disabled hard-error stub: prints an error and exits 1 — no host-native test execution path exists.

`--dry-run` prints the delegated command and exits 0 — no tests run.

### Example

```bash
# Workspace suite:
./verb/test               # runs: runbox .live  →  container  →  verb/test.d/l1
./verb/test --dry-run     # prints: runbox .live

# Module suite:
cd module/claude_profile && ./verb/test
# runs: runbox .live -- ./module/claude_profile/verb/test.d/l1

# Container-internal layer — executed by the engine, never by hand on the host:
./verb/test.d/l1
```

The engine discovers `runbox/runbox.yml` from any directory inside the workspace, so both forms work from anywhere in the tree. One shared image serves every invocation; `runbox .build` refreshes it explicitly, and `.live` rebakes automatically when build inputs drift.
