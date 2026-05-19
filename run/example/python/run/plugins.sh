#!/usr/bin/env bash
# Project-local plugin overrides for python_lib_test.
# Sourced by runbox-run after SCRIPT_DIR/plugins.sh.

_plugin_list_cmd() {
  list_cmd="/workspace/.venv/bin/pytest --collect-only -q /workspace/tests/"
}
