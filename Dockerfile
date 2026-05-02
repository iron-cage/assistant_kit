# workspace / module test runner  (parameterised via build args)
#
# Three-stage build using cargo-chef for dependency caching.
# Rebuilt layers:
#   chef/planner/cook  — only when Cargo.toml or Cargo.lock change
#   test               — on every source change (fast: deps already compiled)
#
# Build args (values come from run/docker.yml, passed by run/docker-run):
#   COOK_FLAGS  — --workspace | -p claude_profile | -p claude_storage
#   TEST_USER   — testuser (chmod-000 + path-resolution tests) | root
#   HOME_DIR    — /workspace (ClaudePaths + path tests) | /root
#   CMD_SCOPE   — --workspace | -p claude_profile | -p claude_storage
#   CMD_FILTER  — nextest filter expression for offline default CMD
#
# Usage (via script — recommended):
#   run/docker .build                        # workspace image
#   module/claude_profile/run/docker .build  # profile image
#   module/claude_storage/run/docker .build  # storage image
#   run/docker .test                         # full test run
#   run/docker .shell                        # interactive shell
#
# Usage (direct docker — workspace):
#   docker build -f Dockerfile -t workspace_test .
#   docker build -f Dockerfile \
#     --build-arg COOK_FLAGS="-p claude_profile" \
#     --build-arg CMD_SCOPE="-p claude_profile" \
#     --build-arg CMD_FILTER='!test(lim_it)' \
#     -t claude_profile_test .
#   docker run --rm workspace_test                          # offline tests (default CMD)
#   docker run --rm \
#     -v ~/.claude:/workspace/.claude:ro \
#     -v $(which w3):/usr/local/bin/w3:ro \
#     workspace_test \
#     /workspace/run/test                                   # all tests
#   docker run --rm -it workspace_test bash                 # interactive shell

# ── Base: cargo-chef installed once, reused by planner and cook ───────────────

FROM rust:slim AS chef
RUN cargo install cargo-chef --locked

# ── Stage 1: planner — generates recipe.json from the workspace manifests ─────
#
# cargo-chef traverses the workspace tree automatically.
# No per-module Cargo.toml listing required.

FROM chef AS planner
WORKDIR /workspace
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 2: cook — compiles dependencies ─────────────────────────────────────
#
# Receives only recipe.json (not source files).
# This layer is cache-stable: rebuilds only when Cargo.toml or Cargo.lock change,
# not when .rs files change.
# COOK_FLAGS selects which crate(s) to compile deps for.

FROM chef AS cook
ARG COOK_FLAGS=--workspace
WORKDIR /workspace
COPY --from=planner /workspace/recipe.json recipe.json
RUN cargo chef cook \
      --recipe-path recipe.json \
      $COOK_FLAGS \
      --tests

# ── Stage 3: test — compiles crate(s) and runs tests ─────────────────────────
#
# Gets precompiled dep artifacts from cook (avoids recompiling external crates).
# Only workspace/module crates themselves are recompiled here.

FROM rust:slim AS test

# nextest: compile from source for architecture portability (layer is cached).
RUN cargo install cargo-nextest --locked

# clippy: rust:slim ships without it; w3 .test level::3 runs clippy -D warnings.
RUN rustup component add clippy

# System utilities required by tests:
#   procps — provides /bin/kill used by send_sigterm / send_sigkill in claude_core::process
#   curl   — used by claude_version history commands to fetch release data
# rust:slim is intentionally minimal and omits both; tests fail with ENOENT without them.
RUN apt-get update \
 && apt-get install -y --no-install-recommends curl procps \
 && rm -rf /var/lib/apt/lists/*

# TEST_USER: testuser when tests require:
#   - chmod 000 file checks (root bypasses permission → silent wrong-path failures)
#   - path-resolution assertions expecting cwd starts with $HOME
ARG TEST_USER=testuser
RUN [ "$TEST_USER" = "root" ] || useradd -m -s /bin/bash "$TEST_USER"

# HOME_DIR: /workspace so ClaudePaths resolves credentials and session storage
# under /workspace/.claude — a single -v ~/.claude:/workspace/.claude:ro mount
# covers both credentials and session storage for all crates.
ARG HOME_DIR=/workspace
ENV HOME=$HOME_DIR

WORKDIR /workspace

# Precompiled dep artifacts + cargo registry from cook.
# Both are required: target/ has compiled .rlib files; registry/ has crate sources
# that cargo validates during the final link step.
COPY --from=cook /usr/local/cargo/registry /usr/local/cargo/registry
COPY --from=cook /workspace/target         /workspace/target

# Full workspace source (includes run/test scripts invoked by cmd_test).
COPY . .

# Create the seed mount point so Docker initialises the named volume with TEST_USER
# ownership when _ensure_build_cache first mounts it.  Without this mkdir, Docker
# creates /workspace/target_seed as root at container start (path absent from image),
# and testuser cannot write into it — causing the seeding cp -a to fail.
RUN mkdir /workspace/target_seed

# Transfer workspace and cargo home ownership so TEST_USER can compile and run tests.
# chmod a+rwX makes files writable by any uid so cmd_test can run as the host UID
# (--user $(id -u):$(id -g)) to access host-owned ~/.claude credentials, while also
# being able to write build artifacts and cargo lock files as that uid.
RUN [ "$TEST_USER" = "root" ] || ( \
      chown -R "$TEST_USER":"$TEST_USER" /workspace /usr/local/cargo && \
      chmod -R a+rwX /workspace /usr/local/cargo )
USER $TEST_USER

# Offline tests by default — no ~/.claude/ storage or w3 required.
# CMD_SCOPE and CMD_FILTER are baked in at build time from docker.yml values.
ARG CMD_SCOPE=--workspace
ARG CMD_FILTER=!test(lim_it) & !binary(behavior)
CMD cargo nextest run $CMD_SCOPE --filter-expr "$CMD_FILTER"
