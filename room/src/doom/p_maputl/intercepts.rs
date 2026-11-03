use core::simd::{
    cmp::{SimdPartialEq, SimdPartialOrd},
    num::SimdInt,
    ptr::SimdConstPtr,
    Simd,
};

use super::intercept_t;

const LANES: usize = 32;
const OFFSETS: Simd<usize, LANES> = Simd::from_array([
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31,
]);

/// Finds the first nearest unvisited intercept. Equal distances retain map order.
#[inline]
pub(super) fn nearest(records: &[intercept_t]) -> Option<usize> {
    let mut nearest = None;
    let mut distance = i32::MAX;
    for start in (0..records.len()).step_by(LANES) {
        let enabled = OFFSETS.simd_lt(Simd::splat(records.len() - start));
        let indices = OFFSETS + Simd::splat(start);
        let offsets = indices * Simd::splat(core::mem::size_of::<intercept_t>())
            + Simd::splat(core::mem::offset_of!(intercept_t, frac));
        let pointers = Simd::splat(records.as_ptr().cast::<u8>())
            .wrapping_add(offsets)
            .cast::<i32>();
        // Each enabled pointer selects an initialized frac field in records.
        let fractions =
            unsafe { Simd::gather_select_ptr(pointers, enabled, Simd::splat(i32::MAX)) };
        let minimum = fractions.reduce_min();
        if minimum < distance {
            distance = minimum;
            let lane = fractions
                .simd_eq(Simd::splat(minimum))
                .to_bitmask()
                .trailing_zeros();
            nearest = Some(start + lane as usize);
        }
    }
    nearest
}

#[cfg(test)]
mod tests;
