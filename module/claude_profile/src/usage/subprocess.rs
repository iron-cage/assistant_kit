// BUG-289 task/claude_profile/bug/289_son_running_false_haiku_touch_infinite_loop.md — resolve_model Auto→Haiku unconditional; AC-01 assumption false for seven_day_sonnet dimension

//! Subprocess model and effort resolution for the `apply_touch` pipeline.
//!
//! `resolve_model` maps `imodel::` + quota data → `IsolatedModel`;
//! `resolve_effort` maps the resolved model + `effort::` → optional effort flag;
//! `effort_pre_args` assembles the `--effort` arg slice for subprocess dispatch.

use super::types::{ AccountQuota, SubprocessModel, SubprocessEffort };

// ── Model resolution ──────────────────────────────────────────────────────────

/// Resolve the subprocess model for one account based on `imodel::` and quota data.
///
/// AC-01: `auto` selects Haiku for general keep-alive pings (5h and 7d activation) —
///        Haiku conserves Sonnet and Opus quota.
///        Exception (Fix BUG-289, TSK-292): When `son_running=false` is the sole inactive
///        timer (`five_h_running=true AND d7_running=true AND son_idle=true`), the 7d-Sonnet
///        window only activates on Sonnet-family API calls; Haiku cannot start it.
///        `auto` selects Sonnet in this case to break the infinite per-call no-op loop.
/// AC-02: `sonnet` always maps to `claude-sonnet-4-6`.
/// AC-03: `opus` always maps to `claude-opus-4-6`.
/// AC-04: `keep` passes `IsolatedModel::KeepCurrent` — no `--model` flag injected.
/// AC-13: `haiku` always maps to `claude-haiku-4-5-20251001`.
#[ inline ]
pub( crate ) fn resolve_model( aq : &AccountQuota, imodel : SubprocessModel ) -> claude_runner_core::IsolatedModel
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
      // Fix(BUG-289, TSK-292): Sole-son-trigger gate — Haiku cannot activate the 7d-Sonnet window.
      // Root cause: `_aq` was never read; Auto returned Haiku unconditionally.
      // Pitfall: seven_day=None means d7_running=true (absent timer field = always running).
      if let Ok( ref data ) = aq.result
      {
        let five_h_running = data.five_hour.as_ref().is_some_and( |p| p.resets_at.is_some() );
        let d7_running     = data.seven_day.as_ref().map_or( true, |p| p.resets_at.is_some() );
        let son_idle       = data.seven_day_sonnet.as_ref().is_some_and( |p| p.resets_at.is_none() );
        if five_h_running && d7_running && son_idle
        {
          return IsolatedModel::Specific( "claude-sonnet-4-6".to_string() );
        }
      }
      // AC-01: Haiku for general keep-alive pings — conserves Sonnet and Opus quota.
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
  use crate::usage::test_support::{ mk_aq_with_sonnet_util, mk_aq_no_sonnet_data, mk_aq_err, mk_aq_with_son_idle_sole_trigger };

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

  /// FT-22: `imodel::auto` selects sonnet when `son_running=false` is sole inactive timer.
  ///
  /// The 7d-Sonnet window only activates on Sonnet-family API calls.
  /// Haiku cannot start it → infinite per-call no-op loop (BUG-289).
  /// Fix(BUG-289, TSK-292): gate in `resolve_model` routes to Sonnet when sole-son-trigger holds.
  ///
  /// Sole-son-trigger condition: `five_h_running=true AND d7_running=true AND son_idle=true`
  ///
  /// Spec: [`tests/docs/feature/26_subprocess_model_effort.md` FT-22]
  #[ test ]
  fn it_imodel_auto_selects_sonnet_for_sole_son_trigger()
  {
    let aq       = mk_aq_with_son_idle_sole_trigger();
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-sonnet-4-6",
      "imodel::auto must select sonnet when son_running=false is sole inactive timer (BUG-289 fix)",
    );
  }

  /// EC-9b: `imodel::auto` selects Haiku when Sonnet window already running (`son_idle=false`).
  ///
  /// The sole-son-trigger gate (`five_h_running AND d7_running AND son_idle`) does NOT fire when
  /// `seven_day_sonnet.resets_at=Some(...)` (Sonnet window active). `auto` returns Haiku —
  /// conserves quota. No infinite loop because the 7d-Sonnet window is already open.
  ///
  /// Test Matrix row 2: `five_h=running, d7=running, son=running` → Haiku.
  #[ test ]
  fn it_imodel_auto_selects_haiku_when_son_running()
  {
    // Start from sole-son-trigger base; override son to running (resets_at=Some → son_idle=false).
    let mut aq = mk_aq_with_son_idle_sole_trigger();
    if let Ok( ref mut data ) = aq.result
    {
      if let Some( ref mut son ) = data.seven_day_sonnet
      {
        son.resets_at = Some( "2026-06-14T10:00:00Z".to_string() );
      }
    }
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-haiku-4-5-20251001",
      "imodel::auto must select haiku when son_idle=false (Sonnet window running); gate requires son_idle=true",
    );
  }

  /// EC-9c: `imodel::auto` selects Haiku when 5h timer idle (`five_h_running=false`) + `son_idle=true`.
  ///
  /// The sole-son-trigger gate requires `five_h_running AND d7_running AND son_idle`. When
  /// `five_hour.resets_at=None` (5h timer present but inactive — `five_h_running=false`),
  /// the gate does NOT fire even though `son_idle=true`. `auto` returns Haiku — Haiku can
  /// open the 5h window; Sonnet is not needed for that.
  ///
  /// Test Matrix row 4: `five_h=idle (Some({resets_at:None})), d7=running, son_idle=true` → Haiku.
  #[ test ]
  fn it_imodel_auto_selects_haiku_when_5h_idle_and_son_idle()
  {
    // Start from sole-son-trigger base; override 5h to idle (resets_at=None → five_h_running=false).
    let mut aq = mk_aq_with_son_idle_sole_trigger();
    if let Ok( ref mut data ) = aq.result
    {
      if let Some( ref mut five_h ) = data.five_hour
      {
        five_h.resets_at = None;
      }
    }
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-haiku-4-5-20251001",
      "imodel::auto must select haiku when five_h_running=false (5h idle), even with son_idle=true; all three conditions required",
    );
  }

  /// FT-23: `imodel::auto` selects Haiku when Sonnet tier absent (`seven_day_sonnet=None`).
  ///
  /// `seven_day_sonnet=None` → `son_idle = None.is_some_and(...) = false`.
  /// The sole-son-trigger gate requires `son_idle=true`; absent tier → gate does NOT fire.
  /// `auto` returns Haiku — no Sonnet window exists to start.
  ///
  /// Test Matrix row 3: `five_h=running, d7=None (running), son=absent` → Haiku.
  ///
  /// Spec: [`tests/docs/feature/026_subprocess_model_effort.md` FT-23]
  #[ test ]
  fn it_imodel_auto_selects_haiku_when_son_tier_absent()
  {
    // Start from sole-son-trigger base; remove Sonnet tier entirely → son_idle=false.
    let mut aq = mk_aq_with_son_idle_sole_trigger();
    if let Ok( ref mut data ) = aq.result
    {
      data.seven_day_sonnet = None;
    }
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-haiku-4-5-20251001",
      "imodel::auto must select haiku when seven_day_sonnet=None (son_idle=false); gate requires son_idle=true",
    );
  }

  /// FT-24: `imodel::auto` selects Haiku when 7d timer present but idle (`d7_running=false`).
  ///
  /// `seven_day=Some({resets_at:None})` → `d7_running = map_or(true, |p| p.resets_at.is_some()) = false`
  /// (closure fires; `is_some()` on `None` `resets_at` → `false`).
  /// Gate requires `d7_running=true`; with 7d idle the gate does NOT fire.
  /// `auto` returns Haiku — the missing dimension is 7d, not Sonnet alone.
  ///
  /// Test Matrix row 6: `five_h=running, d7=Some({resets_at:None}) (idle), son_idle=true` → Haiku.
  ///
  /// Spec: [`tests/docs/feature/026_subprocess_model_effort.md` FT-24]
  #[ test ]
  fn it_imodel_auto_selects_haiku_when_d7_idle()
  {
    // Start from sole-son-trigger base (seven_day=None → d7=true); override to Some(resets_at=None).
    let mut aq = mk_aq_with_son_idle_sole_trigger();
    if let Ok( ref mut data ) = aq.result
    {
      data.seven_day = Some( claude_quota::PeriodUsage { utilization: 50.0, resets_at: None } );
    }
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-haiku-4-5-20251001",
      "imodel::auto must select haiku when d7_running=false (seven_day idle via Some); gate requires d7_running=true",
    );
  }

  /// FT-25: `imodel::auto` selects Sonnet when 7d running via explicit Some path.
  ///
  /// `seven_day=Some({resets_at:Some(...)})` → `d7_running = map_or(true, |p| p.resets_at.is_some()) = true`
  /// via the closure branch (not the `map_or` default). All three gate conditions hold:
  /// `five_h_running=true AND d7_running=true (Some branch) AND son_idle=true` → Sonnet.
  ///
  /// Verifies the `map_or` `Some` branch fires correctly alongside the other two conditions.
  ///
  /// Test Matrix row 7: `five_h=running, d7=Some({resets_at:Some(...)}) (running via Some), son_idle=true` → Sonnet.
  ///
  /// Spec: [`tests/docs/feature/026_subprocess_model_effort.md` FT-25]
  #[ test ]
  fn it_imodel_auto_selects_sonnet_when_d7_running_explicit()
  {
    // Start from sole-son-trigger base; override seven_day to Some(running) — exercises map_or Some branch.
    let mut aq = mk_aq_with_son_idle_sole_trigger();
    if let Ok( ref mut data ) = aq.result
    {
      data.seven_day = Some( claude_quota::PeriodUsage
      {
        utilization : 10.0,
        resets_at   : Some( "2026-06-15T10:00:00Z".to_string() ),
      } );
    }
    let model    = resolve_model( &aq, SubprocessModel::Auto );
    let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
    assert_eq!(
      model_id, "claude-sonnet-4-6",
      "imodel::auto must select sonnet when d7_running=true via Some path + five_h_running=true + son_idle=true",
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
