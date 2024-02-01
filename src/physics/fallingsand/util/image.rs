//! Image utilities
//! I found it useful to write my own image class in ggez and it has been useful in bevy as well
//! keeps us from having to use specific bevy types in the physics engine

use bevy::{
    math::Rect,
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::Image,
    },
};

/// Representing a raw RGBA image
/// Game engine agnostic, full ownership, no lifetimes, not a component
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
            bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
            pixels: Vec::new(),
        }
    }
}

impl RawImage {
    // /// Save the image to a file
    // pub fn save(&self, ctx: &mut Context, path: &str) -> Result<(), ggez::GameError> {
    //     let img = self.to_image(ctx);
    //     img.encode(ctx, ImageEncodingFormat::Png, path)
    // }

    /// Convert to a bevy image
    /// Load this into an asset server to get a texture like the following
    /// ```ignore
    /// let image: RawImage = RawImage::default();
    /// let image_handle: Handle<Image> = asset_server.add(image.to_bevy_image());
    /// let material_handle: Handle<ColorMaterial> = materials.add(image_handle.into());
    /// ```
    pub fn to_bevy_image(self) -> Image {
        let size = Extent3d {
            width: self.bounds.width() as u32,
            height: self.bounds.height() as u32,
            depth_or_array_layers: 1,
        };

        Image::new(
            size,
            TextureDimension::D2,
            self.pixels,
            TextureFormat::Rgba8UnormSrgb, // Assuming RGBA format
        )
    }
}
