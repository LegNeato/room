/// Selects the image made available to the platform draw callback.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OutputMode {
    /// Expands every indexed pixel into the configured framebuffer format.
    #[default]
    Framebuffer,
    /// Leaves the framebuffer unchanged and exposes indexed pixels directly.
    Indexed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framebuffer_is_the_default() {
        assert_eq!(OutputMode::default(), OutputMode::Framebuffer);
        assert_ne!(OutputMode::Indexed, OutputMode::Framebuffer);
    }
}
