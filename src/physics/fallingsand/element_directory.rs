use super::coordinates::coordinate_directory::CoordinateDir;
use super::element_grid::ElementGrid;
use super::util::RawImage;

/// An element grid directory is like a coordinate directory, but for element grids
/// It follow the same layer structure
/// There is a coordinate directory at the root, but also each ElementGrid has its own
/// copy of the chunk coordinates associated with it for convenience
pub struct ElementGridDir {
    coords: CoordinateDir,
    chunks: Vec<Box<ElementGrid>>,
}

impl ElementGridDir {
    pub fn new_empty(coords: CoordinateDir) -> Self {
        let mut chunks = Vec::with_capacity(coords.len());
        for i in 0..coords.len() {
            chunks.push(Box::new(ElementGrid::new_empty(coords.get_chunk_at_idx(i))));
        }
        Self {
            coords,
            chunks: chunks,
        }
    }
    pub fn get_coordinate_dir(&self) -> &CoordinateDir {
        &self.coords
    }
    pub fn len(&self) -> usize {
        self.chunks.len()
    }
    pub fn get_textures(&self) -> Vec<RawImage> {
        let mut out = Vec::with_capacity(self.len());
        for chunk in &self.chunks {
            out.push(chunk.get_texture());
        }
        out
    }
}
