//! File content templates for generated container runner integration files.
//!
//! All functions return complete file content as owned or static strings.
//! Templates are pure functions — no filesystem access.

/// Known project ecosystems for which specific Dockerfile templates exist.
#[ derive( Debug, Clone, PartialEq ) ]
pub enum Ecosystem
{
  /// Rust / Cargo projects — `target/` cache directory.
  Rust,
  /// Node.js / npm projects — `node_modules/` cache directory.
  Nodejs,
  /// Python / venv projects — `.venv/` cache directory.
  Python,
  /// Ecosystem not specified — `.cache/` placeholder cache directory.
  None,
}

impl Ecosystem
{
  /// Parse an ecosystem name string.
  ///
  /// Returns `None` for unknown names.
  #[ must_use ]
  #[ inline ]
  pub fn from_name( s : &str ) -> Option< Self >
  {
    match s
    {
      "rust"   => Some( Ecosystem::Rust   ),
      "nodejs" => Some( Ecosystem::Nodejs ),
      "python" => Some( Ecosystem::Python ),
      "none"   => Some( Ecosystem::None   ),
      _        => None,
    }
  }

  /// Build cache directory for this ecosystem.
  #[ must_use ]
  #[ inline ]
  pub fn cache_dir( &self ) -> &'static str
  {
    match self
    {
      Ecosystem::Rust   => "target",
      Ecosystem::Nodejs => "node_modules",
      Ecosystem::Python => ".venv",
      Ecosystem::None   => ".cache",
    }
  }

  /// Canonical name string.
  #[ must_use ]
  #[ inline ]
  pub fn as_str( &self ) -> &'static str
  {
    match self
    {
      Ecosystem::Rust   => "rust",
      Ecosystem::Nodejs => "nodejs",
      Ecosystem::Python => "python",
      Ecosystem::None   => "none",
    }
  }
}

/// Returns the standard walk-up discovery wrapper script content.
///
/// Identical for every project at any directory depth — the discovery
/// logic finds the runner binary by walking up the directory tree.
#[ must_use ]
#[ inline ]
pub fn wrapper_script() -> &'static str
{
  r#"#!/usr/bin/env bash
# Container runner wrapper — auto-discovers the runner by walking up the directory tree.
# Copy verbatim to any project's runbox/ directory; no path calculation needed.
set -euo pipefail

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

_find_runbox_run() {
  local dir="$1"
  while [[ "$dir" != "/" ]]; do
    [[ -x "$dir/runbox/runbox-run" ]] && echo "$dir/runbox/runbox-run" && return
    dir="$( dirname "$dir" )"
  done
  echo "error: runbox-run not found in any parent directory" >&2
  exit 1
}

exec "$(_find_runbox_run "$SCRIPT_DIR")" "$SCRIPT_DIR/runbox.yml" "$@"
"#
}

/// Returns `runbox.yml` config file content for the given project parameters.
#[ must_use ]
#[ inline ]
pub fn runbox_yml( image : &str, eco : &Ecosystem, test_script : &str ) -> String
{
  format!(
    "image: {image}\n\
     dockerfile: runbox.dockerfile\n\
     cache_dir: {cache_dir}\n\
     workspace_root: ..\n\
     test_script: {test_script}\n",
    image       = image,
    cache_dir   = eco.cache_dir(),
    test_script = test_script,
  )
}

/// Returns the Dockerfile content appropriate for the given ecosystem.
#[ must_use ]
#[ inline ]
pub fn dockerfile( eco : &Ecosystem ) -> &'static str
{
  match eco
  {
    Ecosystem::Rust => r#"FROM rust:latest

ARG WORKSPACE_DIR=/workspace
WORKDIR $WORKSPACE_DIR

COPY . .

RUN cargo build && cargo test --no-run

RUN mkdir target_seed && chmod -R a+rwX target target_seed

CMD ["cargo", "test"]
"#,

    Ecosystem::Nodejs => r#"FROM node:22-slim

ARG WORKSPACE_DIR=/workspace
WORKDIR $WORKSPACE_DIR

COPY . .

RUN npm install

RUN mkdir node_modules_seed && chmod -R a+rwX node_modules node_modules_seed

CMD ["npm", "test"]
"#,

    Ecosystem::Python => r#"FROM python:3.12-slim

ARG WORKSPACE_DIR=/workspace
WORKDIR $WORKSPACE_DIR

COPY . .

RUN python -m venv .venv && .venv/bin/pip install --no-cache-dir .[dev]

RUN mkdir .venv_seed && chmod -R a+rwX .venv .venv_seed

CMD [".venv/bin/pytest", "tests/", "-v"]
"#,

    Ecosystem::None => r#"FROM ubuntu:22.04

ARG WORKSPACE_DIR=/workspace
WORKDIR $WORKSPACE_DIR

COPY . .

RUN mkdir .cache .cache_seed && chmod -R a+rwX .cache .cache_seed

CMD ["/bin/bash", "-c", "echo 'No default test command configured'"]
"#,
  }
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  #[ test ]
  fn wrapper_script_contains_discovery_logic()
  {
    let s = wrapper_script();
    assert!( s.contains( "_find_runbox_run" ), "must contain discovery function" );
    assert!( s.starts_with( "#!/usr/bin/env bash" ), "must have shebang" );
    assert!( s.contains( "runbox.yml" ), "must reference runbox.yml" );
  }

  #[ test ]
  fn runbox_yml_contains_all_required_fields()
  {
    let yml = runbox_yml( "my_image", &Ecosystem::Rust, "verb/test.d/l1" );
    assert!( yml.contains( "image:" ),          "must have image field" );
    assert!( yml.contains( "dockerfile:" ),     "must have dockerfile field" );
    assert!( yml.contains( "cache_dir:" ),      "must have cache_dir field" );
    assert!( yml.contains( "workspace_root:" ), "must have workspace_root field" );
    assert!( yml.contains( "test_script:" ),    "must have test_script field" );
    assert!( yml.contains( "my_image" ),        "must include image value" );
    assert!( yml.contains( "target" ),          "rust cache_dir must be target" );
    assert!( yml.contains( "verb/test.d/l1" ),  "must include test_script value" );
  }

  #[ test ]
  fn cache_dir_per_ecosystem()
  {
    assert_eq!( Ecosystem::Rust.cache_dir(),   "target"       );
    assert_eq!( Ecosystem::Nodejs.cache_dir(), "node_modules" );
    assert_eq!( Ecosystem::Python.cache_dir(), ".venv"        );
    assert_eq!( Ecosystem::None.cache_dir(),   ".cache"       );
  }

  #[ test ]
  fn dockerfile_rust_uses_rust_latest()
  {
    assert!( dockerfile( &Ecosystem::Rust ).contains( "FROM rust:latest" ) );
  }

  #[ test ]
  fn dockerfile_nodejs_uses_node22()
  {
    assert!( dockerfile( &Ecosystem::Nodejs ).contains( "FROM node:22-slim" ) );
  }

  #[ test ]
  fn dockerfile_python_uses_python312()
  {
    assert!( dockerfile( &Ecosystem::Python ).contains( "FROM python:3.12-slim" ) );
  }

  #[ test ]
  fn dockerfile_none_uses_ubuntu()
  {
    assert!( dockerfile( &Ecosystem::None ).contains( "FROM ubuntu:22.04" ) );
  }

  #[ test ]
  fn ecosystem_from_name_roundtrip()
  {
    assert_eq!( Ecosystem::from_name( "rust"   ), Some( Ecosystem::Rust   ) );
    assert_eq!( Ecosystem::from_name( "nodejs" ), Some( Ecosystem::Nodejs ) );
    assert_eq!( Ecosystem::from_name( "python" ), Some( Ecosystem::Python ) );
    assert_eq!( Ecosystem::from_name( "none"   ), Some( Ecosystem::None   ) );
    assert_eq!( Ecosystem::from_name( "java"   ), None                      );
    assert_eq!( Ecosystem::from_name( ""       ), None                      );
  }
}
