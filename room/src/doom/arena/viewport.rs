use super::Error;

/// A high-detail arena raster with the native 8:5 pixel aspect ratio.
/// The backing framebuffer keeps its native stride; only these pixels are drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Viewport(u8);

impl Viewport {
    pub const NATIVE: Self = Self(20);

    pub fn new(width: u32, height: u32) -> Result<Self, Error> {
        if !(16..=320).contains(&width)
            || !(10..=200).contains(&height)
            || !width.is_multiple_of(16)
            || !height.is_multiple_of(10)
            || width / 16 != height / 10
        {
            return Err(Error::Viewport);
        }
        Ok(Self((width / 16) as u8))
    }

    /// Covers a requested output without exceeding native resolution.
    /// Very large requests deliberately fall back to the exact native raster.
    pub fn for_output(width: u32, height: u32) -> Result<Self, Error> {
        if width == 0 || height == 0 {
            return Err(Error::Viewport);
        }
        let scale = width
            .min(320)
            .div_ceil(16)
            .max(height.min(200).div_ceil(10));
        Ok(Self(scale as u8))
    }

    /// One raster covers every viewer; an empty demand needs no rendering.
    pub fn covering(demands: impl IntoIterator<Item = (u32, u32)>) -> Result<Option<Self>, Error> {
        let mut scale = 0;
        for (width, height) in demands {
            scale = scale.max(Self::for_output(width, height)?.0);
        }
        Ok((scale != 0).then_some(Self(scale)))
    }

    pub const fn width(self) -> u16 {
        self.0 as u16 * 16
    }

    pub const fn height(self) -> u16 {
        self.0 as u16 * 10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outputs_cover_both_dimensions_and_keep_native_aspect() {
        for width in 1..=400 {
            for height in [1, 8, 10, 11, 100, 199, 200, 201, u32::MAX] {
                let viewport = Viewport::for_output(width, height).unwrap();
                assert!(u32::from(viewport.width()) >= width.min(320));
                assert!(u32::from(viewport.height()) >= height.min(200));
                assert_eq!(viewport.width() * 5, viewport.height() * 8);
            }
        }
        assert_eq!(Viewport::for_output(8, 8), Viewport::new(16, 10));
        assert_eq!(Viewport::for_output(200, 200), Ok(Viewport::NATIVE));
        assert_eq!(
            Viewport::for_output(u32::MAX, u32::MAX),
            Ok(Viewport::NATIVE)
        );
    }

    #[test]
    fn invalid_rasters_and_empty_outputs_are_rejected() {
        for (width, height) in [
            (0, 10),
            (16, 0),
            (8, 5),
            (16, 11),
            (32, 10),
            (321, 200),
            (320, 201),
            (u32::MAX, u32::MAX),
        ] {
            assert_eq!(Viewport::new(width, height), Err(Error::Viewport));
        }
        assert!(Viewport::for_output(0, 8).is_err());
        assert!(Viewport::for_output(8, 0).is_err());
    }
}
