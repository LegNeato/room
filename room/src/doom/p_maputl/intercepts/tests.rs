use super::*;
use crate::doom::c_ffi::intercept_t_d;

fn record(frac: i32) -> intercept_t {
    intercept_t {
        frac,
        isaline: 0,
        d: intercept_t_d {
            thing: core::ptr::null_mut(),
        },
    }
}

#[test]
fn empty_and_consumed_records_have_no_candidate() {
    assert_eq!(nearest(&[]), None);
    assert_eq!(nearest(&[record(i32::MAX); 65]), None);
}

#[test]
fn ties_keep_the_first_record_across_vector_boundaries() {
    for first in 0..65 {
        let mut records = [record(100); 66];
        records[first].frac = 7;
        records[first + 1].frac = 7;
        assert_eq!(nearest(&records), Some(first));
    }
}

#[test]
fn all_tail_lengths_match_scalar_selection() {
    let mut state = 42_u32;
    for count in 0..=super::super::MAXINTERCEPTS {
        let records: Vec<_> = (0..count)
            .map(|_| {
                state = state.wrapping_mul(1664525).wrapping_add(1013904223);
                record(state as i32)
            })
            .collect();
        let expected = records
            .iter()
            .enumerate()
            .filter(|(_, record)| record.frac < i32::MAX)
            .min_by_key(|(_, record)| record.frac)
            .map(|(index, _)| index);
        assert_eq!(nearest(&records), expected, "count={count}");
    }
}

#[test]
fn rescanning_observes_changed_fractions() {
    let mut records = [record(20), record(10), record(30)];
    assert_eq!(nearest(&records), Some(1));
    records[1].frac = i32::MAX;
    records[2].frac = 0;
    assert_eq!(nearest(&records), Some(2));
}
