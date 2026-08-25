# Verb: `lint`

- **Kind:** canonical
- **Availability:** universal
- **`--dry-run`:** `runbox .live -- ./verb/lint.d/l1` (workspace) / `runbox .live -- ./module/<name>/verb/lint.d/l1` (module) — forwarded clippy args are appended to the printed line

### Command

```bash
runbox .live -- ./verb/lint.d/l1 [clippy-arg...]
```

Delegates to the globally-installed `runbox` engine (provisioned separately, expected on `PATH`; the workspace's owning config is `runbox/runbox.yml`, discovered by walking up from the current directory) and runs the payload inside the container, exactly as `verb/test` does. The payload execs `cargo clippy --workspace --all-targets --all-features -- -D warnings` with cwd at the workspace root.

Module `verb/lint` passes the module's own layer as payload, and the payload swaps `--workspace` for `-p <name>`:

```bash
runbox .live -- ./module/<name>/verb/lint.d/l1 [clippy-arg...]
```

**Why the container.** Clippy is a compilation, not a read: run on the host it writes the host Cargo cache and links against host native libraries, which `rulebook.md § Test Execution : Container-Only Testing` forbids — that rule names clippy explicitly, alongside cargo tests, nextest, and crate binaries. `verb/lint` therefore enforces the same host-side guard `verb/test` does: any `VERB_LAYER` set on the host is a hard error, and `lint.d/l0` is a disabled stub that exits 1 rather than offering a host-native path.

**Argument forwarding.** Every argument given to `./verb/lint` is appended verbatim after the payload and lands on `cargo clippy`, ahead of the `-- -D warnings` separator — at module scope `./verb/lint --no-deps` runs `cargo clippy -p <name> --all-targets --all-features --no-deps -- -D warnings`, and at workspace scope the same with `--workspace` in place of `-p <name>`. Unlike `verb/test`, no argument is supplied by the wrapper that a caller might collide with, so there is no override-versus-append distinction here.

### Layers

| Layer | Context | Container | `CARGO_NET_OFFLINE` | Default |
|-------|---------|-----------|---------------------|---------|
| engine | host → container via `runbox .live` | yes | yes (config env + l1 export) | yes — no `VERB_LAYER` set |
| `l0` | host-native | no | — | disabled — hard-error stub |
| `l1` | container-internal | n/a | yes | payload executed by `.live` |

### Notes

`-D warnings` enforces the zero-warning policy — any clippy warning fails the verb. `--all-features` lints feature-gated code paths so feature-specific warnings cannot hide until CI.

`--all-targets` is deliberate: without it clippy skips `tests/`, `benches/`, and `examples/`, so a lint error in an integration test would pass `verb/lint` and then fail the level-3 gate. This is the same flag set will's level-3 clippy phase uses, which is what keeps `./verb/lint` and `./verb/test` from ever disagreeing about what counts as clean.

`lint` is a subset of what `test` runs: the container suite (`runbox .live` → `test.d/l1` → `will .test level::3`) already ends with clippy. `lint` exists as a standalone verb for rapid feedback — same environment, same flags, without the nextest and doc-test phases.

`--dry-run` prints the delegated command and exits 0 — no analysis runs. It is recognized only as the first argument; anything after it is echoed into the printed line.

The linter is ecosystem-specific — ruff for Python, eslint for Node.js, cargo clippy for Rust — and `verb/lint` is `available` for all project types, since linting is universal. Every module here that implements the verb protocol is a cargo crate, so every `lint.d/l1` in this workspace is a clippy invocation. Three `module/` directories — `claude_memory`, `claude_patch`, `claude_patch_core` — are docs-stage scaffolds with no `Cargo.toml` and no `verb/`, so they have no lint entry point to run at all.

### Example

```bash
# Workspace-wide:
./verb/lint             # runs:   runbox .live -- ./verb/lint.d/l1
                        #           →  cargo clippy --workspace --all-targets --all-features -- -D warnings
./verb/lint --dry-run   # prints: runbox .live -- ./verb/lint.d/l1

# Module (claude_profile):
cd module/claude_profile && ./verb/lint
# runs: runbox .live -- ./module/claude_profile/verb/lint.d/l1
#         →  cargo clippy -p claude_profile --all-targets --all-features -- -D warnings

# Extra args forward to clippy, ahead of the `--` separator:
./verb/lint --dry-run --no-deps
# prints: runbox .live -- ./verb/lint.d/l1 --no-deps

# Host-native execution is refused, not silently downgraded. The guard tests for a
# non-empty VERB_LAYER, so *any* value is rejected before dispatch — l0 included:
VERB_LAYER=l1 ./verb/lint   # ERROR: VERB_LAYER is not valid on the host side.   (exit 1)
VERB_LAYER=l0 ./verb/lint   # ERROR: VERB_LAYER is not valid on the host side.   (exit 1)

# The l0 stub is therefore reachable only by executing it directly:
./verb/lint.d/l0            # ERROR: host-native clippy (l0) is disabled.        (exit 1)
```
