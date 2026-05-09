# Runbox Parameters

Scalar values that configure fixed slots in the runbox infrastructure.
When a parameter changes, the execution shape stays the same — only the value at that slot changes.

See `plugins.md` for swappable behavioral components.

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Configured — present in `runbox.yml` |
| 🔒 | Hardcoded — in `runbox.dockerfile` or `docker-run`, not yet configurable |

## Table

| Parameter | Status | Current State | Where It Flows | Notes |
|-----------|--------|---------------|----------------|-------|
| `image` | ✅ | per-module tag | `docker build -t`, `docker run` | Unique per module |
| `test_user` | ✅ | `testuser` | `ARG TEST_USER` in dockerfile | Non-root for chmod-000 and path-resolution tests |
| `cmd_scope` | ✅ | `--workspace` / `-p crate` | cook `ARG CMD_SCOPE`, nextest run | Single source of truth for dep precompile scope and test scope |
| `cmd_filter` | ✅ | filter expression | Baked into image `CMD` | Offline-safe default; excludes tests that need plugins |
| `test_script` | ✅ | module-relative path | `docker run /workspace/$TEST_SCRIPT` | Full-test entrypoint; may invoke bin plugins |
| `base_image` | 🔒 | `rust:slim` | `FROM` in all four stages | Version-unpinned; identical string baked into chef and test stages |
| `rustup_components` | 🔒 | `clippy` | `rustup component add` in test stage | Single component hardcoded; other projects need `rustfmt`, `llvm-tools-preview` |
| `system_packages` | 🔒 | `curl procps` | `apt-get install` in test stage | Project-specific: `curl` for version history, `procps` for kill; other projects differ |
| `cargo_features` | 🔒 | `--all-features` | nextest list flags in `cmd_list` | Hardcoded in docker-run; projects with conflicting features need `--no-default-features -F …` |
| `workspace_dir` | 🔒 | `/workspace` | `WORKDIR`, all volume paths, `ENV HOME` | Baked into both dockerfile and docker-run; both must change together |
