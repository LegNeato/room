use core::simd::{cmp::SimdPartialOrd, num::SimdInt, Simd};

use crate::doom::m_bbox::BBox;

/// Rejects boxes that only touch. Doom requires positive overlap on both axes.
#[inline]
pub(super) fn boxes_overlap(a: [i32; 4], b: [i32; 4]) -> bool {
    let high = Simd::from_array([a[BBox::RIGHT], b[BBox::RIGHT], a[BBox::TOP], b[BBox::TOP]]);
    let low = Simd::from_array([
        b[BBox::LEFT],
        a[BBox::LEFT],
        b[BBox::BOTTOM],
        a[BBox::BOTTOM],
    ]);
    high.simd_gt(low).all()
}

#[inline]
pub(super) fn things_overlap(a: [i32; 2], b: [i32; 2], distance: i32) -> bool {
    let delta = Simd::from_array(a) - Simd::from_array(b);
    delta.abs().simd_lt(Simd::splat(distance)).all()
}

#[cfg(test)]
mod tests;
