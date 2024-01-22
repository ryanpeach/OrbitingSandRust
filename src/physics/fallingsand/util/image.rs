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

#[cfg(test)]
mod tests {

    use crate::physics::fallingsand::data::element_directory::ElementGridDir;
    use crate::physics::fallingsand::elements::{element::Element, sand::Sand, vacuum::Vacuum};
    use crate::physics::fallingsand::mesh::coordinate_directory::CoordinateDirBuilder;

    /// The default element grid directory for testing
    fn get_element_grid_dir() -> ElementGridDir {
        let coordinate_dir = CoordinateDirBuilder::new()
            .cell_radius(1.0)
            .num_layers(7)
            .first_num_radial_lines(6)
            .second_num_concentric_circles(3)
            .max_concentric_circles_per_chunk(64)
            .max_radial_lines_per_chunk(64)
            .build();
        let fill0: &dyn Element = &Vacuum::default();
        let fill1: &dyn Element = &Sand::default();
        ElementGridDir::new_checkerboard(coordinate_dir, fill0, fill1)
    }

    // #[test]
    // fn test_combine() {
    //     let element_grid = get_element_grid_dir();
    //     let meshes = element_grid
    //         .get_coordinate_dir()
    //         .get_mesh_data(MeshDrawMode::TexturedMesh);
    //     let combined_meshes = OwnedMeshData::combine(&meshes);
    //     let textures = element_grid.get_textures();
    //     let j_size = textures
    //         .iter()
    //         .filter(|x| x.0.k == 0)
    //         .map(|x| x.1.bounds.h as usize)
    //         .sum::<usize>();
    //     let k_size = textures
    //         .iter()
    //         .filter(|x| {
    //             x.0.j == 0 && x.0.i == element_grid.get_coordinate_dir().get_num_layers() - 1
    //         })
    //         .map(|x| x.1.bounds.w as usize)
    //         .sum::<usize>();
    //     let img = RawImage::combine(textures, combined_meshes.uv_bounds);
    //     assert_eq!(img.bounds.h, j_size as f32);
    //     assert_eq!(img.bounds.w, k_size as f32);
    // }
}
