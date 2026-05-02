# claude_profile — test runner
#
# Three-stage build using cargo-chef for dependency caching.
# Rebuilt layers:
#   chef/planner/cook  — only when Cargo.toml or Cargo.lock change
#   test               — on every source change (fast: deps already compiled)
#
# Usage (via script — recommended):
#   module/claude_profile/run/docker .build          # build image
#   module/claude_profile/run/docker .test           # all tests (credentials required)
#   module/claude_profile/run/docker .test.offline   # offline tests only
#
# Usage (direct docker):
#   docker build -f module/claude_profile/Dockerfile -t claude_profile_test .
#   docker run --rm claude_profile_test                  # offline tests (default CMD)
#   docker run --rm \
#     -v ~/.claude/.credentials.json:/root/.claude/.credentials.json:ro \
#     -v $(which w3):/usr/local/bin/w3:ro \
#     claude_profile_test \
#     bash -c "cd /workspace/module/claude_profile && w3 .test level::3"  # all tests

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
      -p claude_profile \
      --tests

# ── Stage 3: test — compiles claude_profile itself and runs tests ─────────────
#
# Gets precompiled dep artifacts from cook (avoids recompiling external crates).
# Only claude_profile and its local workspace deps are recompiled here.

FROM rust:slim AS test

# nextest: compile from source for architecture portability (layer is cached).
RUN cargo install cargo-nextest --locked

WORKDIR /workspace

# Precompiled dep artifacts + cargo registry from cook.
# Both are required: target/ has compiled .rlib files; registry/ has crate sources
# that cargo validates during the final link step.
COPY --from=cook /usr/local/cargo/registry /usr/local/cargo/registry
COPY --from=cook /workspace/target         /workspace/target

# Full workspace source. Only claude_profile and its local workspace deps rebuild.
COPY . .

# Offline tests by default — no credentials or outbound network required.
# The four lim_it* tests (live API calls) are excluded.
#
# To run all tests (including live), mount credentials and w3, then use w3 .test:
#   docker run -v ~/.claude/.credentials.json:/root/.claude/.credentials.json:ro \
#              -v $(which w3):/usr/local/bin/w3:ro \
#              claude_profile_test \
#              bash -c "cd /workspace/module/claude_profile && w3 .test level::3"
CMD [ "cargo", "nextest", "run", \
      "-p", "claude_profile", \
      "--filter-expr", "!test(lim_it)" ]
