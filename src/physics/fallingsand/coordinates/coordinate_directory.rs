use crate::physics::fallingsand::util::grid::Grid;
use crate::physics::fallingsand::util::vectors::{ChunkIjkVector, JkVector};

use super::chunk_coords::ChunkCoords;
use super::core_coords::CoreChunkCoords;
use super::layer_coords::{PartialLayerChunkCoords, PartialLayerChunkCoordsBuilder};
use crate::physics::fallingsand::util::enums::MeshDrawMode;
use crate::physics::fallingsand::util::mesh::OwnedMeshData;
use ggez::glam::Vec2;
use ggez::graphics::{Rect, Vertex};

/// A structure that contains all the chunk coordinates for a celestial body
/// Useful for drawing the total mesh
#[derive(Clone)]
pub struct CoordinateDir {
    second_num_concentric_circles: usize,

    /// Every celestial body has a core
    /// The core is always the first layer and has 1 concentric circle
    core_chunk: CoreChunkCoords,

    /// Layers on top of the core
    /// Every index in the vec represents a layer
    /// The Grid then represents the chunks in that layer
    partial_chunks: Vec<Grid<PartialLayerChunkCoords>>,
}

/// A builder for CoordinateDir
/// Needs more parameters than CoordinateDir because
/// it assembles the chunks whereas CoordinateDir can re-derive
/// these parameters from the chunks themselves
pub struct CoordinateDirBuilder {
    cell_radius: f32,
    num_layers: usize,
    first_num_radial_lines: usize,
    second_num_concentric_circles: usize,
    max_cells: usize,
}

/// Builds a CoordinateDir
/// This is where most of the logic is stored for assembling the directory
impl CoordinateDirBuilder {
    /// Start here
    pub fn new() -> Self {
        Self {
            cell_radius: 1.0,
            num_layers: 1,
            first_num_radial_lines: 6,
            second_num_concentric_circles: 2,
            max_cells: 64 * 64,
        }
    }
    /// The radius of each cell in the circle
    pub fn cell_radius(mut self, cell_radius: f32) -> Self {
        self.cell_radius = cell_radius;
        self
    }
    /// The number of layers in the circle
    pub fn num_layers(mut self, num_layers: usize) -> Self {
        self.num_layers = num_layers;
        self
    }
    /// The number of radial lines in the core.
    /// Each future layer has 2x the number of radial lines as the previous layer.
    pub fn first_num_radial_lines(mut self, first_num_radial_lines: usize) -> Self {
        self.first_num_radial_lines = first_num_radial_lines;
        self
    }
    /// The number of concentric circles in the second layer.
    /// Each future layer has 2x the number of concentric circles as the previous layer.
    /// The reason we define the second layer separately is because the core always has 1
    pub fn second_num_concentric_circles(mut self, second_num_concentric_circles: usize) -> Self {
        // debug_assert!(
        //     second_num_concentric_circles % 3 == 0,
        //     "second_num_concentric_circles must be a multiple of 3, got {}",
        //     second_num_concentric_circles
        // );
        self.second_num_concentric_circles = second_num_concentric_circles;
        self
    }
    /// The max number of cells a chunk is allowed to have
    /// If this number is reached, chunks split in half vertically and horizontally
    pub fn max_cells(mut self, max_cells: usize) -> Self {
        self.max_cells = max_cells;
        self
    }
    /// builds a CoordinateDir by iterating over the number of layers
    /// and dynamically allocating chunks to each layer based on max_cells
    /// and the other parameters of the builder.
    pub fn build(self) -> CoordinateDir {
        debug_assert_ne!(self.num_layers, 0);
        let core_chunk = CoreChunkCoords::new(self.cell_radius, self.first_num_radial_lines);
        let mut partial_chunks: Vec<Grid<PartialLayerChunkCoords>> = Vec::new();

        // These variables will help us keep track of the current layer
        let mut layer_num_radial_lines = self.first_num_radial_lines * 2;
        let mut num_concentric_circles = self.second_num_concentric_circles;
        let mut start_concentric_circle_absolute = 1;
        let mut layer_num = 1;
        let mut total_concentric_circle_chunks = 1;

        // Handle the first few layers
        loop {
            if layer_num >= self.num_layers {
                break;
            }
            let next_layer = PartialLayerChunkCoordsBuilder::new()
                .cell_radius(self.cell_radius)
                .layer_num_radial_lines(layer_num_radial_lines)
                .num_concentric_circles(num_concentric_circles)
                .start_concentric_circle_absolute(start_concentric_circle_absolute)
                .start_concentric_circle_layer_relative(0)
                .start_radial_line(0)
                .end_radial_line(layer_num_radial_lines)
                .chunk_idx(ChunkIjkVector {
                    i: layer_num,
                    j: 0,
                    k: 0,
                })
                .build();
            debug_assert!(next_layer.total_size() <= self.max_cells);
            partial_chunks.push(Grid::new(1, 1, vec![next_layer]));

            // Modify the variables
            start_concentric_circle_absolute += num_concentric_circles;
            layer_num_radial_lines *= 2;
            num_concentric_circles *= 2;
            layer_num += 1;
            total_concentric_circle_chunks += 1;

            // At self.max_cells, break
            if (layer_num_radial_lines * num_concentric_circles) > self.max_cells {
                break;
            }
        }

        // Handle the second set of layers, which just subdivide around the grid
        let mut num_radial_chunks = 6;
        loop {
            if layer_num >= self.num_layers {
                break;
            }

            // TODO: Check this
            let mut layer_partial_chunks = Vec::with_capacity(num_radial_chunks);
            for k in 0..num_radial_chunks {
                let next_layer = PartialLayerChunkCoordsBuilder::new()
                    .cell_radius(self.cell_radius)
                    .layer_num_radial_lines(layer_num_radial_lines)
                    .chunk_idx(ChunkIjkVector {
                        i: layer_num,
                        j: 0,
                        k,
                    })
                    .num_concentric_circles(num_concentric_circles)
                    .start_concentric_circle_absolute(start_concentric_circle_absolute)
                    .start_concentric_circle_layer_relative(0)
                    .start_radial_line(k * (layer_num_radial_lines / num_radial_chunks))
                    .end_radial_line((k + 1) * (layer_num_radial_lines / num_radial_chunks))
                    .build();
                debug_assert!(next_layer.total_size() <= self.max_cells);
                layer_partial_chunks.push(next_layer);
            }
            partial_chunks.push(Grid::new(num_radial_chunks, 1, layer_partial_chunks));

            // Modify the variables
            start_concentric_circle_absolute += num_concentric_circles;
            layer_num_radial_lines *= 2;
            num_concentric_circles *= 2;
            layer_num += 1;
            total_concentric_circle_chunks += 1;

            // If our width would become smaller than our height, break
            if layer_num_radial_lines / (num_radial_chunks * 2) < num_concentric_circles {
                break;
            }

            // At self.max_cells, multiply the number of radial chunks by 4
            if (layer_num_radial_lines / num_radial_chunks * num_concentric_circles)
                > self.max_cells
            {
                num_radial_chunks *= 2;
            }
        }

        // Handle the third set of layers, which just subdivide both around the grid and up/down the grid
        let mut num_concentric_chunks = 3;
        // num_radial_chunks *= 2;
        loop {
            if layer_num >= self.num_layers {
                break;
            }

            // TODO: Check this
            let mut layer_partial_chunks =
                Grid::new_empty(num_radial_chunks, num_concentric_chunks);
            for j in 0..num_concentric_chunks {
                for k in 0..num_radial_chunks {
                    let next_layer = PartialLayerChunkCoordsBuilder::new()
                        .cell_radius(self.cell_radius)
                        .layer_num_radial_lines(layer_num_radial_lines)
                        .chunk_idx(ChunkIjkVector { i: layer_num, j, k })
                        .num_concentric_circles(num_concentric_circles / num_concentric_chunks)
                        .start_concentric_circle_absolute(start_concentric_circle_absolute)
                        .start_concentric_circle_layer_relative(
                            j * (num_concentric_circles / num_concentric_chunks),
                        )
                        .start_radial_line(k * (layer_num_radial_lines / num_radial_chunks))
                        .end_radial_line((k + 1) * (layer_num_radial_lines / num_radial_chunks))
                        .build();
                    debug_assert!(next_layer.total_size() <= self.max_cells);
                    layer_partial_chunks.replace(JkVector { j, k }, next_layer);
                }
            }
            partial_chunks.push(layer_partial_chunks);

            // Modify the variables
            start_concentric_circle_absolute += num_concentric_circles;
            layer_num_radial_lines *= 2;
            num_concentric_circles *= 2;
            layer_num += 1;
            total_concentric_circle_chunks += num_concentric_chunks;

            // At self.max_cells, multiply the number of concentric chunks and radial chunks by 2
            if (layer_num_radial_lines / num_radial_chunks * num_concentric_circles
                / num_concentric_chunks)
                > self.max_cells
            {
                num_radial_chunks *= 2;
                num_concentric_chunks *= 2;
            }
        }

        debug_assert!(total_concentric_circle_chunks % 3 == 0, "For multithreading purposes, the total number of concentric circle chunks must be a multiple of 3, got {}", total_concentric_circle_chunks);

        CoordinateDir {
            second_num_concentric_circles: self.second_num_concentric_circles,
            core_chunk,
            partial_chunks,
        }
    }
}

/* =========================================
 *           Aggregate Getters
 * These functions run a getter over each
 * chunk and return a vector of the results
 * ========================================= */
impl CoordinateDir {
    pub fn get_outlines(&self) -> Vec<Grid<Vec<Vec2>>> {
        let mut outlines = Vec::new();
        outlines.push(Grid::new(1, 1, vec![self.core_chunk.get_outline()]));
        for layer in &self.partial_chunks {
            let new_grid = Grid::new(
                layer.get_width(),
                layer.get_height(),
                layer
                    .get_data()
                    .iter()
                    .map(|partial_chunk| partial_chunk.get_outline())
                    .collect(),
            );
            outlines.push(new_grid);
        }
        outlines
    }
    pub fn get_vertexes(&self) -> Vec<Grid<Vec<Vertex>>> {
        let mut vertexes = Vec::new();
        vertexes.push(Grid::new(1, 1, vec![self.core_chunk.get_vertices()]));
        for layer in &self.partial_chunks {
            let new_grid = Grid::new(
                layer.get_width(),
                layer.get_height(),
                layer
                    .get_data()
                    .iter()
                    .map(|partial_chunk| partial_chunk.get_vertices())
                    .collect(),
            );
            vertexes.push(new_grid);
        }
        vertexes
    }

    pub fn get_positions(&self) -> Vec<Grid<Vec<Vec2>>> {
        let mut positions = Vec::new();
        positions.push(Grid::new(1, 1, vec![self.core_chunk.get_positions()]));
        for layer in &self.partial_chunks {
            let new_grid = Grid::new(
                layer.get_width(),
                layer.get_height(),
                layer
                    .get_data()
                    .iter()
                    .map(|partial_chunk| partial_chunk.get_positions())
                    .collect(),
            );
            positions.push(new_grid);
        }
        positions
    }

    pub fn get_uvs(&self) -> Vec<Grid<Vec<Vec2>>> {
        let mut uvs = Vec::new();
        uvs.push(Grid::new(1, 1, vec![self.core_chunk.get_uvs()]));
        for layer in &self.partial_chunks {
            let new_grid = Grid::new(
                layer.get_width(),
                layer.get_height(),
                layer
                    .get_data()
                    .iter()
                    .map(|partial_chunk| partial_chunk.get_uvs())
                    .collect(),
            );
            uvs.push(new_grid);
        }
        uvs
    }

    pub fn get_indices(&self) -> Vec<Grid<Vec<u32>>> {
        let mut indices = Vec::new();
        indices.push(Grid::new(1, 1, vec![self.core_chunk.get_indices()]));
        for layer in &self.partial_chunks {
            let new_grid = Grid::new(
                layer.get_width(),
                layer.get_height(),
                layer
                    .get_data()
                    .iter()
                    .map(|partial_chunk| partial_chunk.get_indices())
                    .collect(),
            );
            indices.push(new_grid);
        }
        indices
    }

    pub fn get_chunk_bounding_boxes(&self) -> Vec<Grid<Rect>> {
        let mut bounding_boxes = Vec::new();
        bounding_boxes.push(Grid::new(1, 1, vec![self.core_chunk.get_bounding_box()]));
        for layer in &self.partial_chunks {
            let new_grid = Grid::new(
                layer.get_width(),
                layer.get_height(),
                layer
                    .get_data()
                    .iter()
                    .map(|partial_chunk| partial_chunk.get_bounding_box())
                    .collect(),
            );
            bounding_boxes.push(new_grid);
        }
        bounding_boxes
    }
}

/* =========================================
 *         Individual Chunk Getters
 * These functions run a getter on a specific
 * chunk index
 * ========================================= */
impl CoordinateDir {
    pub fn get_chunk_at_idx(&self, chunk_idx: ChunkIjkVector) -> Box<dyn ChunkCoords> {
        if chunk_idx.i == 0 {
            Box::new(self.core_chunk)
        } else {
            Box::new(*self.partial_chunks[chunk_idx.i - 1].get(chunk_idx.to_jk_vector()))
        }
    }
    pub fn get_chunk_bounding_box(&self, chunk_idx: ChunkIjkVector) -> Rect {
        if chunk_idx.i == 0 {
            self.core_chunk.get_bounding_box()
        } else {
            self.partial_chunks[chunk_idx.i - 1]
                .get(chunk_idx.to_jk_vector())
                .get_bounding_box()
        }
    }
    pub fn get_chunk_start_radius(&self, chunk_idx: ChunkIjkVector) -> f32 {
        if chunk_idx.i == 0 {
            self.core_chunk.get_start_radius()
        } else {
            self.partial_chunks[chunk_idx.i - 1]
                .get(chunk_idx.to_jk_vector())
                .get_start_radius()
        }
    }
    pub fn get_chunk_end_radius(&self, chunk_idx: ChunkIjkVector) -> f32 {
        if chunk_idx.i == 0 {
            self.core_chunk.get_end_radius()
        } else {
            self.partial_chunks[chunk_idx.i - 1]
                .get(chunk_idx.to_jk_vector())
                .get_end_radius()
        }
    }
    pub fn get_chunk_start_radial_theta(&self, chunk_idx: ChunkIjkVector) -> f32 {
        if chunk_idx.i == 0 {
            self.core_chunk.get_start_radial_theta()
        } else {
            self.partial_chunks[chunk_idx.i - 1]
                .get(chunk_idx.to_jk_vector())
                .get_start_radial_theta()
        }
    }
    pub fn get_chunk_end_radial_theta(&self, chunk_idx: ChunkIjkVector) -> f32 {
        if chunk_idx.i == 0 {
            self.core_chunk.get_end_radial_theta()
        } else {
            self.partial_chunks[chunk_idx.i - 1]
                .get(chunk_idx.to_jk_vector())
                .get_end_radial_theta()
        }
    }
    pub fn get_chunk_num_radial_lines(&self, chunk_idx: ChunkIjkVector) -> usize {
        if chunk_idx.i == 0 {
            self.core_chunk.get_num_radial_lines()
        } else {
            self.partial_chunks[chunk_idx.i - 1]
                .get(chunk_idx.to_jk_vector())
                .get_num_radial_lines()
        }
    }
    pub fn get_chunk_num_concentric_circles(&self, chunk_idx: ChunkIjkVector) -> usize {
        if chunk_idx.i == 0 {
            self.core_chunk.get_num_concentric_circles()
        } else {
            self.partial_chunks[chunk_idx.i - 1]
                .get(chunk_idx.to_jk_vector())
                .get_num_concentric_circles()
        }
    }
}

/* ============================
 * Shape Conversion Functions
 * ============================ */
impl CoordinateDir {
    /// Returns: (layer_num, relative_concentric_circle)
    pub fn convert_absolute_concentric_circle_to_relative(
        &self,
        concentric_circle: usize,
    ) -> (usize, usize) {
        if concentric_circle == 0 {
            (0, 0)
        } else {
            let layer_num = 1;
            loop {
                let start_concentric_circle_abs =
                    self.get_layer_start_concentric_circle_absolute(layer_num);
                if concentric_circle
                    < self.get_layer_num_concentric_circles(layer_num) + start_concentric_circle_abs
                {
                    return (layer_num, concentric_circle - start_concentric_circle_abs);
                }
            }
        }
    }
}

/* =================
 * Layer Getters
 * Get calculated attributes about a layer
 * ================= */
impl CoordinateDir {
    /// The first concentric circle (absolute) index of a given layer
    pub fn get_layer_start_concentric_circle_absolute(&self, layer_num: usize) -> usize {
        if layer_num == 0 {
            self.core_chunk.get_start_concentric_circle_absolute()
        } else {
            self.partial_chunks[layer_num]
                .get(JkVector::ZERO)
                .get_start_concentric_circle_absolute()
        }
    }
}

/* =================
 * Chunk Layer Getters
 * These differ from layer getters in that they are attributes of a "chunk layer"
 * Which is a Grid of chunks in the partial_chunks vector
 * ================== */
impl CoordinateDir {
    /// Get the number of chunks around the circle in a given layer
    pub fn get_layer_num_radial_chunks(&self, layer_num: usize) -> usize {
        if layer_num == 0 {
            1
        } else {
            self.partial_chunks[layer_num - 1].get_width()
        }
    }
    /// Get the number of chunks in the concentric circle dimension in a given layer
    pub fn get_layer_num_concentric_chunks(&self, layer_num: usize) -> usize {
        if layer_num == 0 {
            1
        } else {
            self.partial_chunks[layer_num - 1].get_height()
        }
    }
    /// Gets the total number of chunks you would encounter if you counted
    /// from the core up to the top layer in one dimension
    pub fn get_total_number_concentric_chunks(&self) -> usize {
        let mut total = 1;
        for layer in &self.partial_chunks {
            total += layer.get_height();
        }
        total
    }

    /// Useful if you want to count in chunk concentric circles and get a layer number
    /// In this case the j is the concentric circle index
    /// Returns (layer_num, chunk_layer_concentric_circle)
    pub fn get_layer_and_chunk_num_from_absolute_concentric_chunk(
        &self,
        j_chunk: usize,
    ) -> Result<(usize, usize), String> {
        if j_chunk == 0 {
            return Ok((0, 0));
        }
        let mut total_concentric_chunks = 1;
        for layer_num in 1..self.get_num_layers() {
            let layer_num_concentric_chunks = self.get_layer_num_concentric_chunks(layer_num);
            if j_chunk < total_concentric_chunks + layer_num_concentric_chunks {
                return Ok((layer_num, j_chunk - total_concentric_chunks));
            }
            total_concentric_chunks += layer_num_concentric_chunks;
        }
        Err("j is out of bounds".to_owned())
    }
}
/* ========================================
 * Simple Getters
 * Misc attributes of the directory itself.
 * ======================================== */
impl CoordinateDir {
    /// The total number of cells in the whole directory
    pub fn total_size(&self) -> usize {
        let mut total_size = self.core_chunk.total_size();
        for partial_chunk in &self.partial_chunks {
            total_size += partial_chunk.total_size();
        }
        total_size
    }
    /// Cell radius is constant for all chunks
    pub fn get_cell_radius(&self) -> f32 {
        self.core_chunk.get_cell_radius()
    }
    /// The number of layers in the circle
    pub fn get_num_layers(&self) -> usize {
        self.partial_chunks.len() + 1
    }
    /// Get the core chunk coordinates, useful for getting its shape
    pub fn get_core_chunk(&self) -> &CoreChunkCoords {
        &self.core_chunk
    }
    /// Useful for getting all the partial chunks, useful for getting their shapes
    pub fn get_partial_chunks(&self, _layer_num: usize) -> &Vec<Grid<PartialLayerChunkCoords>> {
        &self.partial_chunks
    }
    /// The number of concentric circles in a given layer
    /// Always 2x the previous layer except for the first and second layers
    pub fn get_layer_num_concentric_circles(&self, layer_num: usize) -> usize {
        if layer_num == 0 {
            self.core_chunk.get_num_concentric_circles()
        } else {
            let mut total_height = 0;
            for j in 0..self.partial_chunks[layer_num - 1].get_height() {
                total_height += self.partial_chunks[layer_num - 1]
                    .get(JkVector { j, k: 0 })
                    .get_num_concentric_circles();
            }
            total_height
        }
    }
    /// The number of radial lines in a given layer
    /// Always 2x the previous layer except for the first layer
    pub fn get_layer_num_radial_lines(&self, layer_num: usize) -> usize {
        if layer_num == 0 {
            self.core_chunk.get_num_radial_lines()
        } else {
            let mut total_width = 0;
            for k in 0..self.partial_chunks[layer_num - 1].get_width() {
                total_width += self.partial_chunks[layer_num - 1]
                    .get(JkVector { j: 0, k })
                    .get_num_radial_lines();
            }
            total_width
        }
    }
    /// The total number of chunks in the directory
    pub fn get_num_chunks(&self) -> usize {
        let mut out = 1;
        for layer in &self.partial_chunks {
            out += layer.get_width() * layer.get_height();
        }
        out
    }
}

/* ===================
 * Drawing
 * =================== */
impl CoordinateDir {
    /// Gets mesh data for every chunk in the directory
    pub fn get_mesh_data(&self, draw_mode: MeshDrawMode) -> Vec<Grid<OwnedMeshData>> {
        let mut out = Vec::with_capacity(self.get_num_chunks());

        // Get the data for core_chunk
        let core_data = match draw_mode {
            MeshDrawMode::TexturedMesh => self.core_chunk.calc_chunk_meshdata(),
            MeshDrawMode::UVWireframe => self.core_chunk.calc_chunk_uv_wireframe(),
            MeshDrawMode::TriangleWireframe => self.core_chunk.calc_chunk_triangle_wireframe(),
            MeshDrawMode::Outline => self.core_chunk.calc_chunk_outline(),
        };
        out.push(Grid::new(1, 1, vec![core_data]));

        // Get the data for partial_chunks
        for layer in &self.partial_chunks {
            let new_grid = Grid::new(
                layer.get_width(),
                layer.get_height(),
                layer
                    .get_data()
                    .iter()
                    .map(|chunk| match draw_mode {
                        MeshDrawMode::TexturedMesh => chunk.calc_chunk_meshdata(),
                        MeshDrawMode::UVWireframe => chunk.calc_chunk_uv_wireframe(),
                        MeshDrawMode::TriangleWireframe => chunk.calc_chunk_triangle_wireframe(),
                        MeshDrawMode::Outline => chunk.calc_chunk_outline(),
                    })
                    .collect(),
            );
            out.push(new_grid);
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_approx_eq {
        ($a:expr, $b:expr) => {
            assert_approx_eq!($a, $b, 0.1);
        };
        ($a:expr, $b:expr, $epsilon:expr) => {
            assert!(
                ($a - $b).abs() < $epsilon,
                "assertion failed: `(left approx== right)`\n  left: `{}`,\n right: `{}`",
                $a,
                $b
            );
        };
    }

    /// Needed these when I noticed get_layer_num_from_absolute_chunk_concentric_circle was wrong
    mod test_concentric_circles_conversions {
        use super::*;

        fn default_coordinate_dir() -> CoordinateDir {
            CoordinateDirBuilder::new()
                .cell_radius(1.0)
                .num_layers(9)
                .first_num_radial_lines(6)
                .second_num_concentric_circles(3)
                .max_cells(64 * 64)
                .build();
        }

        /// Going to verify the chunk grid sizes before we start testing, and so we can know if they change
        #[test]
        fn test_grid_sizes() {
            let coord_dir = default_coordinate_dir();
            // Core
            assert_eq!(coord_dir.get_layer_num_concentric_chunks(0), 1);
            assert_eq!(coord_dir.get_layer_num_radial_chunks(0), 1);

            // Layer 1
            assert_eq!(coord_dir.get_layer_num_concentric_chunks(1), 1);
            assert_eq!(coord_dir.get_layer_num_radial_chunks(1), 1);

            // Layer 2
            assert_eq!(coord_dir.get_layer_num_concentric_chunks(2), 1);
            assert_eq!(coord_dir.get_layer_num_radial_chunks(2), 1);

            // Layer 3
            assert_eq!(coord_dir.get_layer_num_concentric_chunks(3), 1);
            assert_eq!(coord_dir.get_layer_num_radial_chunks(3), 1);

            // Layer 4
            assert_eq!(coord_dir.get_layer_num_concentric_chunks(4), 1);
            assert_eq!(coord_dir.get_layer_num_radial_chunks(4), 1);

            // Layer 5
            assert_eq!(coord_dir.get_layer_num_concentric_chunks(5), 1);
            assert_eq!(coord_dir.get_layer_num_radial_chunks(5), 6);

            // Layer 6
            assert_eq!(coord_dir.get_layer_num_concentric_chunks(6), 3);
            assert_eq!(coord_dir.get_layer_num_radial_chunks(6), 6);

            // Layer 7
            assert_eq!(coord_dir.get_layer_num_concentric_chunks(7), 6);
            assert_eq!(coord_dir.get_layer_num_radial_chunks(7), 12);

            // Layer 8
            assert_eq!(coord_dir.get_layer_num_concentric_chunks(8), 12);
            assert_eq!(coord_dir.get_layer_num_radial_chunks(8), 24);
        }

        #[test]
        fn test_get_total_number_chunks_in_concentric_circle_dimension() {
            let coord_dir = default_coordinate_dir();
            assert_eq!(coord_dir.get_total_number_concentric_chunks(), 27);
        }

        #[test]
        fn test_get_layer_num_from_absolute_chunk_concentric_circle() {
            let coord_dir = default_coordinate_dir();
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(0)
                    .unwrap(),
                (0, 0)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(1)
                    .unwrap(),
                (1, 0)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(2)
                    .unwrap(),
                (2, 0)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(3)
                    .unwrap(),
                (3, 0)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(4)
                    .unwrap(),
                (4, 0)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(5)
                    .unwrap(),
                (5, 0)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(6)
                    .unwrap(),
                (6, 0)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(7)
                    .unwrap(),
                (6, 1)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(8)
                    .unwrap(),
                (6, 2)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(9)
                    .unwrap(),
                (7, 0)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(10)
                    .unwrap(),
                (7, 1)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(11)
                    .unwrap(),
                (7, 2)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(12)
                    .unwrap(),
                (7, 3)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(13)
                    .unwrap(),
                (7, 4)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(14)
                    .unwrap(),
                (7, 5)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(15)
                    .unwrap(),
                (8, 0)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(16)
                    .unwrap(),
                (8, 1)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(17)
                    .unwrap(),
                (8, 2)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(18)
                    .unwrap(),
                (8, 3)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(19)
                    .unwrap(),
                (8, 4)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(20)
                    .unwrap(),
                (8, 5)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(21)
                    .unwrap(),
                (8, 6)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(22)
                    .unwrap(),
                (8, 7)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(23)
                    .unwrap(),
                (8, 8)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(24)
                    .unwrap(),
                (8, 9)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(25)
                    .unwrap(),
                (8, 10)
            );
            assert_eq!(
                coord_dir
                    .get_layer_and_chunk_num_from_absolute_concentric_chunk(26)
                    .unwrap(),
                (8, 11)
            );
        }
    }

    #[test]
    fn test_radial_mesh_chunk_sizes_manual() {
        let coordinate_dir = CoordinateDirBuilder::new()
            .cell_radius(1.0)
            .num_layers(8)
            .first_num_radial_lines(8)
            .second_num_concentric_circles(2)
            .max_cells(64 * 64) // 24x24
            .build();

        // Layer 0
        // Test that the first chunk is 1x8
        assert_eq!(
            coordinate_dir.get_chunk_num_radial_lines(ChunkIjkVector::ZERO),
            8
        );
        assert_eq!(
            coordinate_dir.get_chunk_num_concentric_circles(ChunkIjkVector::ZERO),
            1
        );
        // The start_radius x end_radius x start_radial_theta x end_radial_theta should be 0 x 1 x 0 x 2pi
        assert_eq!(
            coordinate_dir.get_chunk_start_radius(ChunkIjkVector::ZERO),
            0.0
        );
        assert_eq!(
            coordinate_dir.get_chunk_end_radius(ChunkIjkVector::ZERO),
            1.0
        );
        assert_eq!(
            coordinate_dir.get_chunk_start_radial_theta(ChunkIjkVector::ZERO),
            0.0
        );
        assert_eq!(
            coordinate_dir.get_chunk_end_radial_theta(ChunkIjkVector::ZERO),
            2.0 * std::f32::consts::PI
        );

        // Layer 1
        let layer1 = ChunkIjkVector { i: 1, j: 0, k: 0 };
        // Test that the next chunk is 2x16
        assert_eq!(coordinate_dir.get_chunk_num_radial_lines(layer1), 16);
        assert_eq!(coordinate_dir.get_chunk_num_concentric_circles(layer1), 2);
        // The start_radius x end_radius x start_radial_theta x end_radial_theta should be 1 x 3 x 0 x 2pi
        // 1 comes from the previous layer's end_radius
        // 3 comes from the previous layer's end_radius + the previous layers (end_radius - start_radius)*2
        assert_eq!(coordinate_dir.get_chunk_start_radius(layer1), 1.0);
        assert_eq!(coordinate_dir.get_chunk_end_radius(layer1), 3.0);
        assert_eq!(coordinate_dir.get_chunk_start_radial_theta(layer1), 0.0);
        assert_eq!(
            coordinate_dir.get_chunk_end_radial_theta(layer1),
            2.0 * std::f32::consts::PI
        );

        // Layer 2
        let layer2 = ChunkIjkVector { i: 2, j: 0, k: 0 };
        // Test that the next chunk is 4x32
        assert_eq!(coordinate_dir.get_chunk_num_radial_lines(layer2), 32);
        assert_eq!(coordinate_dir.get_chunk_num_concentric_circles(layer2), 4);
        // The start_radius x end_radius x start_radial_theta x end_radial_theta should be 3 x 7 x 0 x 2pi
        assert_eq!(coordinate_dir.get_chunk_start_radius(layer2), 3.0);
        assert_eq!(coordinate_dir.get_chunk_end_radius(layer2), 7.0);
        assert_eq!(coordinate_dir.get_chunk_start_radial_theta(layer2), 0.0);
        assert_eq!(
            coordinate_dir.get_chunk_end_radial_theta(layer2),
            2.0 * std::f32::consts::PI
        );

        // Layer 3
        let layer3 = ChunkIjkVector { i: 3, j: 0, k: 0 };
        // Test that the next chunk is 8x64
        assert_eq!(coordinate_dir.get_chunk_num_radial_lines(layer3), 64);
        assert_eq!(coordinate_dir.get_chunk_num_concentric_circles(layer3), 8);
        // The start_radius x end_radius x start_radial_theta x end_radial_theta should be 7 x 15 x 0 x 2pi
        assert_eq!(coordinate_dir.get_chunk_start_radius(layer3), 7.0);
        assert_eq!(coordinate_dir.get_chunk_end_radius(layer3), 15.0);
        assert_eq!(coordinate_dir.get_chunk_start_radial_theta(layer3), 0.0);
        assert_eq!(
            coordinate_dir.get_chunk_end_radial_theta(layer3),
            2.0 * std::f32::consts::PI
        );

        // Layer 4
        let layer4 = ChunkIjkVector { i: 4, j: 0, k: 0 };
        //  Test that the next chunk is 16x128
        assert_eq!(coordinate_dir.get_chunk_num_radial_lines(layer4), 128);
        assert_eq!(coordinate_dir.get_chunk_num_concentric_circles(layer4), 16);
        // The start_radius x end_radius x start_radial_theta x end_radial_theta should be 15 x 31 x 0 x 2pi
        assert_eq!(coordinate_dir.get_chunk_start_radius(layer4), 15.0);
        assert_eq!(coordinate_dir.get_chunk_end_radius(layer4), 31.0);
        assert_eq!(coordinate_dir.get_chunk_start_radial_theta(layer4), 0.0);
        assert_eq!(
            coordinate_dir.get_chunk_end_radial_theta(layer4),
            2.0 * std::f32::consts::PI
        );

        // Layer 5
        let layer5 = ChunkIjkVector { i: 5, j: 0, k: 0 };
        // Test that the next chunk is 32x256
        // This divided in to 6
        assert_eq!(coordinate_dir.get_chunk_num_radial_lines(layer5), 256 / 6);
        assert_eq!(coordinate_dir.get_chunk_num_concentric_circles(layer5), 32);
        // The start_radius x end_radius x start_radial_theta x end_radial_theta should be 31 x 63 x 0 x 2pi / 6
        assert_eq!(coordinate_dir.get_chunk_start_radius(layer5), 31.0);
        assert_eq!(coordinate_dir.get_chunk_end_radius(layer5), 63.0);
        assert_eq!(coordinate_dir.get_chunk_start_radial_theta(layer5), 0.0);
        assert_approx_eq!(
            coordinate_dir.get_chunk_end_radial_theta(layer5),
            2.0 * std::f32::consts::PI / 6.0
        );

        // Layer 6
        let layer6 = ChunkIjkVector { i: 6, j: 0, k: 0 };
        // Test that the next chunk is 64x512
        // This is divided radially in 6
        assert_eq!(
            coordinate_dir.get_chunk_num_radial_lines(ChunkIjkVector { i: 6, j: 0, k: 0 }),
            512 / 6
        );
        // And concentrically by 3
        assert_eq!(
            coordinate_dir.get_chunk_num_concentric_circles(ChunkIjkVector { i: 6, j: 0, k: 0 }),
            64 / 3
        );
        assert_eq!(coordinate_dir.get_chunk_start_radius(layer6), 63.0);
        assert_eq!(
            coordinate_dir.get_chunk_end_radius(layer6),
            63.0 + (64 / 3) as f32
        );
        assert_eq!(coordinate_dir.get_chunk_start_radial_theta(layer6), 0.0);
        assert_approx_eq!(
            coordinate_dir.get_chunk_end_radial_theta(layer6),
            2.0 * std::f32::consts::PI / 6.0
        );

        // Layer 7
        let layer7 = ChunkIjkVector { i: 7, j: 0, k: 0 };
        // Test that the next chunk is 128x1024
        // This is divided radially in 12
        assert_eq!(
            coordinate_dir.get_chunk_num_radial_lines(ChunkIjkVector { i: 7, j: 0, k: 0 }),
            1024 / 12
        );
        // And concentrically by 6
        assert_eq!(
            coordinate_dir.get_chunk_num_concentric_circles(ChunkIjkVector { i: 7, j: 0, k: 0 }),
            128 / 6
        );
        assert_eq!(coordinate_dir.get_chunk_start_radius(layer7), 63.0 + 64_f32);
        assert_eq!(
            coordinate_dir.get_chunk_end_radius(layer7),
            63.0 + 64_f32 + (128 / 6) as f32
        );
        assert_eq!(coordinate_dir.get_chunk_start_radial_theta(layer7), 0.0);
        assert_approx_eq!(
            coordinate_dir.get_chunk_end_radial_theta(layer7),
            2.0 * std::f32::consts::PI / 12.0
        );
    }
}
