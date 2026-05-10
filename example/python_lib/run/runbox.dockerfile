FROM python:3.12-slim

# runbox-run passes these build args for all ecosystems; declare to avoid warnings.
ARG WORKSPACE_DIR=/workspace
ARG TEST_USER=root
ARG BASE_IMAGE
ARG CMD_SCOPE
ARG CMD_FILTER
ARG RUSTUP_COMPONENTS
ARG SYSTEM_PACKAGES
ARG CARGO_FEATURES

WORKDIR $WORKSPACE_DIR

COPY . .

# Install project + dev dependencies into a venv (the cache_dir for this ecosystem).
# The venv is seeded into a named Docker volume so .test.offline skips re-downloading.
RUN python -m venv .venv && .venv/bin/pip install --no-cache-dir .[dev]

# Seed mount point: runbox-run copies .venv → .venv_seed on first run,
# then mounts the named volume back at .venv for .test.offline and .shell.
RUN mkdir .venv_seed

CMD [".venv/bin/pytest", "tests/", "-v"]
