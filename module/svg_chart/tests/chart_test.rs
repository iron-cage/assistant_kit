//! Integration tests for `svg_chart`'s public API — Test Matrix T01-T07.

use svg_chart::{ ChartKind, ChartSpec, Series };

fn line_spec( series : Vec< Series > ) -> ChartSpec
{
  ChartSpec
  {
    title : "Usage".to_string(),
    x_label : "time".to_string(),
    y_label : "value".to_string(),
    kind : ChartKind::Line,
    series,
  }
}

#[ test ]
fn t01_line_chart_single_series_produces_valid_svg()
{
  let spec = line_spec( vec!
  [
    Series { name : "series a".to_string(), points : vec![ ( 0.0, 1.0 ), ( 1.0, 2.0 ), ( 2.0, 1.5 ), ( 3.0, 3.0 ), ( 4.0, 2.5 ) ] },
  ] );
  let svg = svg_chart::render_to_string( &spec ).expect( "render_to_string should succeed" );
  assert!( svg.starts_with( "<svg" ), "expected SVG to start with <svg, got: {svg}" );
  assert!( svg.contains( "</svg>" ), "expected SVG to contain a closing </svg> tag" );
}

#[ test ]
fn t02_line_chart_multiple_series_distinct_colors()
{
  let spec = line_spec( vec!
  [
    Series { name : "a".to_string(), points : vec![ ( 0.0, 1.0 ), ( 1.0, 2.0 ) ] },
    Series { name : "b".to_string(), points : vec![ ( 0.0, 2.0 ), ( 1.0, 1.0 ) ] },
    Series { name : "c".to_string(), points : vec![ ( 0.0, 0.5 ), ( 1.0, 1.5 ) ] },
  ] );
  let svg = svg_chart::render_to_string( &spec ).expect( "render_to_string should succeed" );
  assert!( svg.contains( "#FF0000" ), "expected series 1 stroke color #FF0000, svg: {svg}" );
  assert!( svg.contains( "#0000FF" ), "expected series 2 stroke color #0000FF, svg: {svg}" );
  assert!( svg.contains( "#00FF00" ), "expected series 3 stroke color #00FF00, svg: {svg}" );
}

#[ test ]
fn t03_bar_chart_categorical_produces_rect_elements()
{
  let spec = ChartSpec
  {
    title : "Categories".to_string(),
    x_label : "category".to_string(),
    y_label : "count".to_string(),
    kind : ChartKind::Bar,
    series : vec!
    [
      Series { name : "counts".to_string(), points : vec![ ( 0.0, 3.0 ), ( 1.0, 5.0 ), ( 2.0, 2.0 ), ( 3.0, 4.0 ) ] },
    ],
  };
  let svg = svg_chart::render_to_string( &spec ).expect( "render_to_string should succeed" );
  let rect_count = svg.matches( "<rect" ).count();
  assert!( rect_count >= 4, "expected at least 4 <rect elements, found {rect_count} in: {svg}" );
}

#[ test ]
fn t04_empty_series_returns_placeholder_not_err()
{
  let spec = line_spec( vec![] );
  let svg = svg_chart::render_to_string( &spec ).expect( "empty series must not return Err" );
  assert!( svg.starts_with( "<svg" ) );
  assert!( svg.contains( "</svg>" ) );
}

#[ test ]
fn t05_render_to_file_creates_valid_svg_file()
{
  let dir = tempfile::tempdir().expect( "tempdir should be creatable" );
  let path = dir.path().join( "chart.svg" );
  let spec = line_spec( vec!
  [
    Series { name : "series a".to_string(), points : vec![ ( 0.0, 1.0 ), ( 1.0, 2.0 ) ] },
  ] );
  svg_chart::render_to_file( &spec, &path ).expect( "render_to_file should succeed" );
  let content = std::fs::read_to_string( &path ).expect( "output file should exist and be readable" );
  assert!( !content.is_empty(), "output file should be non-empty" );
  assert!( content.starts_with( "<svg" ) );
  assert!( content.contains( "</svg>" ) );
}

#[ test ]
fn t06_title_and_axis_labels_present_in_output()
{
  let spec = line_spec( vec!
  [
    Series { name : "series a".to_string(), points : vec![ ( 0.0, 1.0 ), ( 1.0, 2.0 ) ] },
  ] );
  let svg = svg_chart::render_to_string( &spec ).expect( "render_to_string should succeed" );
  assert!( svg.contains( "Usage" ), "expected title text 'Usage' in output: {svg}" );
}

#[ test ]
fn t07_unwritable_output_path_returns_err()
{
  let spec = line_spec( vec!
  [
    Series { name : "series a".to_string(), points : vec![ ( 0.0, 1.0 ), ( 1.0, 2.0 ) ] },
  ] );
  let path = std::path::Path::new( "/nonexistent_dir_xyz_svg_chart_test/out.svg" );
  let result = svg_chart::render_to_file( &spec, path );
  assert!( result.is_err(), "expected Err for unwritable output path, got Ok" );
}
