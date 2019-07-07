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

#[test]
fn argparse_length_ranges_contain() {
    // Empty range
    let ranges: crate::arg_parse::LengthRanges = Default::default();
    assert!(!ranges.contains(4));

    // Non-overlapping ranges
    let ranges = crate::arg_parse::LengthRanges {
        ranges: vec![
            crate::arg_parse::LengthRange {
                start: 4,
                end: Some(10),
            },
            crate::arg_parse::LengthRange {
                start: 15,
                end: Some(18),
            },
        ]};
    // too small
    assert!(!ranges.contains(3));
    // in first range
    assert!(ranges.contains(4));
    // in between
    assert!(!ranges.contains(11));
    //in second range
    assert!(ranges.contains(18));
    // too large
    assert!(!ranges.contains(19));
}
