use crate::doom::m_fixed::mul_simd;

use core::simd::{cmp::SimdPartialOrd, Select, Simd};

/// Fixed-point line geometry without the map's linked structures.
#[derive(Clone, Copy)]
pub(super) struct Line {
    pub origin: [i32; 2],
    pub direction: [i32; 2],
}

impl Line {
    /// Tests both endpoints with the linedef renderer's fixed-point precision.
    #[inline]
    pub(super) fn sides(self, points: [[i32; 2]; 2]) -> [i32; 2] {
        let x = Simd::from_array([points[0][0], points[1][0]]);
        let y = Simd::from_array([points[0][1], points[1][1]]);
        if let Some(sides) = self.axial_sides(x, y) {
            return sides;
        }
        let dx = x - Simd::splat(self.origin[0]);
        let dy = y - Simd::splat(self.origin[1]);
        let left = mul_simd(Simd::splat(self.direction[1] >> 16), dx);
        let right = mul_simd(dy, Simd::splat(self.direction[0] >> 16));
        right
            .simd_ge(left)
            .select(Simd::splat(1), Simd::splat(0))
            .to_array()
    }

    /// Tests both endpoints with the trace's sign shortcut and eight-bit shifts.
    #[inline]
    pub(super) fn trace_sides(self, points: [[i32; 2]; 2]) -> [i32; 2] {
        let x = Simd::from_array([points[0][0], points[1][0]]);
        let y = Simd::from_array([points[0][1], points[1][1]]);
        if let Some(sides) = self.axial_sides(x, y) {
            return sides;
        }
        let dx = x - Simd::splat(self.origin[0]);
        let dy = y - Simd::splat(self.origin[1]);
        let direction_x = Simd::splat(self.direction[0]);
        let direction_y = Simd::splat(self.direction[1]);
        let shortcut = (direction_y ^ direction_x ^ dx ^ dy).simd_lt(Simd::splat(0));
        let sign_side = (direction_y ^ dx).simd_lt(Simd::splat(0));
        let left = mul_simd(direction_y >> 8, dx >> 8);
        let right = mul_simd(dy >> 8, direction_x >> 8);
        let side = (shortcut & sign_side) | (!shortcut & right.simd_ge(left));
        side.select(Simd::splat(1), Simd::splat(0)).to_array()
    }

    #[inline]
    fn axial_sides(self, x: Simd<i32, 2>, y: Simd<i32, 2>) -> Option<[i32; 2]> {
        let [dx, dy] = self.direction;
        if dx == 0 {
            Some(
                x.simd_le(Simd::splat(self.origin[0]))
                    .select(
                        Simd::splat(i32::from(dy > 0)),
                        Simd::splat(i32::from(dy < 0)),
                    )
                    .to_array(),
            )
        } else if dy == 0 {
            Some(
                y.simd_le(Simd::splat(self.origin[1]))
                    .select(
                        Simd::splat(i32::from(dx < 0)),
                        Simd::splat(i32::from(dx > 0)),
                    )
                    .to_array(),
            )
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests;
