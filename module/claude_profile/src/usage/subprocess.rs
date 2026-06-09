//! Subprocess model and effort resolution for the `apply_touch` pipeline.
//!
//! `resolve_model` maps `imodel::` + quota data → `IsolatedModel`;
//! `resolve_effort` maps the resolved model + `effort::` → optional effort flag;
//! `effort_pre_args` assembles the `--effort` arg slice for subprocess dispatch.

use super::types::{ AccountQuota, SubprocessModel, SubprocessEffort };

// ── Model resolution ──────────────────────────────────────────────────────────

/// Resolve the subprocess model for one account based on `imodel::` and quota data.
///
/// AC-01: `auto` selects Haiku — subprocess operations are keep-alive pings,
///        Haiku is sufficient and conserves quota.
/// AC-02: `sonnet` always maps to `claude-sonnet-4-6`.
/// AC-03: `opus` always maps to `claude-opus-4-6`.
/// AC-04: `keep` passes `IsolatedModel::KeepCurrent` — no `--model` flag injected.
/// AC-13: `haiku` always maps to `claude-haiku-4-5-20251001`.
#[ inline ]
pub( crate ) fn resolve_model( _aq : &AccountQuota, imodel : SubprocessModel ) -> claude_runner_core::IsolatedModel
{
  use claude_runner_core::IsolatedModel;
  match imodel
  {
    SubprocessModel::Sonnet => IsolatedModel::Specific( "claude-sonnet-4-6".to_string() ),
    SubprocessModel::Opus   => IsolatedModel::Specific( "claude-opus-4-6".to_string() ),
    SubprocessModel::Keep   => IsolatedModel::KeepCurrent,
    SubprocessModel::Haiku  => IsolatedModel::Specific( "claude-haiku-4-5-20251001".to_string() ),
    SubprocessModel::Auto   =>
    {
      // AC-01: Haiku is sufficient for keep-alive pings (touch/refresh) — conserves quota.
      IsolatedModel::Specific( "claude-haiku-4-5-20251001".to_string() )
    }
  }
}

// ── Effort resolution ─────────────────────────────────────────────────────────

/// Resolve the `--effort` flag value for a subprocess given the resolved model.
///
/// Returns `None` when no `--effort` flag should be injected.
/// AC-05: `auto` → `low` for any model that supports effort (Sonnet, Opus); `None` for Haiku or `KeepCurrent`.
///         Haiku has no extended thinking; injecting `--effort` would have no effect or API error.
///         `KeepCurrent` → `None` (model unknown at dispatch time).
/// AC-06: `high` always injects `--effort high`.
/// AC-07: `max` always injects `--effort max`.
/// AC-14: `low` always injects `--effort low`.
/// AC-15: `normal` always injects `--effort normal`.
#[ inline ]
pub( crate ) fn resolve_effort( model : &claude_runner_core::IsolatedModel, effort : SubprocessEffort ) -> Option< &'static str >
{
  use claude_runner_core::IsolatedModel;
  match effort
  {
    SubprocessEffort::High   => Some( "high" ),
    SubprocessEffort::Max    => Some( "max" ),
    SubprocessEffort::Low    => Some( "low" ),
    SubprocessEffort::Normal => Some( "normal" ),
    SubprocessEffort::Auto => match model
    {
      IsolatedModel::Specific( m ) if m.as_str() == "claude-haiku-4-5-20251001" => None,
      IsolatedModel::Specific( _ )                                               => Some( "low" ),
      IsolatedModel::KeepCurrent | IsolatedModel::Default                       => None,
    },
  }
}

/// Build the `extra_pre_args` slice to prepend before `["--print", "."]` in a subprocess.
///
/// Returns `["--effort", value]` when effort resolves to `Some`, otherwise an empty vec.
#[ inline ]
pub( crate ) fn effort_pre_args( model : &claude_runner_core::IsolatedModel, effort : SubprocessEffort ) -> Vec< String >
{
  match resolve_effort( model, effort )
  {
    Some( e ) => vec![ "--effort".to_string(), e.to_string() ],
    None      => vec![],
  }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[ cfg( test ) ]
mod tests
{
  use super::*;
  use crate::usage::test_support::{ mk_aq_with_sonnet_util, mk_aq_no_sonnet_data, mk_aq_err };

  // ── resolve_model ──────────────────────────────────────────────────────────

  /// FT-01: `imodel::auto` selects haiku regardless of quota state.
  ///
  /// Subprocess keep-alive operations don't need expensive models; Haiku conserves quota.
  /// Quota data is read but not used — any quota percentage yields haiku.
  ///
  /// Spec: [`tests/docs/feature/026_subprocess_model_effort.md` FT-01]
  #[ test ]
  fn it_imodel_auto_selects_haiku()
  {
    let aq       = mk_aq_with_sonnet_util( 85.0 );
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-haiku-4-5-20251001",
      "imodel::auto must always select haiku for keep-alive operations",
    );
  }

  /// FT-04: `imodel::auto` with absent quota data → haiku (quota not needed).
  ///
  /// Spec: [`tests/docs/feature/026_subprocess_model_effort.md` FT-04]
  #[ test ]
  fn it_imodel_auto_selects_haiku_without_quota_data()
  {
    let aq       = mk_aq_no_sonnet_data();
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-haiku-4-5-20251001",
      "imodel::auto with absent quota data must still select haiku",
    );
  }

  /// EC-9a: `imodel::auto` with account error result → haiku (quota data irrelevant).
  #[ test ]
  fn it_imodel_auto_selects_haiku_with_err_result()
  {
    let aq       = mk_aq_err();
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-haiku-4-5-20251001",
      "imodel::auto with Err result must select haiku (quota data not needed)",
    );
  }

  /// Auto model + `effort::auto` → `None` (haiku does not support extended thinking).
  ///
  /// End-to-end: `resolve_model(aq, Auto)` returns haiku; `resolve_effort(haiku, Auto)` → `None`.
  #[ test ]
  fn it_effort_auto_resolves_none_for_auto_haiku()
  {
    let aq     = mk_aq_no_sonnet_data();
    let model  = resolve_model( &aq, SubprocessModel::Auto );
    let effort = resolve_effort( &model, SubprocessEffort::Auto );
    assert!(
      effort.is_none(),
      "imodel::auto + effort::auto must produce None effort (haiku has no extended thinking), got: {effort:?}",
    );
  }

  /// EC-6: `imodel::sonnet` always returns `IsolatedModel::Specific("claude-sonnet-4-6")`.
  ///
  /// Spec: [`tests/docs/cli/param/035_imodel.md` EC-6]
  #[ test ]
  fn it_imodel_sonnet_explicit()
  {
    let aq       = mk_aq_no_sonnet_data();
    let model    = resolve_model( &aq, SubprocessModel::Sonnet );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!( model_id, "claude-sonnet-4-6", "imodel::sonnet must always return claude-sonnet-4-6" );
  }

  /// EC-7: `imodel::opus` always returns `IsolatedModel::Specific("claude-opus-4-6")`.
  ///
  /// Spec: [`tests/docs/cli/param/035_imodel.md` EC-7]
  #[ test ]
  fn it_imodel_opus_explicit()
  {
    let aq       = mk_aq_no_sonnet_data();
    let model    = resolve_model( &aq, SubprocessModel::Opus );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!( model_id, "claude-opus-4-6", "imodel::opus must always return claude-opus-4-6" );
  }

  /// EC-8: `imodel::keep` returns `IsolatedModel::KeepCurrent` — no `--model` flag.
  ///
  /// Spec: [`tests/docs/cli/param/035_imodel.md` EC-8]
  #[ test ]
  fn it_imodel_keep_no_model_flag()
  {
    let aq    = mk_aq_no_sonnet_data();
    let model = resolve_model( &aq, SubprocessModel::Keep );
    assert!(
      matches!( model, claude_runner_core::IsolatedModel::KeepCurrent ),
      "imodel::keep must return KeepCurrent (no --model flag)",
    );
  }

  // ── resolve_effort ────────────────────────────────────────────────────────

  /// `effort::high` always returns `Some("high")` regardless of model.
  ///
  /// Spec: [`tests/docs/cli/param/036_effort.md` EC-5]
  #[ test ]
  fn it_effort_high_explicit()
  {
    let sonnet = claude_runner_core::IsolatedModel::Specific( "claude-sonnet-4-6".to_string() );
    let opus   = claude_runner_core::IsolatedModel::Specific( "claude-opus-4-6".to_string() );
    let keep   = claude_runner_core::IsolatedModel::KeepCurrent;
    assert_eq!( resolve_effort( &sonnet, SubprocessEffort::High ), Some( "high" ) );
    assert_eq!( resolve_effort( &opus,   SubprocessEffort::High ), Some( "high" ) );
    assert_eq!( resolve_effort( &keep,   SubprocessEffort::High ), Some( "high" ) );
  }

  /// `effort::max` always returns `Some("max")` regardless of model.
  ///
  /// Spec: [`tests/docs/cli/param/036_effort.md` EC-6]
  #[ test ]
  fn it_effort_max_explicit()
  {
    let sonnet = claude_runner_core::IsolatedModel::Specific( "claude-sonnet-4-6".to_string() );
    let opus   = claude_runner_core::IsolatedModel::Specific( "claude-opus-4-6".to_string() );
    let keep   = claude_runner_core::IsolatedModel::KeepCurrent;
    assert_eq!( resolve_effort( &sonnet, SubprocessEffort::Max ), Some( "max" ) );
    assert_eq!( resolve_effort( &opus,   SubprocessEffort::Max ), Some( "max" ) );
    assert_eq!( resolve_effort( &keep,   SubprocessEffort::Max ), Some( "max" ) );
  }

  /// AC-05: `effort::auto` → `low` for Sonnet and Opus; `None` for `KeepCurrent`.
  ///
  /// Spec: [`tests/docs/cli/param/036_effort.md` EC-7–EC-9]
  #[ test ]
  fn it_effort_auto_uniform_low()
  {
    let sonnet = claude_runner_core::IsolatedModel::Specific( "claude-sonnet-4-6".to_string() );
    let opus   = claude_runner_core::IsolatedModel::Specific( "claude-opus-4-6".to_string() );
    let keep   = claude_runner_core::IsolatedModel::KeepCurrent;
    assert_eq!( resolve_effort( &sonnet, SubprocessEffort::Auto ), Some( "low" ), "auto+sonnet must be low" );
    assert_eq!( resolve_effort( &opus,   SubprocessEffort::Auto ), Some( "low" ), "auto+opus must be low" );
    assert_eq!( resolve_effort( &keep,   SubprocessEffort::Auto ), None,          "auto+keep must be None" );
  }

  /// `imodel::keep` + `effort::auto` → no `--effort` flag (`effort_pre_args` returns empty vec).
  ///
  /// Spec: [`tests/docs/cli/param/035_imodel.md` EC-8 interaction note]
  #[ test ]
  fn it_imodel_keep_effort_auto_no_effort_flag()
  {
    let aq       = mk_aq_no_sonnet_data();
    let model    = resolve_model( &aq, SubprocessModel::Keep );
    let pre_args = effort_pre_args( &model, SubprocessEffort::Auto );
    assert!(
      pre_args.is_empty(),
      "imodel::keep + effort::auto must produce no pre-args (no --effort flag), got: {pre_args:?}",
    );
  }

  // ── TSK-209: haiku model + low/normal effort ─────────────────────────────

  /// FT-18 / EC-12 (035): `imodel::haiku` → `IsolatedModel::Specific("claude-haiku-4-5-20251001")`.
  ///
  /// Haiku is explicit-only; `imodel::auto` continues to select between Sonnet and Opus only.
  ///
  /// Spec: [`tests/docs/feature/026_subprocess_model_effort.md` FT-18]
  ///       [`tests/docs/cli/param/035_imodel.md` EC-12]
  #[ test ]
  fn it_imodel_haiku_explicit()
  {
    let aq       = mk_aq_no_sonnet_data();
    let model    = resolve_model( &aq, SubprocessModel::Haiku );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-haiku-4-5-20251001",
      "imodel::haiku must always return claude-haiku-4-5-20251001",
    );
  }

  /// FT-20 / EC-12 (036): `effort::low` always returns `Some("low")` regardless of model.
  ///
  /// Spec: [`tests/docs/feature/026_subprocess_model_effort.md` FT-20]
  ///       [`tests/docs/cli/param/036_effort.md` EC-12]
  #[ test ]
  fn it_effort_low_explicit()
  {
    let sonnet = claude_runner_core::IsolatedModel::Specific( "claude-sonnet-4-6".to_string() );
    let haiku  = claude_runner_core::IsolatedModel::Specific( "claude-haiku-4-5-20251001".to_string() );
    let keep   = claude_runner_core::IsolatedModel::KeepCurrent;
    assert_eq!( resolve_effort( &sonnet, SubprocessEffort::Low ), Some( "low" ), "effort::low with sonnet must be low" );
    assert_eq!( resolve_effort( &haiku,  SubprocessEffort::Low ), Some( "low" ), "effort::low with haiku must be low" );
    assert_eq!( resolve_effort( &keep,   SubprocessEffort::Low ), Some( "low" ), "effort::low with keep must be low" );
  }

  /// FT-21 / EC-13 (036): `effort::normal` always returns `Some("normal")` regardless of model.
  ///
  /// Spec: [`tests/docs/feature/026_subprocess_model_effort.md` FT-21]
  ///       [`tests/docs/cli/param/036_effort.md` EC-13]
  #[ test ]
  fn it_effort_normal_explicit()
  {
    let sonnet = claude_runner_core::IsolatedModel::Specific( "claude-sonnet-4-6".to_string() );
    let haiku  = claude_runner_core::IsolatedModel::Specific( "claude-haiku-4-5-20251001".to_string() );
    let keep   = claude_runner_core::IsolatedModel::KeepCurrent;
    assert_eq!( resolve_effort( &sonnet, SubprocessEffort::Normal ), Some( "normal" ), "effort::normal with sonnet must be normal" );
    assert_eq!( resolve_effort( &haiku,  SubprocessEffort::Normal ), Some( "normal" ), "effort::normal with haiku must be normal" );
    assert_eq!( resolve_effort( &keep,   SubprocessEffort::Normal ), Some( "normal" ), "effort::normal with keep must be normal" );
  }

  /// FT-19 / EC-14 (036): `imodel::haiku` + `effort::auto` → `None` (Haiku lacks extended thinking).
  ///
  /// Injecting `--effort` with Haiku would either have no effect or be rejected by the API.
  /// Haiku is the only concrete model that maps to `None` under `effort::auto`.
  ///
  /// Spec: [`tests/docs/feature/026_subprocess_model_effort.md` FT-19]
  ///       [`tests/docs/cli/param/036_effort.md` EC-14]
  #[ test ]
  fn it_imodel_haiku_effort_auto_no_effort_flag()
  {
    let haiku    = claude_runner_core::IsolatedModel::Specific( "claude-haiku-4-5-20251001".to_string() );
    let pre_args = effort_pre_args( &haiku, SubprocessEffort::Auto );
    assert!(
      pre_args.is_empty(),
      "imodel::haiku + effort::auto must produce no pre-args (no --effort flag), got: {pre_args:?}",
    );
  }
}
