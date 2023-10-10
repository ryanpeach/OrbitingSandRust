use ggez::{
    glam::Vec2,
    graphics::{Image, ImageFormat, MeshData, Rect, Vertex},
};

/// For some reason ggez::graphics::Image requires a Context to be created
pub struct RawImage {
    pub bounds: Rect,
    pub pixels: Vec<u8>,
}

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
    pub fn to_image(&self, ctx: &mut ggez::Context) -> ggez::graphics::Image {
        Image::from_pixels(
            ctx,
            &self.pixels[..],
            ImageFormat::Rgba8Unorm,
            self.bounds.w as u32,
            self.bounds.h as u32,
        )
    }

    // TODO: Test
    pub fn combine(lst: &Vec<RawImage>) -> RawImage {
        // Calculate total width and height for the canvas
        let width: f32 = lst.iter().map(|img| img.bounds.w).sum();
        let height: f32 = lst.iter().map(|img| img.bounds.h).sum();
        let min_x: f32 = lst
            .iter()
            .map(|img| img.bounds.x)
            .fold(f32::INFINITY, |a, b| a.min(b));
        let min_y: f32 = lst
            .iter()
            .map(|img| img.bounds.y)
            .fold(f32::INFINITY, |a, b| a.min(b));
        let mut canvas = vec![0; (width * height) as usize]; // Assuming pixels are u8 or some type and initialized to 0

        let mut current_x = 0;

        for image in lst {
            for y in 0..image.bounds.h as u32 {
                // Get a slice of the source and destination
                let src_slice = &image.pixels[(y * (image.bounds.h as u32)) as usize
                    ..((y + 1) * (image.bounds.w as u32)) as usize];
                let dst_slice = &mut canvas[(current_x + y * (width as u32)) as usize
                    ..(current_x + y * (width as u32) + (image.bounds.w as u32)) as usize];

                // Use copy_from_slice for faster copying
                dst_slice.copy_from_slice(src_slice);
            }
            current_x += image.bounds.w as u32; // Move the position for the next image
        }

        RawImage {
            bounds: Rect::new(min_x, min_y, width, height),
            pixels: canvas,
        }
    }
}

/// For some reason a MeshData object has a lifetime and is a set of borrows.
pub struct OwnedMeshData {
    pub uv_bounds: Rect,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Default for OwnedMeshData {
    fn default() -> Self {
        Self {
            uv_bounds: Rect {
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
            },
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl OwnedMeshData {
    pub fn to_mesh_data(&self) -> MeshData {
        MeshData {
            vertices: &self.vertices,
            indices: self.indices.as_slice(),
        }
    }

    /// You need to add the previous last_idx to all the elements of the next indices
    /// You also need to un_normalize the uvs and then re_normalize them at the end
    pub fn combine(lst: &Vec<OwnedMeshData>) -> OwnedMeshData {
        let mut combined_vertices = Vec::new();
        let mut combined_indices = Vec::new();

        let mut last_idx = 0u32;

        // This is to find the max and min bounds for the UVs
        let mut min_u = f32::MAX;
        let mut min_v = f32::MAX;
        let mut max_u = f32::MIN;
        let mut max_v = f32::MIN;

        for mesh_data in lst {
            for vertex in &mesh_data.vertices {
                let un_normalized_u = vertex.uv[0] * mesh_data.uv_bounds.w + mesh_data.uv_bounds.x;
                let un_normalized_v = vertex.uv[1] * mesh_data.uv_bounds.h + mesh_data.uv_bounds.y;

                min_u = min_u.min(un_normalized_u);
                min_v = min_v.min(un_normalized_v);
                max_u = max_u.max(un_normalized_u);
                max_v = max_v.max(un_normalized_v);
            }

            combined_vertices.extend(mesh_data.vertices.clone());

            for index in mesh_data.indices.clone() {
                combined_indices.push(index + last_idx);
            }

            last_idx += mesh_data.vertices.len() as u32;
        }

        let uv_width = max_u - min_u;
        let uv_height = max_v - min_v;

        // Re-normalizing the UVs
        for vertex in &mut combined_vertices {
            vertex.uv[0] = (vertex.uv[0] * uv_width + min_u - min_u) / uv_width;
            vertex.uv[1] = (vertex.uv[1] * uv_height + min_v - min_v) / uv_height;
        }

        OwnedMeshData {
            uv_bounds: Rect::new(min_u, min_v, uv_width, uv_height),
            vertices: combined_vertices,
            indices: combined_indices,
        }
    }
}

/// The different ways to draw a chunk
#[derive(Copy, Clone, PartialEq)]
pub enum DrawMode {
    TexturedMesh,
    Outline,
    TriangleWireframe,
    UVWireframe,
}

/// Tests if a number is a power of 2
/// I found it's important that some values are powers of two in order to enable grid_iter to work
pub fn is_pow_2(n: usize) -> bool {
    n != 0 && (n & (n - 1)) == 0
}

/// Tests if a step is valid for a grid_iter
/// A valid step is 1, len - 1, or a factor of len - 1
/// We convert things less than 1 to 1, or things greater than len - 1 to len - 1
pub fn valid_step(len: usize, step: usize) -> bool {
    step <= 1 || step >= len - 1 || (len - 1) % step == 0
}

/// Finds a point halfway between two points
pub fn interpolate_points(p1: &Vec2, p2: &Vec2) -> Vec2 {
    Vec2::new((p1.x + p2.x) * 0.5, (p1.y + p2.y) * 0.5)
}

/// This is like the "skip" method but it always keeps the first and last item
/// If it is larger than the number of items, it will just return the first and last item
/// If the step is not a multiple of the number of items, it will round down to the previous multiple
pub fn grid_iter(start: usize, end: usize, step: usize) -> Vec<usize> {
    let len = end - start;
    if len <= 1 {
        // Return [0]
        return vec![start];
    }
    if step >= len {
        return vec![start, end - 1];
    }
    debug_assert_ne!(step, 0, "Step should not be 0.");

    debug_assert!(
        valid_step(len, step),
        "Step should be 1, len - 1, or a factor of len - 1. len: {}, step: {}",
        len,
        step
    );

    let start_item = start;
    let end_item = end - 1;

    let mut out = Vec::new();
    out.push(start_item);
    for i in (start_item + step..end_item).step_by(step) {
        if i % step == 0 && i != 0 && i != len - 1 {
            out.push(i);
        }
    }
    out.push(end_item);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_element() {
        let v: Vec<_> = grid_iter(0, 1, 16);
        assert_eq!(v, vec![0]);
    }

    #[test]
    fn test_two_elements() {
        let v: Vec<_> = grid_iter(0, 2, 16);
        assert_eq!(v, vec![0, 1]);
    }

    #[test]
    fn test_basic() {
        let v: Vec<_> = grid_iter(0, 11, 2);
        assert_eq!(v, vec![0, 2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_step_one() {
        let v: Vec<_> = grid_iter(0, 11, 1);
        assert_eq!(v, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    /// At a large step size, we should just get the first and last elements
    #[test]
    fn test_large_step() {
        let v: Vec<_> = grid_iter(0, 10, 20);
        assert_eq!(v, vec![0, 9]);
    }

    #[test]
    fn test_basic_5() {
        let v: Vec<_> = grid_iter(0, 5, 2);
        assert_eq!(v, vec![0, 2, 4]);
    }

    /// In this case, because three doesnt work, we automatically round down to 2
    #[test]
    fn test_round_7() {
        let v: Vec<_> = grid_iter(0, 7, 3);
        assert_eq!(v, vec![0, 3, 6]);
    }

    #[test]
    fn test_is_pow_2() {
        assert!(is_pow_2(1));
        assert!(is_pow_2(2));
        assert!(is_pow_2(4));
        assert!(is_pow_2(8));
        assert!(!is_pow_2(0));
        assert!(!is_pow_2(3));
        assert!(!is_pow_2(6));
    }

    #[test]
    fn test_valid_step() {
        // Tests for len = 10
        assert!(valid_step(10, 1)); // 1 is valid for any len
        assert!(valid_step(10, 9)); // len - 1 is valid for any len
        assert!(valid_step(10, 3)); // 3 is a factor of len - 1
        assert!(!valid_step(10, 2)); // 2 is not a factor of len - 1 and not within the valid range
        assert!(!valid_step(10, 8)); // 8 is not a factor of len - 1 and not within the valid range
    }

    #[test]
    fn test_interpolate_points() {
        let p1 = Vec2::new(0.0, 0.0);
        let p2 = Vec2::new(2.0, 2.0);
        let midpoint = interpolate_points(&p1, &p2);

        assert_eq!(midpoint.x, 1.0);
        assert_eq!(midpoint.y, 1.0);

        let p3 = Vec2::new(-2.0, -1.0);
        let p4 = Vec2::new(2.0, 3.0);
        let midpoint2 = interpolate_points(&p3, &p4);

        assert_eq!(midpoint2.x, 0.0);
        assert_eq!(midpoint2.y, 1.0);
    }
}
