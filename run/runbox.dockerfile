# workspace / module test runner  (parameterised via build args)
#
# Why cargo-chef:
#   On a local machine, cargo keeps a persistent target/ directory.  Changing a .rs
#   file only recompiles the affected crate and its dependents — external crates are
#   untouched.  Docker breaks this: there is no persistent target/ between builds.
#   Any .rs change invalidates the COPY . . layer, so the following cargo command
#   starts from an empty target/ and recompiles every external dependency from
#   scratch — even for a one-line change in your own code.
#
#   cargo-chef restores the local-dev behaviour inside Docker by splitting
#   compilation into two separate layers:
#     1. cook  — compiles all external deps from recipe.json (derived from
#                Cargo.toml/Cargo.lock only, so .rs changes never touch it)
#     2. test  — compiles only workspace crates (deps already done in cook)
#   Docker caches the cook layer indefinitely; only the test layer reruns on
#   source changes.  Net effect: same incremental speed as a local cargo build.
#
#   What crosses stage boundaries:
#
#     planner ──► recipe.json
#                   dep manifest derived purely from Cargo.toml + Cargo.lock;
#                   contains no .rs content, so it never changes on source edits
#
#     cook    ──► /workspace/target/debug/deps/*.rlib
#                   compiled external crate artifacts (serde, tokio, anyhow, …)
#                 /usr/local/cargo/registry/
#                   downloaded crate source archives (cargo re-validates these
#                   during the final link step; omitting them causes link errors)
#
#     test    ──► receives both cook outputs, then:
#                 COPY . . adds the full workspace source on top
#
#   Filesystem when cargo runs in the test stage:
#
#     /workspace/
#     ├── Cargo.toml, Cargo.lock     ← COPY . .
#     ├── src/ module/ …             ← COPY . .  (workspace source)
#     └── target/debug/deps/         ← from cook  (external deps pre-built)
#             ├── libserde-*.rlib         ✓ already compiled — skipped
#             ├── libtokio-*.rlib         ✓ already compiled — skipped
#             └── …                       ✓ all external crates — skipped
#
#     /usr/local/cargo/registry/     ← from cook  (source archives for linking)
#
#   cargo sees a populated target/debug/deps/ and skips every external crate.
#   It compiles only the workspace crates (absent from deps/), then links.
#   This is identical to what cargo does on a local machine after the first build.
#
# Stages:
#   A Docker stage is one isolated build environment started by a FROM instruction.
#   Each stage gets its own filesystem; the only way to move data between stages is
#   an explicit COPY --from=<stage>.  The final image contains only the last stage —
#   all intermediate stages are discarded (they exist only to produce artifacts for
#   downstream stages to cherry-pick).
#
#   Stage    Base        cargo-chef    Purpose
#   ──────── ─────────── ────────────  ────────────────────────────────────────────
#   chef     rust:slim   installs it   shared base — compiled once, inherited below
#   planner  chef        inherited     scans workspace manifests → emits recipe.json
#   cook     chef        inherited     compiles all external dep .rlib from recipe.json
#   test     rust:slim   absent        final image — receives cook artifacts + source
#
#   Stage    Receives                         Produces                       In image?
#   ──────── ───────────────────────────────  ─────────────────────────────  ─────────
#   chef     rust:slim                        rust:slim + cargo-chef binary  no
#   planner  chef + COPY . . (full source)    recipe.json                    no
#   cook     chef + recipe.json               target/debug/deps/ + registry  no
#   test     rust:slim + cook dirs + source   the runnable image             YES
#
#   Data flow:
#
#     rust:slim
#         │
#         ▼
#     ┌─ chef ──────────────────────────────────────┐
#     │  RUN cargo install cargo-chef               │
#     └──────────┬──────────────────────┬───────────┘
#                │                      │
#                ▼                      ▼
#     ┌─ planner ────────┐    ┌─ cook ───────────────────────────┐
#     │  COPY . .        │    │  COPY --from=planner recipe.json │
#     │  chef prepare    │    │  chef cook --tests               │
#     └────────┬─────────┘    └──────────────────┬───────────────┘
#              │ recipe.json                      │ target/debug/deps/*.rlib
#              └──────────────────────────────────┘ /usr/local/cargo/registry/
#                                                  │
#                                                  ▼  COPY --from=cook
#                                        ┌─ test ───────────────────────────┐
#                                        │  FROM rust:slim                  │
#                                        │  + nextest, clippy, curl, procps │
#                                        │  + cook artifacts (COPY --from=cook)│
#                                        │  + COPY . .  (full source)       │
#                                        │  ── final runnable image ─────── │
#                                        └──────────────────────────────────┘
#
# Rebuilt layers:
#   chef/planner/cook  — only when Cargo.toml or Cargo.lock change
#   test               — on every source change (fast: deps already compiled)
#
# Build args (values come from run/runbox.yml, passed by run/runbox-run):
#   BASE_IMAGE         — FROM image for chef and test stages (default: rust:slim)
#   TEST_USER          — non-root user for chmod-000 / path-resolution tests (default: testuser)
#   CMD_SCOPE          — --workspace | -p claude_profile | -p claude_storage
#   CMD_FILTER         — nextest filter expression for offline default CMD
#   RUSTUP_COMPONENTS  — space-separated rustup components to add (default: clippy)
#   SYSTEM_PACKAGES    — space-separated apt packages; empty string skips install (default: '')
#   CARGO_FEATURES     — feature flags passed to nextest (default: --all-features)
#   WORKSPACE_DIR      — container WORKDIR; must match in runbox-run (default: /workspace)
#
# Usage (via script — recommended):
#   run/runbox .build                        # workspace image
#   module/claude_profile/run/runbox .build  # profile image
#   run/runbox .test                         # full test run
#   run/runbox .shell                        # interactive shell
#
# Usage (direct docker — workspace):
#   docker build -f run/runbox.dockerfile -t workspace_test .
#   docker run --rm workspace_test                          # offline tests (default CMD)
#   docker run --rm \
#     -v ~/.claude:/workspace/.claude:rw \
#     -v $(which w3):/usr/local/bin/w3:ro \
#     workspace_test \
#     /workspace/run/test                                   # all tests (plugin mounts required)
#   docker run --rm -it workspace_test bash                 # interactive shell

# ── Base: cargo-chef installed once, reused by planner and cook ───────────────
# Pre-FROM ARG: available in all FROM instructions in this file.

ARG BASE_IMAGE=rust:slim

FROM $BASE_IMAGE AS chef
RUN cargo install cargo-chef --locked

# ── Stage 1: planner — generates recipe.json from the workspace manifests ─────
#
# cargo-chef traverses the workspace tree automatically.
# No per-module Cargo.toml listing required.

FROM chef AS planner
ARG WORKSPACE_DIR=/workspace
WORKDIR $WORKSPACE_DIR
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 2: cook — compiles dependencies ─────────────────────────────────────
#
# Receives only recipe.json (not source files).
# This layer is cache-stable: rebuilds only when Cargo.toml or Cargo.lock change,
# not when .rs files change.
# CMD_SCOPE scopes which crates' deps to precompile — the same value drives nextest run.

FROM chef AS cook
ARG WORKSPACE_DIR=/workspace
ARG CMD_SCOPE=--workspace
WORKDIR $WORKSPACE_DIR
COPY --from=planner $WORKSPACE_DIR/recipe.json recipe.json
RUN cargo chef cook \
      --recipe-path recipe.json \
      $CMD_SCOPE \
      --tests

# ── Stage 3: test — compiles crate(s) and runs tests ─────────────────────────
#
# Gets precompiled dep artifacts from cook (avoids recompiling external crates).
# Only workspace/module crates themselves are recompiled here.

FROM $BASE_IMAGE AS test

ARG WORKSPACE_DIR=/workspace
ARG RUSTUP_COMPONENTS=clippy
ARG SYSTEM_PACKAGES=curl procps
ARG CARGO_FEATURES=--all-features

# nextest: compile from source for architecture portability (layer is cached).
RUN cargo install cargo-nextest --locked

# clippy: rust:slim ships without it; w3 .test level::3 runs clippy -D warnings.
RUN rustup component add $RUSTUP_COMPONENTS

# System utilities required by tests:
#   procps — provides /bin/kill used by send_sigterm / send_sigkill in claude_core::process
#   curl   — used by claude_version history commands to fetch release data
# rust:slim is intentionally minimal and omits both; tests fail with ENOENT without them.
RUN [ -z "$SYSTEM_PACKAGES" ] || ( \
      apt-get update \
      && apt-get install -y --no-install-recommends $SYSTEM_PACKAGES \
      && rm -rf /var/lib/apt/lists/* )

# TEST_USER: testuser when tests require:
#   - chmod 000 file checks (root bypasses permission → silent wrong-path failures)
#   - path-resolution assertions expecting cwd starts with $HOME
ARG TEST_USER=testuser
RUN [ "$TEST_USER" = "root" ] || useradd -m -s /bin/bash "$TEST_USER"

# HOME=$WORKSPACE_DIR so ClaudePaths resolves .claude/ under the workspace — plugin mounts land there.
ENV HOME=$WORKSPACE_DIR

WORKDIR $WORKSPACE_DIR

# Precompiled dep artifacts + cargo registry from cook.
# Both are required: target/ has compiled .rlib files; registry/ has crate sources
# that cargo validates during the final link step.
COPY --from=cook /usr/local/cargo/registry /usr/local/cargo/registry
COPY --from=cook $WORKSPACE_DIR/target     $WORKSPACE_DIR/target

# Full workspace source (includes test_script paths invoked by cmd_test).
COPY . .

# Create the seed mount point so Docker initialises the named volume with TEST_USER
# ownership when _ensure_build_cache first mounts it.  Without this mkdir, Docker
# creates $WORKSPACE_DIR/target_seed as root at container start (path absent from image),
# and testuser cannot write into it — causing the seeding cp -a to fail.
RUN mkdir $WORKSPACE_DIR/target_seed

# Transfer workspace and cargo home ownership so TEST_USER can compile and run tests.
# chmod a+rwX makes files writable by any uid so cmd_test can run as the host UID
# (--user $(id -u):$(id -g)) to access host-owned ~/.claude credentials, while also
# being able to write build artifacts and cargo lock files as that uid.
RUN [ "$TEST_USER" = "root" ] || ( \
      chown -R "$TEST_USER":"$TEST_USER" $WORKSPACE_DIR /usr/local/cargo && \
      chmod -R a+rwX $WORKSPACE_DIR /usr/local/cargo )
USER $TEST_USER

# Offline tests by default — no ~/.claude/ storage or w3 required.
# CMD_SCOPE and CMD_FILTER are baked in at build time from runbox.yml values.
ARG CMD_SCOPE=--workspace
ARG CMD_FILTER=!test(lim_it) & !binary(behavior)
CMD cargo nextest run $CMD_SCOPE $CARGO_FEATURES --filter-expr "$CMD_FILTER"
