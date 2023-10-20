use ggez::{
    graphics::{Image, ImageFormat, Rect},
    Context,
};

/// Representing a raw RGBA image
/// For some reason ggez::graphics::Image requires a
/// Context for an image to be created, so we use this instead
#[derive(Clone)]
pub struct RawImage {
    pub bounds: Rect,
    pub pixels: Vec<u8>,
}

/// Create an empty image
impl Default for RawImage {
    fn default() -> Self {
        Self {
            bounds: Rect {
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
            },
            pixels: Vec::new(),
        }
    }
}

impl RawImage {
    /// Create a ggez Image from this type using a context
    pub fn to_image(&self, ctx: &mut Context) -> Image {
        Image::from_pixels(
            ctx,
            &self.pixels[..],
            ImageFormat::Rgba8Unorm,
            self.bounds.w as u32,
            self.bounds.h as u32,
        )
    }
}
