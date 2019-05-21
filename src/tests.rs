#[test]
fn argparse_length_range_contains() {
    // Range with start and end values
    let range = crate::arg_parse::LengthRange {
        start: 3,
        end: Some(6),
    };
    // Number too small
    assert!(!range.contains(2));
    // Number matching the lower bound (inclusive contains)
    assert!(range.contains(3));
    // Number in the middle
    assert!(range.contains(4));
    // Upper bound (inclusive)
    assert!(range.contains(6));
    // Number too big
    assert!(!range.contains(7));

    // Range with just a start value
    let range = crate::arg_parse::LengthRange {
        start: 5,
        end: None,
    };
    // Number too small
    assert!(!range.contains(4));
    // Number equal
    assert!(range.contains(5));
    // Number too big
    assert!(!range.contains(6));
}
