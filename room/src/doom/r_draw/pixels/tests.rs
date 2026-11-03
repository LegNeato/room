use super::*;

const SENTINEL: u8 = 0xa5;

fn colormap() -> [u8; 256] {
    core::array::from_fn(|index| (index as u8).wrapping_mul(73).rotate_left(3))
}

mod columns {
    use super::*;

    fn compare<const COPIES: usize>(count: usize, fraction: i32, step: i32) {
        let source: [u8; 128] = core::array::from_fn(|index| (index as u8).wrapping_mul(59));
        let colors = colormap();
        let stride = 320;
        let mut actual = vec![SENTINEL; 320 * 202];
        let mut expected = actual.clone();
        let mut position = fraction;
        for row in 0..count {
            let pixel = colors[source[((position >> 16) & 127) as usize] as usize];
            for copy in 0..COPIES {
                expected[17 + row * stride + copy] = pixel;
            }
            position = position.wrapping_add(step);
        }
        let destinations = core::array::from_fn(|copy| actual.as_mut_ptr().wrapping_add(17 + copy));
        // The arrays cover all wrapped samples and all distinct output addresses.
        unsafe {
            Column {
                count,
                fraction,
                step,
                source: source.as_ptr(),
                colormap: colors.as_ptr(),
            }
            .draw::<COPIES>(destinations, stride);
        }
        assert_eq!(
            actual, expected,
            "count={count} fraction={fraction} step={step}"
        );
    }

    #[test]
    fn matches_scalar_at_every_tail_length() {
        for count in 0..=200 {
            for (fraction, step) in [
                (0, 1 << 16),
                (-65537, 32769),
                (i32::MAX, i32::MAX),
                (i32::MIN, i32::MIN),
                (12345, -99999),
                (77 << 16, 0),
            ] {
                compare::<1>(count, fraction, step);
                compare::<2>(count, fraction, step);
            }
        }
    }

    #[test]
    fn short_posts_do_not_require_a_full_texture_column() {
        let source = [42_u8];
        let colors = colormap();
        let mut actual = [SENTINEL; 33 * 7 + 1];
        unsafe {
            Column {
                count: 33,
                fraction: 0,
                step: 0,
                source: source.as_ptr(),
                colormap: colors.as_ptr(),
            }
            .draw([actual.as_mut_ptr()], 7);
        }
        for (index, pixel) in actual.into_iter().enumerate() {
            assert_eq!(
                pixel,
                if index < 33 * 7 && index % 7 == 0 {
                    colors[42]
                } else {
                    SENTINEL
                }
            );
        }
    }

    #[test]
    fn empty_column_does_not_access_pointers() {
        unsafe {
            Column {
                count: 0,
                fraction: i32::MAX,
                step: i32::MAX,
                source: core::ptr::null(),
                colormap: core::ptr::null(),
            }
            .draw([core::ptr::null_mut()], 320);
        }
    }
}

mod spans {
    use super::*;

    fn compare<const COPIES: usize>(count: usize, mut position: u32, step: u32) {
        let source: [u8; 4096] = core::array::from_fn(|index| (index as u8).wrapping_mul(41));
        let colors = colormap();
        let mut actual = vec![SENTINEL; 642];
        let mut expected = actual.clone();
        unsafe {
            Span {
                count,
                position,
                step,
                source: source.as_ptr(),
                colormap: colors.as_ptr(),
            }
            .draw::<COPIES>(actual.as_mut_ptr().add(1));
        }
        for column in 0..count {
            let sample = ((position >> 26) | ((position >> 4) & 0xfc0)) as usize;
            for copy in 0..COPIES {
                expected[1 + column * COPIES + copy] = colors[source[sample] as usize];
            }
            position = position.wrapping_add(step);
        }
        assert_eq!(actual, expected, "count={count} step={step}");
    }

    #[test]
    fn matches_scalar_with_packed_carries_and_wraparound() {
        for count in 0..=320 {
            for (position, step) in [
                (0, 0),
                (0xffff, 1),
                (0x1234_abcd, 0x7fff_ffff),
                (u32::MAX, u32::MAX),
                (0x8765_4321, 0xffff_0001),
            ] {
                compare::<1>(count, position, step);
                compare::<2>(count, position, step);
            }
        }
    }

    #[test]
    fn empty_span_does_not_access_pointers() {
        unsafe {
            Span {
                count: 0,
                position: u32::MAX,
                step: u32::MAX,
                source: core::ptr::null(),
                colormap: core::ptr::null(),
            }
            .draw::<2>(core::ptr::null_mut());
        }
    }
}
