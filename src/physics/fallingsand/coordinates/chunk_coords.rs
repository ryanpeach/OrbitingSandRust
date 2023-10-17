use ggez::glam::Vec2;
use ggez::graphics::Color;
use ggez::graphics::MeshBuilder;
use ggez::graphics::Rect;
use ggez::graphics::Vertex;
use std::f32::consts::PI;

use crate::physics::fallingsand::util::mesh::OwnedMeshData;
use crate::physics::fallingsand::util::vectors::ChunkIjkVector;
use crate::physics::fallingsand::util::vectors::IjkVector;
use crate::physics::fallingsand::util::vectors::JkVector;
use crate::physics::util::vectors::RelXyPoint;

/// A set of coordinates that tell you where on the circle a chunk is located
/// and how big it is. Also provides methods for drawing the mesh.
pub trait ChunkCoords: Send + Sync {
    /* Raw Data */
    fn get_outline(&self) -> Vec<Vec2>;
    fn get_positions(&self) -> Vec<Vec2>;
    fn get_indices(&self) -> Vec<u32>;
    fn get_uvs(&self) -> Vec<Vec2>;

    /* Shape Parameter Getters */
    fn get_num_radial_lines(&self) -> usize;
    fn get_num_concentric_circles(&self) -> usize;
    fn total_size(&self) -> usize {
        self.get_num_radial_lines() * self.get_num_concentric_circles()
    }

    /* Position on the Circle */
    fn get_bounding_box(&self) -> Rect;
    fn get_layer_num(&self) -> usize;
    fn get_chunk_idx(&self) -> ChunkIjkVector;
    fn get_cell_radius(&self) -> f32;
    fn get_start_radius(&self) -> f32;
    fn get_end_radius(&self) -> f32;
    /// This gets the theta (degrees around) of the first drawn radial line
    /// On a full layer, this would be 0
    /// Inclusive
    fn get_start_radial_theta(&self) -> f32;
    /// This gets the theta (degrees around) of the last drawn radial line
    /// On a full layer, this would be 2 * PI
    /// Inclusive
    fn get_end_radial_theta(&self) -> f32;
    /// This gets the index of the first drawn concentric circle
    /// layer relative means that its relative to the layer, not the greater circle
    /// On a full layer, this would be 0
    /// Inclusive
    fn get_start_concentric_circle_layer_relative(&self) -> usize;
    /// This gets the index of the last drawn concentric circle
    /// layer relative means that its relative to the layer, not the greater circle
    /// NOT the last drawn concentric cell. This would usually be that + 1
    /// Inclusive
    fn get_end_concentric_circle_layer_relative(&self) -> usize;
    /// This gets the index of the first drawn concentric circle
    /// absolute means that its relative to the greater circle, not the layer
    /// On a full layer, this would be 0
    /// Inclusive
    fn get_start_concentric_circle_absolute(&self) -> usize;
    /// This gets the index of the last drawn concentric circle
    /// absolute means that its relative to the greater circle, not the layer
    /// NOT the last drawn concentric cell. This would usually be that + 1
    /// Inclusive
    fn get_end_concentric_circle_absolute(&self) -> usize;
    /// This gets the index of the last drawn radial line
    /// NOT the last drawn radial cell. This would usually be that + 1
    /// Inclusive
    fn get_end_radial_line(&self) -> usize;
    /// This gets the index of the first drawn radial line
    /// On a full layer, this would be 0
    /// Inclusive
    fn get_start_radial_line(&self) -> usize;

    /* Positions in the chunk */
    /// Checks to see if an absolute position around the circle is in the chunk
    fn contains(&self, idx: IjkVector) -> bool {
        idx.i == self.get_layer_num()
            && idx.j >= self.get_start_radial_line()
            && idx.j < self.get_end_radial_line()
            && idx.k >= self.get_start_concentric_circle_absolute()
            && idx.k < self.get_end_concentric_circle_absolute()
    }
    /// Converts a coordinate from anywhere on the circle, assuming it is in the chunk
    /// to a coordinate inside the grid of this chunk
    fn get_internal_coord_from_external_coord(&self, external_coord: IjkVector) -> JkVector {
        debug_assert!(self.contains(external_coord));
        JkVector {
            j: external_coord.j - self.get_start_radial_line(),
            k: external_coord.k - self.get_start_concentric_circle_absolute(),
        }
    }
    /// Converts a coordinate from inside this chunk to a coordinate on the circle
    fn get_external_coord_from_internal_coord(&self, internal_coord: JkVector) -> IjkVector {
        debug_assert!(internal_coord.j < self.get_num_radial_lines());
        debug_assert!(internal_coord.k < self.get_num_concentric_circles());
        IjkVector {
            i: self.get_layer_num(),
            j: internal_coord.j + self.get_start_radial_line(),
            k: internal_coord.k + self.get_start_concentric_circle_absolute(),
        }
    }

    /* Convienience Functions */
    fn get_vertices(&self) -> Vec<Vertex> {
        let positions = self.get_positions();
        let uvs = self.get_uvs();
        let vertexes: Vec<Vertex> = positions
            .iter()
            .zip(uvs.iter())
            .map(|(p, uv)| Vertex {
                position: [p.x, p.y],
                uv: [uv.x, uv.y],
                color: [1.0, 1.0, 1.0, 1.0],
            })
            .collect();
        vertexes
    }
    fn calc_chunk_outline(&self) -> OwnedMeshData {
        let mut mb = MeshBuilder::new();
        let outline = self.get_outline();
        let _ = mb.line(&outline, 0.1, Color::WHITE);
        let meshdata = mb.build();
        OwnedMeshData {
            vertices: meshdata.vertices.to_owned(),
            indices: meshdata.indices.to_owned(),
            uv_bounds: Rect::new(
                self.get_start_radial_line() as f32,
                self.get_start_concentric_circle_absolute() as f32,
                self.get_end_radial_line() as f32 - self.get_start_radial_line() as f32,
                self.get_end_concentric_circle_absolute() as f32
                    - self.get_start_concentric_circle_absolute() as f32,
            ),
        }
    }
    fn calc_chunk_meshdata(&self) -> OwnedMeshData {
        let indices = self.get_indices();
        let vertices: Vec<Vertex> = self.get_vertices();
        OwnedMeshData {
            vertices,
            indices,
            uv_bounds: Rect::new(
                self.get_start_radial_line() as f32,
                self.get_start_concentric_circle_absolute() as f32,
                self.get_end_radial_line() as f32 - self.get_start_radial_line() as f32,
                self.get_end_concentric_circle_absolute() as f32
                    - self.get_start_concentric_circle_absolute() as f32,
            ),
        }
    }
    fn calc_chunk_triangle_wireframe(&self) -> OwnedMeshData {
        let mut mb = MeshBuilder::new();
        let indices = self.get_indices();
        let vertices: Vec<Vertex> = self.get_vertices();
        for i in (0..indices.len()).step_by(3) {
            let i1: usize = indices[i] as usize;
            let i2 = indices[i + 1] as usize;
            let i3 = indices[i + 2] as usize;

            let p1 = vertices[i1].position;
            let p2 = vertices[i2].position;
            let p3 = vertices[i3].position;

            let _ = mb.line(&[p1, p2, p3, p1], 0.1, Color::WHITE).unwrap();
        }
        let meshdata = mb.build();
        OwnedMeshData {
            vertices: meshdata.vertices.to_owned(),
            indices: meshdata.indices.to_owned(),
            uv_bounds: Rect::new(
                self.get_start_radial_line() as f32,
                self.get_start_concentric_circle_absolute() as f32,
                self.get_end_radial_line() as f32 - self.get_start_radial_line() as f32,
                self.get_end_concentric_circle_absolute() as f32
                    - self.get_start_concentric_circle_absolute() as f32,
            ),
        }
    }
    fn calc_chunk_uv_wireframe(&self) -> OwnedMeshData {
        let mut mb = MeshBuilder::new();
        let indices = self.get_indices();
        let vertices: Vec<Vertex> = self.get_vertices();
        for i in (0..indices.len()).step_by(3) {
            let i1 = indices[i] as usize;
            let i2 = indices[i + 1] as usize;
            let i3 = indices[i + 2] as usize;

            let p1 = vertices[i1].uv;
            let p1_multiplied = Vec2::new(p1[0] * 10.0, p1[1] * 10.0);
            let p2 = vertices[i2].uv;
            let p2_multiplied = Vec2::new(p2[0] * 10.0, p2[1] * 10.0);
            let p3 = vertices[i3].uv;
            let p3_multiplied = Vec2::new(p3[0] * 10.0, p3[1] * 10.0);

            let _ = mb
                .line(
                    &[p1_multiplied, p2_multiplied, p3_multiplied, p1_multiplied],
                    0.1,
                    Color::WHITE,
                )
                .unwrap();
        }
        let meshdata = mb.build();
        OwnedMeshData {
            vertices: meshdata.vertices.to_owned(),
            indices: meshdata.indices.to_owned(),
            uv_bounds: Rect::new(
                self.get_start_radial_line() as f32,
                self.get_start_concentric_circle_absolute() as f32,
                self.get_end_radial_line() as f32 - self.get_start_radial_line() as f32,
                self.get_end_concentric_circle_absolute() as f32
                    - self.get_start_concentric_circle_absolute() as f32,
            ),
        }
    }

    /// Converts a position relative to the origin of the circle to a cell index
    /// Returns an Err if the position is not on the circle
    fn rel_pos_to_cell_idx(&self, xy_coord: RelXyPoint) -> Result<IjkVector, String> {
        let norm_vertex_coord = (xy_coord.0.x * xy_coord.0.x + xy_coord.0.y * xy_coord.0.y).sqrt();
        let start_concentric_circle = self.get_start_concentric_circle_layer_relative();
        let end_concentric_circle = self.get_end_concentric_circle_layer_relative();
        let starting_r = self.get_start_radius();
        let ending_r = self.get_end_radius();
        let num_concentric_circles = self.get_num_concentric_circles();
        let num_radial_lines = self.get_num_radial_lines();
        let start_radial_line = self.get_start_radial_line();
        let end_radial_line = self.get_end_radial_line();
        let start_radial_theta = self.get_start_radial_theta();
        let end_radial_theta = self.get_end_radial_theta();

        // Get the concentric circle we are on
        let circle_separation_distance = (ending_r - starting_r) / num_concentric_circles as f32;

        // Calculate 'j' directly without the while loop
        let j_rel =
            ((norm_vertex_coord - starting_r) / circle_separation_distance).floor() as usize;
        let j = j_rel.min(end_concentric_circle - 1) + start_concentric_circle;

        // Get the radial line to the left of the vertex
        let angle = (xy_coord.0.y.atan2(xy_coord.0.x) + 2.0 * PI) % (2.0 * PI);
        let theta = (end_radial_theta - start_radial_theta) / num_radial_lines as f32;

        // Calculate 'k' directly without the while loop
        let k_rel = (angle / theta).floor() as usize;
        let k = k_rel.min(end_radial_line - 1);

        // Check to see if the vertex is in the chunk
        if j < start_concentric_circle && j >= end_concentric_circle {
            return Err(format!(
                "Vertex j {:?} is not in chunk {:?}. start_concentric_circle: {}, end_concentric_circle: {}",
                xy_coord,
                self.get_chunk_idx(),
                start_concentric_circle,
                end_concentric_circle,
            ));
        }
        if k < start_radial_line && k >= end_radial_line {
            return Err(format!(
                "Vertex k {:?} is not in chunk {:?}. start_radial_line: {}, end_radial_line: {}",
                xy_coord,
                self.get_chunk_idx(),
                start_radial_line,
                end_radial_line,
            ));
        }
        Ok(IjkVector {
            i: self.get_layer_num(),
            j,
            k,
        })
    }

    /// Convert a cell coordinate "on the circle" to a position "on the chunk"
    /// Return an Err if this is not on the chunk
    fn absolute_cell_idx_to_in_chunk_cell_idx(
        &self,
        cell_idx: IjkVector,
    ) -> Result<JkVector, String> {
        if cell_idx.i != self.get_layer_num() {
            return Err(format!(
                "Cell index i {:?} is not in chunk {:?}",
                cell_idx,
                self.get_chunk_idx()
            ));
        }
        let start_radial_line = self.get_start_radial_line();
        let end_radial_line = self.get_end_radial_line();
        let start_concentric_circle = self.get_start_concentric_circle_layer_relative();
        let end_concentric_circle = self.get_end_concentric_circle_layer_relative();
        if cell_idx.j < start_concentric_circle || cell_idx.j >= end_concentric_circle {
            return Err(format!(
                "Cell index j {:?} is not in chunk {:?}. start_concentric_circle: {}, end_concentric_circle: {}",
                cell_idx,
                self.get_chunk_idx(),
                start_concentric_circle,
                end_concentric_circle,
            ));
        }
        if cell_idx.k < start_radial_line || cell_idx.k >= end_radial_line {
            return Err(format!(
                "Cell index k {:?} is not in chunk {:?}. start_radial_line: {}, end_radial_line: {}",
                cell_idx,
                self.get_chunk_idx(),
                start_radial_line,
                end_radial_line,
            ));
        }
        Ok(JkVector {
            j: cell_idx.j - start_concentric_circle,
            k: cell_idx.k - start_radial_line,
        })
    }
}
