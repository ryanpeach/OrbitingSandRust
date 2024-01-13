use hashbrown::HashMap;

use crate::physics::util::vectors::Rect;

use super::vectors::ChunkIjkVector;

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
    // /// Create a ggez Image from this type using a context
    // pub fn to_image(&self, ctx: &mut Context) -> Image {
    //     Image::from_pixels(
    //         ctx,
    //         &self.pixels[..],
    //         ImageFormat::Rgba8Unorm,
    //         self.bounds.w as u32,
    //         self.bounds.h as u32,
    //     )
    // }

    // /// Save the image to a file
    // pub fn save(&self, ctx: &mut Context, path: &str) -> Result<(), ggez::GameError> {
    //     let img = self.to_image(ctx);
    //     img.encode(ctx, ImageEncodingFormat::Png, path)
    // }

    /// Combine a list of images into one image
    /// The images are placed on the canvas according to their bounds
    /// This dramatically increases draw speed in testing.
    /// TODO: Test
    pub fn combine(vec_grid: HashMap<ChunkIjkVector, RawImage>, uvbounds: Rect) -> RawImage {
        let lst = vec_grid.into_iter().map(|x| x.1).collect::<Vec<RawImage>>();
        // Calculate total width and height for the canvas
        let width: f32 = uvbounds.w;
        let height: f32 = uvbounds.h;
        let min_x: f32 = uvbounds.x;
        let min_y: f32 = uvbounds.y;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::fallingsand::data::element_directory::ElementGridDir;
    use crate::physics::fallingsand::elements::{element::Element, sand::Sand, vacuum::Vacuum};
    use crate::physics::fallingsand::mesh::coordinate_directory::CoordinateDirBuilder;
    use crate::physics::fallingsand::util::enums::MeshDrawMode;
    use crate::physics::fallingsand::util::mesh::OwnedMeshData;

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

    #[test]
    fn test_combine() {
        let element_grid = get_element_grid_dir();
        let meshes = element_grid
            .get_coordinate_dir()
            .get_mesh_data(MeshDrawMode::TexturedMesh);
        let combined_meshes = OwnedMeshData::combine(&meshes);
        let textures = element_grid.get_textures();
        let j_size = textures
            .iter()
            .filter(|x| x.0.k == 0)
            .map(|x| x.1.bounds.h as usize)
            .sum::<usize>();
        let k_size = textures
            .iter()
            .filter(|x| {
                x.0.j == 0 && x.0.i == element_grid.get_coordinate_dir().get_num_layers() - 1
            })
            .map(|x| x.1.bounds.w as usize)
            .sum::<usize>();
        let img = RawImage::combine(textures, combined_meshes.uv_bounds);
        assert_eq!(img.bounds.h, j_size as f32);
        assert_eq!(img.bounds.w, k_size as f32);
    }
}
