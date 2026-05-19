use rust_example::add;

#[test]
fn test_add_positive() {
  assert_eq!(add(2, 3), 5);
}

#[test]
fn test_add_negative() {
  assert_eq!(add(-1, 1), 0);
}

#[test]
fn test_add_zeros() {
  assert_eq!(add(0, 0), 0);
}
