use ndarray::Array1;

use crate::physics::fallingsand::{
    convolution::{
        behaviors::ElementGridConvolutionNeighbors,
        neighbor_grids::{BottomNeighborGrids, LeftRightNeighborGrids, TopNeighborGrids},
    },
    data::element_grid::ElementGrid,
    util::vectors::JkVector,
};

/// The output of [BorderTemperatures::get_border_temps]
#[derive(Clone, Debug, Default)]
pub struct ElementGridConvolutionNeighborTemperatures {
    /// The heat of the top neighbor border
    pub top: Option<Array1<f32>>,
    /// The heat of the bottom neighbor border
    pub bottom: Option<Array1<f32>>,
    /// The heat of the left neighbor border
    pub left: Array1<f32>,
    /// The heat of the right neighbor border
    pub right: Array1<f32>,
}

impl ElementGridConvolutionNeighbors {
    /// Get the heat of the neighbors at their border
    /// Keep these images in your mind as you read this code
    /// ![chunk doubling](assets/docs/wireframes/layer_transition.png)
    /// ![wireframe](assets/docs/wireframes/wireframe.png)
    pub fn get_border_temps(
        &self,
        target_chunk: &ElementGrid,
    ) -> ElementGridConvolutionNeighborTemperatures {
        let coords = target_chunk.get_chunk_coords();
        let mut out = ElementGridConvolutionNeighborTemperatures::default();
        let mut this = Array1::zeros(coords.get_num_radial_lines());
        match &self.grids.top {
            TopNeighborGrids::Normal { t, tl, tr: _ } => {
                top_neighbor_grids_normal(t, coords, &mut this, tl, &mut out);
                out.top = Some(this);
            }
            // In this case t1 and t0 are both half the size of target_chunk
            // TODO: Test this
            TopNeighborGrids::ChunkDoubling { t1, t0, .. } => {
                top_neighbor_grids_chunk_doubling(t0, coords, t1, &mut this, &mut out);
                out.top = Some(this);
            }
            TopNeighborGrids::TopOfGrid => {
                out.top = None;
            }
        }
        let mut this = Array1::zeros(coords.get_num_radial_lines());
        match &self.grids.bottom {
            BottomNeighborGrids::Normal { b, .. } => {
                bottom_neighbor_grids_normal(coords, b, &mut this, &mut out);
                out.bottom = Some(this);
            }
            // In this case bl and br are both twice the size of target_chunk
            BottomNeighborGrids::ChunkDoubling { bl, br } => {
                bottom_neighbor_grids_chunk_doubling(coords, target_chunk, bl, br);
                out.bottom = Some(this);
            }
            BottomNeighborGrids::BottomOfGrid => {
                out.bottom = None;
            }
        }
        match &self.grids.left_right {
            LeftRightNeighborGrids::LR { l, r } => {
                left_right_neighbor_grids(l, &mut out, r);
            }
        }
        out
    }
}

fn top_neighbor_grids_normal(
    t: &ElementGrid,
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    this: &mut ndarray::prelude::ArrayBase<
        ndarray::OwnedRepr<f32>,
        ndarray::prelude::Dim<[usize; 1]>,
    >,
    tl: &ElementGrid,
    out: &mut ElementGridConvolutionNeighborTemperatures,
) {
    if t.get_chunk_coords().get_num_radial_lines() == coords.get_num_radial_lines() {
        top_neighbor_grids_normal_no_cell_doubling(coords, t, this);
    } else {
        top_neighbor_grids_normal_cell_doubling(coords, t, this, tl);
    }
}

fn top_neighbor_grids_normal_no_cell_doubling(
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    t: &ElementGrid,
    this: &mut ndarray::prelude::ArrayBase<
        ndarray::OwnedRepr<f32>,
        ndarray::prelude::Dim<[usize; 1]>,
    >,
) {
    for k in 0..coords.get_num_radial_lines() {
        let idx = JkVector { j: 0, k };
        let temp = t.get_temperature(idx);
        this[idx.to_ndarray_coords(coords).x] = temp.0;
    }
}

fn top_neighbor_grids_normal_cell_doubling(
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    t: &ElementGrid,
    this: &mut ndarray::prelude::ArrayBase<
        ndarray::OwnedRepr<f32>,
        ndarray::prelude::Dim<[usize; 1]>,
    >,
    tl: &ElementGrid,
) {
    // In this case we are dealing with a cell doubling tangentially
    // So we will average the two cells
    // TODO: Test this
    for k in 0..coords.get_num_radial_lines() * 2 {
        let their_idx = JkVector { j: 0, k };
        let our_idx = JkVector { j: 0, k: k / 2 };
        // In this case we will put the cell in the memory because
        // it comes first
        if k % 2 == 0 {
            let temp = t.get_temperature(their_idx);
            this[our_idx.to_ndarray_coords(coords).x] = temp.0;
        }
        // In this case we will average it with ourselves
        else {
            let temp = tl.get_temperature(their_idx);
            this[our_idx.to_ndarray_coords(coords).x] =
                (temp.0 + this[our_idx.to_ndarray_coords(coords).x]) / 2.0;
        }
    }
}

fn top_neighbor_grids_chunk_doubling(
    t0: &ElementGrid,
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    t1: &ElementGrid,
    this: &mut ndarray::prelude::ArrayBase<
        ndarray::OwnedRepr<f32>,
        ndarray::prelude::Dim<[usize; 1]>,
    >,
    out: &mut ElementGridConvolutionNeighborTemperatures,
) {
    debug_assert_eq!(
        t0.get_chunk_coords().get_num_radial_lines(),
        coords.get_num_radial_lines(),
        "The number of radial lines should be the same, even though the sizes are different"
    );
    debug_assert_eq!(
        t1.get_chunk_coords().get_num_radial_lines(),
        coords.get_num_radial_lines(),
        "The number of radial lines should be the same, even though the sizes are different"
    );
    // First lets do this from t0, iterating over its bottom border
    // and averaging that into every other one of our cells
    for k in 0..t0.get_chunk_coords().get_num_radial_lines() {
        let their_idx = JkVector { j: 0, k };
        let temp = t0.get_temperature(their_idx);
        let our_idx = JkVector {
            j: coords.get_num_concentric_circles() - 1,
            k: k / 2,
        };
        // In this case we will put the cell in the memory because
        // it comes first
        if k % 2 == 0 {
            this[our_idx.to_ndarray_coords(coords).x] = temp.0;
        }
        // In this case we will average it with ourselves
        else {
            this[our_idx.to_ndarray_coords(coords).x] =
                (temp.0 + this[our_idx.to_ndarray_coords(coords).x]) / 2.0;
        }
    }
    // Now lets do this from t1, iterating over its bottom border
    // and averaging that into every other one of our cells
    // startging from the middle of our radial lines
    for k in 0..t1.get_chunk_coords().get_num_radial_lines() {
        let their_idx = JkVector { j: 0, k };
        let temp = t1.get_temperature(their_idx);
        let our_idx = JkVector {
            j: coords.get_num_concentric_circles() - 1,
            k: k / 2 + coords.get_num_radial_lines() / 2,
        };
        // In this case we will put the cell in the memory because
        // it comes first
        if k % 2 == 0 {
            this[our_idx.to_ndarray_coords(coords).x] = temp.0;
        }
        // In this case we will average it with ourselves
        else {
            this[our_idx.to_ndarray_coords(coords).x] =
                (temp.0 + this[our_idx.to_ndarray_coords(coords).x]) / 2.0;
        }
    }
}

fn bottom_neighbor_grids_normal(
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    b: &ElementGrid,
    this: &mut ndarray::prelude::ArrayBase<
        ndarray::OwnedRepr<f32>,
        ndarray::prelude::Dim<[usize; 1]>,
    >,
    out: &mut ElementGridConvolutionNeighborTemperatures,
) {
    if coords.get_num_radial_lines() == b.get_chunk_coords().get_num_radial_lines() {
        bottom_neighbor_grids_normal_no_cell_doubling(coords, b, this);
    } else {
        bottom_neighbor_grids_normal_cell_doubling(coords, b, this, out);
    }
}

fn bottom_neighbor_grids_normal_cell_doubling(
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    b: &ElementGrid,
    this: &mut ndarray::prelude::ArrayBase<
        ndarray::OwnedRepr<f32>,
        ndarray::prelude::Dim<[usize; 1]>,
    >,
    out: &mut ElementGridConvolutionNeighborTemperatures,
) {
    // In this case we are dealing with a cell halving tangentially
    // So we put the same cell in the memory twice
    for k in 0..coords.get_num_radial_lines() {
        let our_idx = JkVector {
            j: coords.get_num_concentric_circles() - 1,
            k,
        };
        let their_idx = JkVector {
            j: b.get_chunk_coords().get_num_concentric_circles() - 1,
            k: k / 2,
        };
        let temp = b.get_temperature(their_idx);
        this[our_idx.to_ndarray_coords(coords).x] = temp.0;
    }
}

fn bottom_neighbor_grids_normal_no_cell_doubling(
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    b: &ElementGrid,
    this: &mut ndarray::prelude::ArrayBase<
        ndarray::OwnedRepr<f32>,
        ndarray::prelude::Dim<[usize; 1]>,
    >,
) {
    for k in 0..coords.get_num_radial_lines() {
        let idx = JkVector {
            j: coords.get_num_concentric_circles() - 1,
            k,
        };
        let temp = b.get_temperature(idx);
        this[idx.to_ndarray_coords(coords).x] = temp.0;
    }
}

fn bottom_neighbor_grids_chunk_doubling(
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    target_chunk: &ElementGrid,
    bl: &ElementGrid,
    br: &ElementGrid,
) {
    // TODO: document this with pictures
    // TODO: Unit test
    let mut this = Array1::zeros(coords.get_num_radial_lines());
    // This is the case where the bottom neighbor is the bl chunk
    // And we are straddling the right side of the bl chunk
    if target_chunk.get_chunk_coords().get_chunk_idx().k % 2 == 0 {
        debug_assert_eq!(
            bl.get_chunk_coords().get_num_radial_lines(),
            coords.get_num_radial_lines(),
            "The number of radial lines should be the same, even though the sizes are different"
        );
        // We are going to iterate over half its border
        // starting at our k=0 and ending at our k=coords.get_num_radial_lines()
        // but from its perspective starting at k=0 (because we are on its right side)
        // and ending at k=bl.get_chunk_coords().get_num_radial_lines()/2
        // This means we are putting the same cell in the memory twice
        for k in 0..coords.get_num_radial_lines() {
            let our_idx = JkVector { j: 0, k };
            let their_idx = JkVector {
                j: bl.get_chunk_coords().get_num_concentric_circles() - 1,
                k: k / 2,
            };
            let temp = bl.get_temperature(their_idx);
            this[our_idx.to_ndarray_coords(coords).x] = temp.0;
        }
    }
    // This is the case where the bottom neighbor is the br chunk
    // And we are straddling the left side of the br chunk
    else {
        debug_assert_eq!(
            br.get_chunk_coords().get_num_radial_lines(),
            coords.get_num_radial_lines(),
            "The number of radial lines should be the same, even though the sizes are different"
        );
        // We are going to iterate over half its border
        // Starting at our k=0 and ending at our k=coords.get_num_radial_lines()
        // but from its perspective starting at k=br.get_chunk_coords().get_num_radial_lines()/2
        // (because we are on its left side)
        // and ending at k=br.get_chunk_coords().get_num_radial_lines()
        for k in 0..coords.get_num_radial_lines() {
            let our_idx = JkVector { j: 0, k };
            let their_idx = JkVector {
                j: br.get_chunk_coords().get_num_concentric_circles() - 1,
                k: k / 2 + br.get_chunk_coords().get_num_radial_lines() / 2,
            };
            let temp = br.get_temperature(their_idx);
            this[our_idx.to_ndarray_coords(coords).x] = temp.0;
        }
    };
}

fn left_right_neighbor_grids(
    l: &ElementGrid,
    out: &mut ElementGridConvolutionNeighborTemperatures,
    r: &ElementGrid,
) {
    let coords = l.get_chunk_coords();
    let mut this = Array1::zeros(coords.get_num_concentric_circles());
    for j in 0..coords.get_num_concentric_circles() {
        let idx = JkVector { j, k: 0 };
        let temp = l.get_temperature(idx);
        this[idx.to_ndarray_coords(coords).y] = temp.0;
    }
    out.left = this;

    let coords = r.get_chunk_coords();
    let mut this = Array1::zeros(coords.get_num_concentric_circles());
    for j in 0..coords.get_num_concentric_circles() {
        let idx = JkVector {
            j,
            k: coords.get_num_radial_lines() - 1,
        };
        let temp = r.get_temperature(idx);
        this[idx.to_ndarray_coords(coords).y] = temp.0;
    }
    out.right = this;
}
