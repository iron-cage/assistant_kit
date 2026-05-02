# claude_storage — test runner
#
# Three-stage build using cargo-chef for dependency caching.
# Rebuilt layers:
#   chef/planner/cook  — only when Cargo.toml or Cargo.lock change
#   test               — on every source change (fast: deps already compiled)
#
# Usage (via script — recommended):
#   module/claude_storage/scripts/docker .build          # build image
#   module/claude_storage/scripts/docker .test           # all tests (real ~/.claude/ required)
#   module/claude_storage/scripts/docker .test.offline   # offline tests only
#
# Usage (direct docker):
#   docker build -f module/claude_storage/Dockerfile -t claude_storage_test .
#   docker run --rm claude_storage_test                  # offline tests (default CMD)
#   docker run --rm \
#     -v ~/.claude:/workspace/.claude:ro \
#     claude_storage_test \
#     cargo nextest run -p claude_storage                # all tests

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

# ── Stage 2: cook — compiles all dependencies ─────────────────────────────────
#
# Receives only recipe.json (not source files).
# This layer is cache-stable: rebuilds only when Cargo.toml or Cargo.lock change,
# not when .rs files change.

FROM chef AS cook
WORKDIR /workspace
COPY --from=planner /workspace/recipe.json recipe.json
RUN cargo chef cook \
      --recipe-path recipe.json \
      -p claude_storage \
      --tests

# ── Stage 3: test — compiles claude_storage itself and runs tests ──────────────
#
# Gets precompiled dep artifacts from cook (avoids recompiling external crates).
# Only claude_storage and its local workspace deps are recompiled here.

FROM rust:slim AS test

# nextest: compile from source for architecture portability (layer is cached).
RUN cargo install cargo-nextest --locked

# Non-root user: permission-based tests (chmod 000) require non-root execution.
# Root bypasses file permission checks, causing those tests to silently pass the
# wrong code path (no warning emitted for "unreadable" sessions).
RUN useradd -m -s /bin/bash testuser

# Path resolution tests assert cwd starts with $HOME (mirrors dev-machine layout).
# Set HOME to workspace root so /workspace/... satisfies that precondition.
ENV HOME=/workspace

WORKDIR /workspace

# Precompiled dep artifacts + cargo registry from cook.
# Both are required: target/ has compiled .rlib files; registry/ has crate sources
# that cargo validates during the final link step.
COPY --from=cook /usr/local/cargo/registry /usr/local/cargo/registry
COPY --from=cook /workspace/target         /workspace/target

# Full workspace source. Only claude_storage and its local workspace deps rebuild.
COPY . .

# Transfer workspace and cargo home ownership so testuser can compile and run tests.
RUN chown -R testuser:testuser /workspace /usr/local/cargo

USER testuser

# Offline tests by default — no ~/.claude/ storage required.
# The behavior/* tests (B1..B18) inspect real ~/.claude/ and are excluded.
#
# To run all tests (including behavior), mount ~/.claude/ and drop the filter:
#   docker run -v ~/.claude:/workspace/.claude:ro \
#              claude_storage_test \
#              cargo nextest run -p claude_storage
CMD [ "cargo", "nextest", "run", \
      "-p", "claude_storage", \
      "--filter-expr", "!binary(behavior)" ]
