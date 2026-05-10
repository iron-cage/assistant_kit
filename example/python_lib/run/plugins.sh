#!/usr/bin/env bash
# Python project plugin overrides for runbox-run.
# Sourced after dev/run/plugins.sh — overrides only what differs for Python.

_plugin_list_cmd() {
  list_cmd=".venv/bin/pytest --collect-only -q tests/"
}
