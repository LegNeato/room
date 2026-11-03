//! Independent pixel operations for the column and span renderers.

use core::simd::{
    cmp::SimdPartialOrd,
    num::SimdUint,
    ptr::{SimdConstPtr, SimdMutPtr},
    Simd,
};

// A logical vector covers one GPU warp when compiled through FTL.
const LANES: usize = 32;
const OFFSETS: Simd<u32, LANES> = Simd::from_array([
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31,
]);

pub(super) struct Column {
    pub count: usize,
    pub fraction: i32,
    pub step: i32,
    pub source: *const u8,
    pub colormap: *const u8,
}

impl Column {
    /// Draws a column into each destination.
    ///
    /// # Safety
    /// For a nonempty column, each selected texture byte and each of the 256 colormap entries must be
    /// readable. Each destination must have `count` writable bytes at `stride`
    /// intervals. These destinations must be distinct and must not overlap the
    /// texture or colormap. `count` must fit in `u32`.
    #[inline]
    pub(super) unsafe fn draw<const COPIES: usize>(
        self,
        destinations: [*mut u8; COPIES],
        stride: usize,
    ) {
        let mut fraction =
            Simd::splat(self.fraction as u32) + OFFSETS * Simd::splat(self.step as u32);
        let advance = Simd::splat((self.step as u32).wrapping_mul(LANES as u32));
        for start in (0..self.count).step_by(LANES) {
            let enabled = OFFSETS
                .simd_lt(Simd::splat((self.count - start) as u32))
                .cast();
            let samples = ((fraction >> 16) & Simd::splat(127)).cast::<usize>();
            let source = Simd::splat(self.source).wrapping_add(samples);
            // Disabled lanes can point beyond a short sprite post. They never load.
            let texels = unsafe { Simd::gather_select_ptr(source, enabled, Simd::splat(0)) };
            let colors = Simd::splat(self.colormap).wrapping_add(texels.cast::<usize>());
            let pixels = unsafe { Simd::gather_select_ptr(colors, enabled, Simd::splat(0)) };
            let offsets = (OFFSETS.cast::<usize>() + Simd::splat(start)) * Simd::splat(stride);
            for destination in destinations {
                let pointers = Simd::splat(destination).wrapping_add(offsets);
                unsafe { pixels.scatter_select_ptr(pointers, enabled) };
            }
            fraction += advance;
        }
    }
}

pub(super) struct Span {
    pub count: usize,
    pub position: u32,
    pub step: u32,
    pub source: *const u8,
    pub colormap: *const u8,
}

impl Span {
    /// Draws a span with one or two output bytes per sampled texel.
    ///
    /// # Safety
    /// For a nonempty span, `source` must contain 4096 readable bytes and
    /// `colormap` must contain 256.
    /// `destination` must have `count` writable bytes, or twice that when
    /// `DOUBLE` is true. The output must not overlap either input. `count` must
    /// fit in `u32`.
    #[inline]
    pub(super) unsafe fn draw<const DOUBLE: bool>(self, destination: *mut u8) {
        let copies = if DOUBLE { 2 } else { 1 };
        let mut position = Simd::splat(self.position) + OFFSETS * Simd::splat(self.step);
        let advance = Simd::splat(self.step.wrapping_mul(LANES as u32));
        for start in (0..self.count).step_by(LANES) {
            let enabled = OFFSETS
                .simd_lt(Simd::splat((self.count - start) as u32))
                .cast();
            // Keep the packed addition: carry from the Y fraction affects X.
            let samples =
                ((position >> 26) | ((position >> 4) & Simd::splat(0xfc0))).cast::<usize>();
            let source = Simd::splat(self.source).wrapping_add(samples);
            let texels = unsafe { Simd::gather_select_ptr(source, enabled, Simd::splat(0)) };
            let colors = Simd::splat(self.colormap).wrapping_add(texels.cast::<usize>());
            let pixels = unsafe { Simd::gather_select_ptr(colors, enabled, Simd::splat(0)) };
            let offsets = (OFFSETS.cast::<usize>() + Simd::splat(start)) * Simd::splat(copies);
            for copy in 0..copies {
                let pointers = Simd::splat(destination).wrapping_add(offsets + Simd::splat(copy));
                unsafe { pixels.scatter_select_ptr(pointers, enabled) };
            }
            position += advance;
        }
    }
}

#[cfg(test)]
mod tests;
