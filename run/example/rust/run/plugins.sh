#!/usr/bin/env bash
# Project-local plugin overrides for rust_example_test.
# Sourced by runbox-run after SCRIPT_DIR/plugins.sh.

_plugin_list_cmd() {
  list_cmd="cargo test --manifest-path /workspace/Cargo.toml -- --list 2>/dev/null | grep ': test'"
}
