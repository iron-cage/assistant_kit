# workspace — full test runner
#
# Three-stage build using cargo-chef for dependency caching.
# Rebuilt layers:
#   chef/planner/cook  — only when Cargo.toml or Cargo.lock change
#   test               — on every source change (fast: deps already compiled)
#
# Usage (via script — recommended):
#   run/docker .build          # build image
#   run/docker .test           # all tests (real ~/.claude/ required)
#   run/docker .test.offline   # offline tests only
#   run/docker .shell          # interactive bash shell
#
# Usage (direct docker):
#   docker build -f Dockerfile -t workspace_test .
#   docker run --rm workspace_test                          # offline tests (default CMD)
#   docker run --rm \
#     -v ~/.claude:/workspace/.claude:ro \
#     -v $(which w3):/usr/local/bin/w3:ro \
#     workspace_test \
#     w3 .test level::3                                     # all tests
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
      --workspace \
      --tests

# ── Stage 3: test — compiles all workspace crates and runs tests ───────────────
#
# Gets precompiled dep artifacts from cook (avoids recompiling external crates).
# Only workspace crates themselves are recompiled here.

FROM rust:slim AS test

# nextest: compile from source for architecture portability (layer is cached).
RUN cargo install cargo-nextest --locked

# Non-root user: claude_storage tests use chmod 000 — root bypasses permission checks,
# causing those tests to silently pass the wrong code path.
RUN useradd -m -s /bin/bash testuser

# Path resolution tests in claude_storage assert cwd starts with $HOME.
# HOME=/workspace satisfies this for all /workspace/... paths.
# This also causes ClaudePaths to resolve credentials and storage under /workspace/.claude,
# so a single -v ~/.claude:/workspace/.claude:ro mount covers all crates.
ENV HOME=/workspace

WORKDIR /workspace

# Precompiled dep artifacts + cargo registry from cook.
# Both are required: target/ has compiled .rlib files; registry/ has crate sources
# that cargo validates during the final link step.
COPY --from=cook /usr/local/cargo/registry /usr/local/cargo/registry
COPY --from=cook /workspace/target         /workspace/target

# Full workspace source.
COPY . .

# Transfer workspace and cargo home ownership so testuser can compile and run tests.
RUN chown -R testuser:testuser /workspace /usr/local/cargo

USER testuser

# Offline tests by default — no ~/.claude/ storage or credentials required.
# Excludes lim_it* (claude_profile live API calls) and behavior binary (claude_storage real-storage tests).
#
# To run all tests, mount ~/.claude/ and w3, then use w3 .test:
#   docker run -v ~/.claude:/workspace/.claude:ro \
#              -v $(which w3):/usr/local/bin/w3:ro \
#              workspace_test \
#              w3 .test level::3
CMD [ "cargo", "nextest", "run", \
      "--workspace", \
      "--filter-expr", "!test(lim_it) & !binary(behavior)" ]
