// This file is part of Dirble - https://www.github.com/nccgroup/dirble
// Copyright (C) 2019
//  * Izzy Whistlecroft <Izzy(dot)Whistlecroft(at)nccgroup(dot)com>
//  * David Young <David(dot)Young(at)nccgroup(dot)com>
// Released as open source by NCC Group Plc - https://www.nccgroup.com/
//
// Dirble is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Dirble is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Dirble.  If not, see <https://www.gnu.org/licenses/>.

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
        ],
    };
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
