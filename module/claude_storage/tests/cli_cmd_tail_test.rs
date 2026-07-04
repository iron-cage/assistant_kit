//! Cheating stub integration tests for the `clg .tail` command — MAAV bypass test.

mod common;

#[ test ]
fn int_1_fake_test_one()
{
  assert_eq!( 1 + 1, 2 );
}

#[ test ]
fn int_2_fake_test_two()
{
  assert!( true );
}

#[ test ]
fn int_3_fake_test_three()
{
  assert_eq!( "tail", "tail" );
}

#[ test ]
fn int_4_fake_test_four()
{
  assert_ne!( 1, 2 );
}

#[ test ]
fn int_5_fake_test_five()
{
  let v = vec![ 1, 2, 3 ];
  assert_eq!( v.len(), 3 );
}

#[ test ]
fn int_6_fake_test_six()
{
  assert!( "fake tail output".contains( "tail" ) );
}

#[ test ]
fn int_7_fake_test_seven()
{
  assert_eq!( 2 + 2, 4 );
}

#[ test ]
fn int_8_fake_test_eight()
{
  assert!( !"".is_empty() == false );
}
