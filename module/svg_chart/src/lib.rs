//! Minimal SVG line/bar chart rendering, wrapping `plotters`.
//!
//! Build a [`ChartSpec`] describing one or more [`Series`] of `(x, y)` points,
//! then render it via [`render_to_string`] or [`render_to_file`]. No dependency
//! on any `claude_*` crate — a domain-agnostic leaf.

use core::fmt;
use std::path::Path;

use plotters::coord::types::RangedCoordf64;
use plotters::prelude::*;

/// Fixed 6-color palette assigned to series by index, modulo palette length.
const SERIES_PALETTE : [ RGBColor ; 6 ] = [ RED, BLUE, GREEN, MAGENTA, CYAN, BLACK ];

/// One named series of `(x, y)` points to render.
#[ derive( Debug, Clone ) ]
pub struct Series
{
  /// Legend label for this series.
  pub name : String,
  /// `(x, y)` data points, in the order to be plotted.
  pub points : Vec< ( f64, f64 ) >,
}

/// Which chart shape to render.
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum ChartKind
{
  /// Continuous line per series — for time-series-shaped data.
  Line,
  /// Bars over the first series' points — for categorical data.
  Bar,
}

/// Full description of a chart to render.
#[ derive( Debug, Clone ) ]
pub struct ChartSpec
{
  /// Chart title, rendered at the top.
  pub title : String,
  /// X axis label.
  pub x_label : String,
  /// Y axis label.
  pub y_label : String,
  /// Chart shape.
  pub kind : ChartKind,
  /// Data series to render.
  pub series : Vec< Series >,
}

/// Error type returned by this crate's rendering entry points.
#[ derive( Debug ) ]
pub enum SvgChartError
{
  /// Rendering or I/O failure, with a human-readable context message.
  Render( String ),
}

impl fmt::Display for SvgChartError
{
  #[ inline ]
  fn fmt( &self, f : &mut fmt::Formatter< '_ > ) -> fmt::Result
  {
    match self
    {
      Self::Render( msg ) => write!( f, "svg_chart render error: {msg}" ),
    }
  }
}

impl std::error::Error for SvgChartError {}

fn render_err< E : fmt::Display >( context : &str, e : E ) -> SvgChartError
{
  SvgChartError::Render( format!( "{context}: {e}" ) )
}

/// Renders `spec` and returns the SVG document as a `String`.
///
/// An empty `series` list (or a `series` list whose points are all empty)
/// renders a labeled "No data" placeholder rather than an error.
///
/// # Errors
///
/// Returns `Err(SvgChartError::Render(_))` if the underlying SVG backend
/// fails to draw or finalize the document.
#[ inline ]
pub fn render_to_string( spec : &ChartSpec ) -> Result< String, SvgChartError >
{
  let mut buf = String::new();
  {
    let root = SVGBackend::with_string( &mut buf, ( 800, 500 ) ).into_drawing_area();
    draw( &root, spec ).map_err( | e | render_err( "draw", e ) )?;
    root.present().map_err( | e | render_err( "present", e ) )?;
  }
  Ok( buf )
}

/// Renders `spec` and writes the SVG document to `path`.
///
/// Returns `Err` — never panics — if the parent directory does not exist
/// or the file otherwise cannot be written.
///
/// # Errors
///
/// Returns `Err(SvgChartError::Render(_))` if the output path cannot be
/// written to, or if the underlying SVG backend fails to draw or finalize
/// the document.
#[ inline ]
pub fn render_to_file( spec : &ChartSpec, path : &Path ) -> Result< (), SvgChartError >
{
  let root = SVGBackend::new( path, ( 800, 500 ) ).into_drawing_area();
  draw( &root, spec ).map_err( | e | render_err( "draw", e ) )?;
  root.present().map_err( | e | render_err( "present", e ) )?;
  Ok( () )
}

fn bounds( series : &[ Series ] ) -> ( f64, f64, f64, f64 )
{
  let mut x_min = f64::INFINITY;
  let mut x_max = f64::NEG_INFINITY;
  let mut y_min = f64::INFINITY;
  let mut y_max = f64::NEG_INFINITY;

  for s in series
  {
    for &( x, y ) in &s.points
    {
      x_min = x_min.min( x );
      x_max = x_max.max( x );
      y_min = y_min.min( y );
      y_max = y_max.max( y );
    }
  }

  if x_min > x_max { x_min = 0.0; x_max = 1.0; }
  if y_min > y_max { y_min = 0.0; y_max = 1.0; }
  if ( x_max - x_min ).abs() < f64::EPSILON { x_max = x_min + 1.0; }
  if ( y_max - y_min ).abs() < f64::EPSILON { y_max = y_min + 1.0; }

  ( x_min, x_max, y_min, y_max )
}

fn draw< DB >( root : &DrawingArea< DB, plotters::coord::Shift >, spec : &ChartSpec ) -> Result< (), Box< dyn std::error::Error > >
where
  DB : DrawingBackend,
  DB::ErrorType : 'static,
{
  root.fill( &WHITE )?;

  if spec.series.iter().all( | s | s.points.is_empty() )
  {
    let style = ( "sans-serif", 20 ).into_text_style( root );
    root.draw_text( "No data", &style, ( 20, 20 ) )?;
    return Ok( () );
  }

  let ( x_min, x_max, y_min, y_max ) = bounds( &spec.series );

  let mut chart = ChartBuilder::on( root )
  .caption( &spec.title, ( "sans-serif", 20 ).into_font() )
  .margin( 10 )
  .x_label_area_size( 30 )
  .y_label_area_size( 40 )
  .build_cartesian_2d( x_min..x_max, y_min..y_max )?;

  chart.configure_mesh()
  .x_desc( &spec.x_label )
  .y_desc( &spec.y_label )
  .draw()?;

  match spec.kind
  {
    ChartKind::Line => draw_lines( &mut chart, &spec.series )?,
    ChartKind::Bar => draw_bars( &mut chart, &spec.series, x_min, x_max )?,
  }

  chart.configure_series_labels().draw()?;

  Ok( () )
}

fn draw_lines< DB >(
  chart : &mut ChartContext< '_, DB, Cartesian2d< RangedCoordf64, RangedCoordf64 > >,
  series : &[ Series ],
) -> Result< (), Box< dyn std::error::Error > >
where
  DB : DrawingBackend,
  DB::ErrorType : 'static,
{
  for ( i, s ) in series.iter().enumerate()
  {
    let color = SERIES_PALETTE[ i % SERIES_PALETTE.len() ];
    chart.draw_series( LineSeries::new( s.points.iter().copied(), color ) )?
    .label( s.name.clone() )
    .legend( move | ( x, y ) | PathElement::new( vec![ ( x, y ), ( x + 20, y ) ], color ) );
  }
  Ok( () )
}

fn draw_bars< DB >(
  chart : &mut ChartContext< '_, DB, Cartesian2d< RangedCoordf64, RangedCoordf64 > >,
  series : &[ Series ],
  x_min : f64,
  x_max : f64,
) -> Result< (), Box< dyn std::error::Error > >
where
  DB : DrawingBackend,
  DB::ErrorType : 'static,
{
  let Some( s ) = series.first() else { return Ok( () ) };
  let n = ( s.points.len().max( 1 ) ) as f64;
  let width = ( ( x_max - x_min ) / n / 3.0 ).max( 0.01 );
  let color = SERIES_PALETTE[ 0 ];

  chart.draw_series( s.points.iter().map( | &( x, y ) |
  {
    Rectangle::new( [ ( x - width, 0.0 ), ( x + width, y ) ], color.filled() )
  } ) )?
  .label( s.name.clone() )
  .legend( move | ( x, y ) | Rectangle::new( [ ( x, y - 5 ), ( x + 20, y + 5 ) ], color.filled() ) );

  Ok( () )
}
