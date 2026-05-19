#!/usr/bin/env bash
# Project-local plugin overrides for nodejs_example_test.
# Sourced by runbox-run after SCRIPT_DIR/plugins.sh.

_plugin_list_cmd() {
  list_cmd="node --test --test-reporter=spec /workspace/tests/"
}
