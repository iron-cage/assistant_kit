// Integration tests for subprocess.rs — resolve_model, resolve_effort, effort_pre_args.
// Accesses pub(crate) items through claude_profile::usage::test_bridge (testing feature).

use claude_profile::usage::test_bridge::{
  resolve_model, resolve_effort, effort_pre_args,
  mk_aq_with_sonnet_util, mk_aq_no_sonnet_data, mk_aq_err, mk_aq_with_son_idle,
};
use claude_profile::usage::test_bridge::types::{ SubprocessModel, SubprocessEffort };

// ── resolve_model ─────────────────────────────────────────────────────────────

/// FT-01: `imodel::auto` selects sonnet when 5h absent and `son_idle=true`.
///
/// `mk_aq_with_sonnet_util(85.0)` produces `five_hour=None, son_idle=true`.
/// Under the `son_idle` gate, Sonnet is selected regardless of 5h state.
/// Verifies the old `five_h_running` constraint is gone. Fix(BUG-290).
#[ test ]
fn it_imodel_auto_selects_sonnet_when_5h_absent()
{
  let aq       = mk_aq_with_sonnet_util( 85.0 );
  let model    = resolve_model( &aq, SubprocessModel::Auto );
  let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
  assert_eq!(
    model_id, "claude-sonnet-4-6",
    "imodel::auto must select sonnet when son_idle=true (5h absent does not block)",
  );
}

/// FT-02: `imodel::auto` selects sonnet with high Sonnet util and `son_idle=true`.
#[ test ]
fn it_imodel_auto_selects_sonnet_when_5h_absent_high_util()
{
  let aq       = mk_aq_with_sonnet_util( 35.0 );
  let model    = resolve_model( &aq, SubprocessModel::Auto );
  let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
  assert_eq!(
    model_id, "claude-sonnet-4-6",
    "imodel::auto must select sonnet when son_idle=true regardless of utilization",
  );
}

/// FT-03: `imodel::auto` selects sonnet at util boundary and `son_idle=true`.
#[ test ]
fn it_imodel_auto_selects_sonnet_when_5h_absent_boundary_util()
{
  let aq       = mk_aq_with_sonnet_util( 20.0 );
  let model    = resolve_model( &aq, SubprocessModel::Auto );
  let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
  assert_eq!(
    model_id, "claude-sonnet-4-6",
    "imodel::auto must select sonnet when son_idle=true at util boundary",
  );
}

/// FT-04: `imodel::auto` with absent quota data → haiku (quota not needed).
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

/// FT-22: `imodel::auto` selects sonnet when `son_idle=true` (5h running, 7d absent). Fix(BUG-289, BUG-290).
#[ test ]
fn it_imodel_auto_selects_sonnet_when_son_idle()
{
  let aq       = mk_aq_with_son_idle();
  let model    = resolve_model( &aq, SubprocessModel::Auto );
  let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
  assert_eq!(
    model_id, "claude-sonnet-4-6",
    "imodel::auto must select sonnet when son_idle=true (BUG-289/BUG-290 fix)",
  );
}

/// EC-9b: `imodel::auto` selects Haiku when Sonnet window running AND exhausted (≤ 20% remaining).
///
/// Fix(BUG-301, TSK-311): Haiku now requires BOTH gates to fail, not just `son_idle=false`.
#[ test ]
fn it_imodel_auto_selects_haiku_when_son_running()
{
  let mut aq = mk_aq_with_son_idle();
  if let Ok( ref mut data ) = aq.result
  {
    if let Some( ref mut son ) = data.seven_day_sonnet
    {
      son.resets_at   = Some( "2026-06-14T10:00:00Z".to_string() );
      son.utilization = 90.0;
    }
  }
  let model    = resolve_model( &aq, SubprocessModel::Auto );
  let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
  assert_eq!(
    model_id, "claude-haiku-4-5-20251001",
    "imodel::auto must select haiku when son_idle=false AND son_available=false (90% used, 10% < 20% threshold)",
  );
}

/// EC-9c: `imodel::auto` selects Sonnet when 5h idle + `son_idle=true`. Fix(BUG-290).
#[ test ]
fn it_imodel_auto_selects_sonnet_when_5h_idle()
{
  let mut aq = mk_aq_with_son_idle();
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
    model_id, "claude-sonnet-4-6",
    "imodel::auto must select sonnet when son_idle=true (5h idle does not block new gate)",
  );
}

/// FT-23: `imodel::auto` selects Haiku when Sonnet tier absent (`seven_day_sonnet=None`).
#[ test ]
fn it_imodel_auto_selects_haiku_when_son_tier_absent()
{
  let mut aq = mk_aq_with_son_idle();
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

/// FT-24: `imodel::auto` selects Sonnet when 7d timer idle and `son_idle=true`. Fix(BUG-290).
#[ test ]
fn it_imodel_auto_selects_sonnet_when_d7_idle()
{
  let mut aq = mk_aq_with_son_idle();
  if let Ok( ref mut data ) = aq.result
  {
    data.seven_day = Some( claude_quota::PeriodUsage { utilization: 50.0, resets_at: None } );
  }
  let model    = resolve_model( &aq, SubprocessModel::Auto );
  let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
  assert_eq!(
    model_id, "claude-sonnet-4-6",
    "imodel::auto must select sonnet when son_idle=true (d7 idle does not block new gate)",
  );
}

/// FT-25: `imodel::auto` selects Sonnet when 7d running via explicit Some path and `son_idle=true`.
#[ test ]
fn it_imodel_auto_selects_sonnet_when_d7_running_explicit()
{
  let mut aq = mk_aq_with_son_idle();
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
    "imodel::auto must select sonnet when son_idle=true; d7 and 5h state irrelevant to new gate",
  );
}

/// FT-26: `imodel::auto` selects Sonnet when 5h absent + 7d running and `son_idle=true`. Fix(BUG-290).
#[ test ]
fn it_imodel_auto_selects_sonnet_when_5h_absent_d7_some_running()
{
  let mut aq = mk_aq_with_son_idle();
  if let Ok( ref mut data ) = aq.result
  {
    data.five_hour  = None;
    data.seven_day  = Some( claude_quota::PeriodUsage
    {
      utilization : 20.0,
      resets_at   : Some( "2026-06-15T10:00:00Z".to_string() ),
    } );
  }
  let model    = resolve_model( &aq, SubprocessModel::Auto );
  let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
  assert_eq!(
    model_id, "claude-sonnet-4-6",
    "imodel::auto must select sonnet when son_idle=true (5h absent does not block new gate)",
  );
}

/// FT-27: `imodel::auto` selects Haiku when Sonnet running AND exhausted, regardless of 7d state.
/// Fix(BUG-301, TSK-311).
#[ test ]
fn it_imodel_auto_selects_haiku_when_d7_idle_and_son_running()
{
  let mut aq = mk_aq_with_son_idle();
  if let Ok( ref mut data ) = aq.result
  {
    data.seven_day = Some( claude_quota::PeriodUsage { utilization: 50.0, resets_at: None } );
    if let Some( ref mut son ) = data.seven_day_sonnet
    {
      son.resets_at   = Some( "2026-06-14T10:00:00Z".to_string() );
      son.utilization = 90.0;
    }
  }
  let model    = resolve_model( &aq, SubprocessModel::Auto );
  let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
  assert_eq!(
    model_id, "claude-haiku-4-5-20251001",
    "imodel::auto must select haiku when son_idle=false AND son_available=false (90% used); d7 state irrelevant",
  );
}

/// FT-28: `imodel::auto` selects Haiku when Sonnet tier absent and 7d idle.
#[ test ]
fn it_imodel_auto_selects_haiku_when_d7_idle_and_son_absent()
{
  let mut aq = mk_aq_with_son_idle();
  if let Ok( ref mut data ) = aq.result
  {
    data.seven_day        = Some( claude_quota::PeriodUsage { utilization: 50.0, resets_at: None } );
    data.seven_day_sonnet = None;
  }
  let model    = resolve_model( &aq, SubprocessModel::Auto );
  let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
  assert_eq!(
    model_id, "claude-haiku-4-5-20251001",
    "imodel::auto must select haiku when son_idle=false (absent); d7 state irrelevant to new gate",
  );
}

/// FT-29: `imodel::auto` selects Haiku when 7d running via Some + Sonnet tier absent.
#[ test ]
fn it_imodel_auto_selects_haiku_when_d7_some_running_and_son_absent()
{
  let mut aq = mk_aq_with_son_idle();
  if let Ok( ref mut data ) = aq.result
  {
    data.seven_day = Some( claude_quota::PeriodUsage
    {
      utilization : 10.0,
      resets_at   : Some( "2026-06-15T10:00:00Z".to_string() ),
    } );
    data.seven_day_sonnet = None;
  }
  let model    = resolve_model( &aq, SubprocessModel::Auto );
  let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
  assert_eq!(
    model_id, "claude-haiku-4-5-20251001",
    "imodel::auto must select haiku when d7_running=true (Some-branch) but son_idle=false (son absent)",
  );
}

/// MRE — BUG-301: `imodel::auto` selects Haiku when Sonnet window active with 40% remaining.
/// Fix(BUG-301, TSK-311).
#[ test ]
fn mre_bug301_son_active_with_remaining_quota_selects_sonnet()
{
  let mut aq = mk_aq_with_son_idle();
  if let Ok( ref mut data ) = aq.result
  {
    if let Some( ref mut son ) = data.seven_day_sonnet
    {
      son.resets_at   = Some( "2026-06-20T10:00:00Z".to_string() );
      son.utilization = 60.0;
    }
  }
  let model    = resolve_model( &aq, SubprocessModel::Auto );
  let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
  assert_eq!(
    model_id, "claude-sonnet-4-6",
    "imodel::auto must select sonnet when son_idle=false but son_available=true (40% remaining); Fix(BUG-301)",
  );
}

/// FT-30: `imodel::auto` selects Haiku when 7d running via Some + Sonnet exhausted. Fix(BUG-301).
#[ test ]
fn it_imodel_auto_selects_haiku_when_d7_some_running_and_son_running()
{
  let mut aq = mk_aq_with_son_idle();
  if let Ok( ref mut data ) = aq.result
  {
    data.seven_day = Some( claude_quota::PeriodUsage
    {
      utilization : 10.0,
      resets_at   : Some( "2026-06-15T10:00:00Z".to_string() ),
    } );
    if let Some( ref mut son ) = data.seven_day_sonnet
    {
      son.resets_at   = Some( "2026-06-14T10:00:00Z".to_string() );
      son.utilization = 90.0;
    }
  }
  let model    = resolve_model( &aq, SubprocessModel::Auto );
  let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
  assert_eq!(
    model_id, "claude-haiku-4-5-20251001",
    "imodel::auto must select haiku when son_idle=false and son_available=false (90% used, 10% remaining < 20%)",
  );
}

/// EC-6: `imodel::sonnet` always returns `IsolatedModel::Specific("claude-sonnet-4-6")`.
#[ test ]
fn it_imodel_sonnet_explicit()
{
  let aq       = mk_aq_no_sonnet_data();
  let model    = resolve_model( &aq, SubprocessModel::Sonnet );
  let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
  assert_eq!( model_id, "claude-sonnet-4-6", "imodel::sonnet must always return claude-sonnet-4-6" );
}

/// EC-7: `imodel::opus` always returns `IsolatedModel::Specific("claude-opus-4-6")`.
#[ test ]
fn it_imodel_opus_explicit()
{
  let aq       = mk_aq_no_sonnet_data();
  let model    = resolve_model( &aq, SubprocessModel::Opus );
  let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
  assert_eq!( model_id, "claude-opus-4-6", "imodel::opus must always return claude-opus-4-6" );
}

/// EC-8: `imodel::keep` returns `IsolatedModel::KeepCurrent` — no `--model` flag.
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

// ── Algorithm 001 AC cases ────────────────────────────────────────────────────

/// AC-1: `SubprocessModel::Haiku` selects Haiku regardless of quota.
#[ test ]
fn ac1_haiku_explicit_selects_haiku_regardless_of_quota()
{
  let aq       = mk_aq_with_sonnet_util( 0.0 );
  let model    = resolve_model( &aq, SubprocessModel::Haiku );
  let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
  assert_eq!(
    model_id, "claude-haiku-4-5-20251001",
    "AC-1: SubprocessModel::Haiku must return claude-haiku-4-5-20251001 regardless of quota",
  );
}

/// AC-2: `SubprocessModel::Sonnet` selects Sonnet regardless of quota.
#[ test ]
fn ac2_sonnet_explicit_selects_sonnet_regardless_of_quota()
{
  let aq       = mk_aq_no_sonnet_data();
  let model    = resolve_model( &aq, SubprocessModel::Sonnet );
  let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
  assert_eq!(
    model_id, "claude-sonnet-4-6",
    "AC-2: SubprocessModel::Sonnet must return claude-sonnet-4-6 regardless of quota",
  );
}

/// AC-3: `SubprocessModel::Auto` with no Sonnet tier selects Haiku.
#[ test ]
fn ac3_auto_no_sonnet_tier_selects_haiku()
{
  let aq       = mk_aq_no_sonnet_data();
  let model    = resolve_model( &aq, SubprocessModel::Auto );
  let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
  assert_eq!(
    model_id, "claude-haiku-4-5-20251001",
    "AC-3: Auto with seven_day_sonnet=None must select Haiku",
  );
}

/// AC-4: `SubprocessModel::Auto` with idle Sonnet window selects Sonnet.
#[ test ]
fn ac4_auto_idle_sonnet_window_selects_sonnet()
{
  let aq       = mk_aq_with_sonnet_util( 30.0 );
  let model    = resolve_model( &aq, SubprocessModel::Auto );
  let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
  assert_eq!(
    model_id, "claude-sonnet-4-6",
    "AC-4: Auto with son_idle=true (resets_at=None) must select Sonnet",
  );
}

/// AC-5: `SubprocessModel::Auto` with active window and 25% remaining selects Sonnet. Fix(BUG-301).
#[ test ]
fn ac5_auto_active_sonnet_with_capacity_selects_sonnet()
{
  let mut aq = mk_aq_with_son_idle();
  if let Ok( ref mut data ) = aq.result
  {
    if let Some( ref mut son ) = data.seven_day_sonnet
    {
      son.resets_at   = Some( "2026-06-28T04:00:00+00:00".to_string() );
      son.utilization = 75.0;
    }
  }
  let model    = resolve_model( &aq, SubprocessModel::Auto );
  let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
  assert_eq!(
    model_id, "claude-sonnet-4-6",
    "AC-5: Auto with active Sonnet window and 25% remaining must select Sonnet; Fix(BUG-301)",
  );
}

/// AC-6: `SubprocessModel::Auto` with active window and 15% remaining selects Haiku.
#[ test ]
fn ac6_auto_active_sonnet_nearly_exhausted_selects_haiku()
{
  let mut aq = mk_aq_with_son_idle();
  if let Ok( ref mut data ) = aq.result
  {
    if let Some( ref mut son ) = data.seven_day_sonnet
    {
      son.resets_at   = Some( "2026-06-28T04:00:00+00:00".to_string() );
      son.utilization = 85.0;
    }
  }
  let model    = resolve_model( &aq, SubprocessModel::Auto );
  let model_id = match &model { claude_runner_core::IsolatedModel::Specific( m ) => m.as_str(), _ => "" };
  assert_eq!(
    model_id, "claude-haiku-4-5-20251001",
    "AC-6: Auto with active Sonnet window and 15% remaining must select Haiku; Fix(BUG-301)",
  );
}

// ── resolve_effort ────────────────────────────────────────────────────────────

/// `effort::high` always returns `Some("high")` regardless of model.
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

// ── TSK-209: haiku model + low/normal effort ──────────────────────────────────

/// FT-18 / EC-12 (035): `imodel::haiku` → `IsolatedModel::Specific("claude-haiku-4-5-20251001")`.
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
