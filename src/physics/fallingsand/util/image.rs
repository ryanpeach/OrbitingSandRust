use ggez::{
    graphics::{Image, ImageFormat, Rect},
    Context,
};

use super::grid::Grid;

/// Representing a raw RGBA image
/// For some reason ggez::graphics::Image requires a
/// Context for an image to be created, so we use this instead
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

    /// Combine a list of images into one image
    /// The images are placed on the canvas according to their bounds
    /// This dramatically increases draw speed in testing.
    /// TODO: Test
    pub fn combine(vec_grid: &[Grid<RawImage>]) -> RawImage {
        let lst = vec_grid.iter().flatten().collect::<Vec<&RawImage>>();
        // Calculate total width and height for the canvas
        let width: f32 = lst
            .iter()
            .map(|img| img.bounds.w + img.bounds.x)
            .fold(0.0, |a, b| a.max(b));
        let height: f32 = lst
            .iter()
            .map(|img| img.bounds.h + img.bounds.y)
            .fold(0.0, |a, b| a.max(b));
        let min_x: f32 = lst
            .iter()
            .map(|img| img.bounds.x)
            .fold(f32::INFINITY, |a, b| a.min(b));
        let min_y: f32 = lst
            .iter()
            .map(|img| img.bounds.y)
            .fold(f32::INFINITY, |a, b| a.min(b));
        let mut canvas = vec![0; width as usize * height as usize * 4usize]; // Assuming pixels are u8 or some type and initialized to 0

        for image in lst {
            for y in 0..image.bounds.h as usize {
                // Get a slice of the source and destination
                let src_start_idx = y * (image.bounds.w as usize * 4);
                let src_end_idx = src_start_idx + (image.bounds.w as usize * 4);
                let src_slice = &image.pixels[src_start_idx..src_end_idx];

                let dst_start_idx =
                    (image.bounds.x as usize + (image.bounds.y as usize + y) * width as usize) * 4;
                let dst_end_idx = dst_start_idx + (image.bounds.w as usize * 4);
                let dst_slice = &mut canvas[dst_start_idx..dst_end_idx];

                // Use copy_from_slice for faster copying
                dst_slice.copy_from_slice(src_slice);
            }
        }

        RawImage {
            bounds: Rect::new(min_x, min_y, width, height),
            pixels: canvas,
        }
    }
}
