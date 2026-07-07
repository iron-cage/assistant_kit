//! Params catalog structural integrity tests
//!
//! ## Purpose
//!
//! Verify `params_catalog()` itself is well-formed: no duplicate names, every
//! entry has at least one observable form, and entries are sorted
//! alphabetically as documented. `lookup()` correctness is covered alongside.
//!
//! ## Coverage
//!
//! - Catalog is non-empty and large enough to match the documented surface
//! - No duplicate `name` values
//! - Every entry has at least one of `cli_flag` / `env_var` / `config_key`
//! - Entries are sorted alphabetically by `name`
//! - `lookup` finds a known entry and returns `None` for an unknown key
//!
//! ## Test Matrix
//!
//! | Test | Scenario |
//! |------|----------|
//! | `catalog_has_minimum_entry_count` | catalog has ≥130 entries |
//! | `catalog_has_no_duplicate_names` | no `name` value repeats |
//! | `catalog_entries_have_at_least_one_form` | every entry has cli_flag/env_var/config_key |
//! | `catalog_is_sorted_alphabetically` | `name` values are non-decreasing |
//! | `lookup_finds_known_entry` | `lookup("model")` returns `Some` |
//! | `lookup_unknown_key_returns_none` | `lookup("NONEXISTENT")` returns `None` |

use claude_version_core::params_catalog::{ lookup, params_catalog };

#[test]
fn catalog_has_minimum_entry_count()
{
  let catalog = params_catalog();
  assert!( catalog.len() >= 130, "expected >= 130 entries, got {}", catalog.len() );
}

#[test]
fn catalog_has_no_duplicate_names()
{
  let catalog = params_catalog();
  let mut names : Vec< &str > = catalog.iter().map( | p | p.name ).collect();
  names.sort_unstable();
  let mut deduped = names.clone();
  deduped.dedup();
  assert_eq!( names.len(), deduped.len(), "catalog contains duplicate param names" );
}

#[test]
fn catalog_entries_have_at_least_one_form()
{
  let catalog = params_catalog();
  for param in catalog
  {
    assert!(
      param.cli_flag.is_some() || param.env_var.is_some() || param.config_key.is_some(),
      "param '{}' has no observable form (cli_flag/env_var/config_key all None)", param.name
    );
  }
}

#[test]
fn catalog_is_sorted_alphabetically()
{
  let catalog = params_catalog();
  let names : Vec< &str > = catalog.iter().map( | p | p.name ).collect();
  let mut sorted = names.clone();
  sorted.sort_unstable();
  assert_eq!( names, sorted, "catalog entries are not sorted alphabetically by name" );
}

#[test]
fn lookup_finds_known_entry()
{
  let param = lookup( "model" );
  assert!( param.is_some(), "expected 'model' to be in the catalog" );
  assert_eq!( param.unwrap().name, "model" );
}

#[test]
fn lookup_unknown_key_returns_none()
{
  assert!( lookup( "NONEXISTENT" ).is_none() );
}
