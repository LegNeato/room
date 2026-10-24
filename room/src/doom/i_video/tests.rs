use super::*;
use std::sync::{Mutex, MutexGuard};

/// `COLORS` is a `static mut` shared across the whole crate; the
/// tests below mutate it, so they must run serially.
static PALETTE_LOCK: Mutex<()> = Mutex::new(());

/// RAII guard that owns the palette lock for the duration of a
/// test and restores the `COLORS` snapshot captured at acquisition
/// time on drop, even on panic. Keeps the global palette
/// invisible to subsequent tests in the same binary.
struct PaletteGuard {
    _lock: MutexGuard<'static, ()>,
    saved: [u32; 256],
    gamma: c_int,
    pixels: *mut u8,
    mode: OutputMode,
    scaling: c_int,
    framebuffer: FB_ScreenInfo,
}

impl PaletteGuard {
    fn acquire() -> Self {
        let lock = PALETTE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let saved = unsafe { ptr::addr_of!(COLORS).read() };
        unsafe {
            Self {
                _lock: lock,
                saved,
                gamma: usegamma,
                pixels: I_VideoBuffer,
                mode: OUTPUT_MODE,
                scaling: fb_scaling,
                framebuffer: ptr::addr_of!(s_Fb).read(),
            }
        }
    }
}

impl Drop for PaletteGuard {
    fn drop(&mut self) {
        unsafe {
            ptr::addr_of_mut!(COLORS).write(self.saved);
            usegamma = self.gamma;
            I_VideoBuffer = self.pixels;
            OUTPUT_MODE = self.mode;
            fb_scaling = self.scaling;
            ptr::swap(ptr::addr_of_mut!(s_Fb), &mut self.framebuffer);
        }
    }
}

fn set_test_palette(entries: &[(u8, u8, u8)]) {
    let colors = ptr::addr_of_mut!(COLORS) as *mut u32;
    unsafe {
        for i in 0..256 {
            let (r, g, b) = entries.get(i).copied().unwrap_or((0, 0, 0));
            colors
                .add(i)
                .write((r as u32) << 16 | (g as u32) << 8 | b as u32);
        }
    }
}

#[test]
fn indexed_view_requires_initialized_pixels() {
    let _guard = PaletteGuard::acquire();
    unsafe {
        let saved = I_VideoBuffer;
        I_VideoBuffer = ptr::null_mut();
        assert!(indexed_frame().is_none());
        let mut pixels = [0u8; (SCREENWIDTH * SCREENHEIGHT) as usize];
        pixels[17] = 42;
        I_VideoBuffer = pixels.as_mut_ptr();
        let (indices, palette) = indexed_frame().unwrap();
        assert_eq!(indices[17], 42);
        assert_eq!(palette.as_ptr(), ptr::addr_of!(COLORS).cast());
        I_VideoBuffer = saved;
    }
}

#[test]
fn palette_packs_every_gamma_corrected_channel() {
    let _guard = PaletteGuard::acquire();
    unsafe {
        let saved_gamma = usegamma;
        let mut palette = [0u8; 256 * 3];
        for (index, rgb) in palette.chunks_exact_mut(3).enumerate() {
            rgb.copy_from_slice(&[
                index as u8,
                (255 - index) as u8,
                (index as u8).rotate_left(3),
            ]);
        }
        for (gamma, table) in gammatable.iter().enumerate() {
            usegamma = gamma as c_int;
            I_SetPalette(palette.as_mut_ptr());
            for (index, rgb) in palette.chunks_exact(3).enumerate() {
                let expected = (table[rgb[0] as usize] as u32) << 16
                    | (table[rgb[1] as usize] as u32) << 8
                    | table[rgb[2] as usize] as u32;
                assert_eq!(
                    ptr::addr_of!(COLORS).cast::<u32>().add(index).read(),
                    expected
                );
            }
        }
        usegamma = saved_gamma;
    }
}

#[test]
fn packed_palette_preserves_framebuffer_formats() {
    let _guard = PaletteGuard::acquire();
    let entries: Vec<_> = (0..=255u8)
        .map(|r| (r, 255 - r, r.rotate_left(3)))
        .collect();
    set_test_palette(&entries);
    let mut indices: Vec<_> = (0..=255u8).collect();
    let mut pixels = [0u32; 256];
    let mut pixels565 = [0u16; 256];
    unsafe {
        fb_scaling = 1;
        s_Fb.bits_per_pixel = 32;
        for (red, green, blue) in [(16, 8, 0), (0, 8, 16)] {
            s_Fb.red.offset = red;
            s_Fb.green.offset = green;
            s_Fb.blue.offset = blue;
            cmap_to_fb(pixels.as_mut_ptr().cast(), indices.as_mut_ptr(), 256);
            for (actual, &(r, g, b)) in pixels.iter().zip(&entries) {
                assert_eq!(
                    *actual,
                    (r as u32) << red | (g as u32) << green | (b as u32) << blue
                );
            }
        }
        s_Fb.bits_per_pixel = 16;
        cmap_to_fb(pixels565.as_mut_ptr().cast(), indices.as_mut_ptr(), 256);
        for (actual, &(r, g, b)) in pixels565.iter().zip(&entries) {
            assert_eq!(
                *actual,
                ((r as u16 & 0xf8) << 8) | ((g as u16 & 0xfc) << 3) | (b as u16 >> 3)
            );
        }
    }
}

#[test]
fn indexed_draw_does_not_access_framebuffer_memory() {
    let _guard = PaletteGuard::acquire();
    unsafe {
        set_output_mode(OutputMode::Indexed);
        I_VideoBuffer = ptr::null_mut();
        // The callback needs no pixels in the headless test platform.
        // Conversion would dereference this null input.
        I_FinishUpdate();
    }
}

#[test]
fn get_palette_index_nearest_rgb() {
    let _guard = PaletteGuard::acquire();
    let palette: [(u8, u8, u8); 4] = [(0, 0, 0), (255, 0, 0), (0, 255, 0), (0, 0, 255)];
    set_test_palette(&palette);

    unsafe {
        assert_eq!(I_GetPaletteIndex(0, 0, 0), 0, "exact black");
        assert_eq!(I_GetPaletteIndex(255, 0, 0), 1, "exact red");
        assert_eq!(I_GetPaletteIndex(0, 255, 0), 2, "exact green");
        assert_eq!(I_GetPaletteIndex(0, 0, 255), 3, "exact blue");

        assert_eq!(I_GetPaletteIndex(250, 5, 5), 1, "near red");
        assert_eq!(I_GetPaletteIndex(5, 250, 5), 2, "near green");
        assert_eq!(I_GetPaletteIndex(10, 10, 10), 0, "near black");

        assert_eq!(
            I_GetPaletteIndex(128, 0, 0),
            1,
            "128 is one step closer to 255 (diff 127^2) than to 0 (diff 128^2)"
        );
        assert_eq!(
            I_GetPaletteIndex(127, 0, 0),
            0,
            "127 is one step closer to 0 than to 255"
        );

        // Out-of-range inputs must not overflow the diff accumulator.
        assert_eq!(I_GetPaletteIndex(c_int::MAX, 0, 0), 1, "saturate to red");
        assert_eq!(I_GetPaletteIndex(c_int::MIN, 0, 0), 0, "saturate to black");
    }
}

#[test]
fn get_palette_index_returns_first_exact_match() {
    let _guard = PaletteGuard::acquire();

    // Index 42 is the target; index 200 is a duplicate. Strict
    // `diff < best_diff` plus the outer-scope `if diff == 0
    // { break }` together guarantee the first exact match wins
    // and the loop terminates there; this test pins the
    // first-match contract so regressions in either branch show
    // up as a wrong index.
    let mut palette: Vec<(u8, u8, u8)> = (0..256).map(|_| (200, 0, 0)).collect();
    palette[42] = (42, 0, 0);
    palette[200] = (42, 0, 0);
    set_test_palette(&palette);

    unsafe {
        assert_eq!(I_GetPaletteIndex(42, 0, 0), 42);
    }
}
