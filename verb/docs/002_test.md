# Verb: `test`

- **Kind:** canonical
- **Availability:** universal
- **`--dry-run`:** `runbox .live` (workspace) / `runbox .live -- ./module/<name>/verb/test.d/l1` (module)

### Command

```bash
runbox .live
```

Delegates to the globally-installed `runbox` engine (ships with `family_dev`; the workspace's owning config is `runbox/runbox.yml`, discovered by walking up from the current directory). The engine bakes the shared image (`assistant_kit_claude_profile_test`) if needed, mounts the workspace read-only at its real path, and runs the payload inside the container. With no explicit payload the config's `script:` runs — `verb/test.d/l1`, which execs `will .test level::3 show_build::1`: nextest (all features) + doc tests + clippy, all warnings-as-errors, workspace-scoped by cwd (`show_build::1` pins full per-job streaming — will's current default is quiet-on-success, which prints nothing on a clean pass). `will` is a binary plugin mounted at `/usr/local/bin/will` — the engine unwraps the host's knob-aware `bin/will` wrapper to the fresh managed ELF, and will prints its own summary report at the end of the gate.

Module `verb/test` passes the module's own layer as payload instead:

```bash
runbox .live -- ./module/<name>/verb/test.d/l1
```

The module l1 `cd`s to the module directory first, so the same `will .test level::3` gate runs package-scoped by cwd — no `-p` flags.

### Layers

| Layer | Context | Container | `CARGO_NET_OFFLINE` | Default |
|-------|---------|-----------|---------------------|---------|
| engine | host → container via `runbox .live` | yes | yes (config env + l1 export) | yes — no `VERB_LAYER` set |
| `l0` | host-native | no | — | disabled — hard-error stub |
| `l1` | container-internal | n/a | yes | payload executed by `.live` |

### Notes

`verb/test` rejects any `VERB_LAYER` set on the host side — container execution is the only path (see `module/claude_profile/docs/invariant/009_container_only_test_execution.md`). The authorized host escape hatch is `VERB_LAYER=l0 cargo nextest run` (bypasses `verb/test` entirely; honored by the nextest setup script).

`verb/test.d/l1` is the container-internal implementation: exports `RUNBOX_CONTAINER=1`, `NO_COLOR=1`, `CARGO_NET_OFFLINE=true`, and `W3_TEST_DELEGATE=0` (will's verb-first delegation kill switch — delegated jobs would re-invoke module `verb/test` wrappers, which need the host-side `runbox`; in-container the direct pipeline is the correct semantic), then execs `will .test level::3` (nextest + doc tests + clippy — will owns the `RUSTFLAGS` policy, so the layer no longer exports it). The engine supplies `CARGO_TARGET_DIR` (the `claude_profile_targets` working volume), so compilation artifacts land outside the read-only workspace mount.

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
