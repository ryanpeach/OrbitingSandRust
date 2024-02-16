use std::ops::{Add, Div, Index, IndexMut};

use ndarray::Array1;

use crate::physics::fallingsand::{
    convolution::{
        behaviors::ElementGridConvolutionNeighbors,
        neighbor_grids::{BottomNeighborGrids, LeftRightNeighborGrids, TopNeighborGrids},
    },
    data::element_grid::ElementGrid,
    util::vectors::JkVector,
};

use super::components::{Density, SpecificHeat, ThermalConductivity, ThermodynamicTemperature};
#[derive(Clone, Debug, Default)]
pub struct MatrixBorderHeatProperties {
    // Now stores a single array of ElementHeatProperties
    pub properties: Option<Array1<ElementHeatProperties>>,
}

impl MatrixBorderHeatProperties {
    pub fn new(
        temperature: Array1<ThermodynamicTemperature>,
        thermal_conductivity: Array1<ThermalConductivity>,
        specific_heat_capacity: Array1<SpecificHeat>,
        density: Array1<Density>,
    ) -> MatrixBorderHeatProperties {
        let properties = temperature
            .iter()
            .zip(thermal_conductivity.iter())
            .zip(specific_heat_capacity.iter())
            .zip(density.iter())
            .map(
                |(((temperature, thermal_conductivity), specific_heat_capacity), density)| {
                    Some(ElementHeatProperties {
                        temperature: *temperature,
                        thermal_conductivity: *thermal_conductivity,
                        specific_heat_capacity: *specific_heat_capacity,
                        density: *density,
                    })
                },
            )
            .collect();
        Self { properties }
    }
    pub fn zeros(size: usize) -> MatrixBorderHeatProperties {
        let temperature = Array1::zeros(size);
        let thermal_conductivity = Array1::zeros(size);
        let specific_heat_capacity = Array1::zeros(size);
        let density = Array1::zeros(size);
        Self::new(
            temperature,
            thermal_conductivity,
            specific_heat_capacity,
            density,
        )
    }
    pub fn temperature(&self, idx: usize) -> ThermodynamicTemperature {
        self.properties.as_ref().unwrap()[idx].temperature
    }
    pub fn thermal_conductivity(&self, idx: usize) -> ThermalConductivity {
        self.properties.as_ref().unwrap()[idx].thermal_conductivity
    }
    pub fn specific_heat_capacity(&self, idx: usize) -> SpecificHeat {
        self.properties.as_ref().unwrap()[idx].specific_heat_capacity
    }
    pub fn density(&self, idx: usize) -> Density {
        self.properties.as_ref().unwrap()[idx].density
    }
    pub fn temperatures(&self) -> Array1<f32> {
        self.properties
            .as_ref()
            .unwrap()
            .iter()
            .map(|x| x.temperature.0)
            .collect()
    }
    pub fn thermal_conductivities(&self) -> Array1<f32> {
        self.properties
            .as_ref()
            .unwrap()
            .iter()
            .map(|x| x.thermal_conductivity.0)
            .collect()
    }
    pub fn specific_heat_capacities(&self) -> Array1<f32> {
        self.properties
            .as_ref()
            .unwrap()
            .iter()
            .map(|x| x.specific_heat_capacity.0)
            .collect()
    }
    pub fn densities(&self) -> Array1<f32> {
        self.properties
            .as_ref()
            .unwrap()
            .iter()
            .map(|x| x.density.0)
            .collect()
    }
}

impl Index<usize> for MatrixBorderHeatProperties {
    type Output = ElementHeatProperties;

    fn index(&self, index: usize) -> &Self::Output {
        &self.properties.as_ref().unwrap()[index]
    }
}

impl IndexMut<usize> for MatrixBorderHeatProperties {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.properties.as_mut().unwrap()[index]
    }
}

#[derive(Clone, Debug, Copy)]
pub struct ElementHeatProperties {
    pub temperature: ThermodynamicTemperature,
    pub thermal_conductivity: ThermalConductivity,
    pub specific_heat_capacity: SpecificHeat,
    pub density: Density,
}

impl Add for ElementHeatProperties {
    type Output = ElementHeatProperties;

    fn add(self, rhs: Self) -> Self::Output {
        ElementHeatProperties {
            temperature: self.temperature + rhs.temperature,
            thermal_conductivity: self.thermal_conductivity + rhs.thermal_conductivity,
            specific_heat_capacity: self.specific_heat_capacity + rhs.specific_heat_capacity,
            density: self.density + rhs.density,
        }
    }
}

impl Div<f32> for ElementHeatProperties {
    type Output = ElementHeatProperties;

    fn div(self, rhs: f32) -> Self::Output {
        ElementHeatProperties {
            temperature: self.temperature / rhs,
            thermal_conductivity: self.thermal_conductivity / rhs,
            specific_heat_capacity: self.specific_heat_capacity / rhs,
            density: self.density / rhs,
        }
    }
}

/// The output of [BorderTemperatures::get_border_temps]
#[derive(Clone, Debug, Default)]
pub struct ElementGridConvolutionNeighborTemperatures {
    /// The heat of the top neighbor border
    pub top: Option<MatrixBorderHeatProperties>,
    /// The heat of the bottom neighbor border
    pub bottom: Option<MatrixBorderHeatProperties>,
    /// The heat of the left neighbor border
    pub left: MatrixBorderHeatProperties,
    /// The heat of the right neighbor border
    pub right: MatrixBorderHeatProperties,
}

impl ElementGridConvolutionNeighborTemperatures {
    /// Create a new [ElementGridConvolutionNeighborTemperatures] with zero arrays
    pub fn zeros(size: usize) -> ElementGridConvolutionNeighborTemperatures {
        ElementGridConvolutionNeighborTemperatures {
            top: Some(MatrixBorderHeatProperties::zeros(size)),
            bottom: Some(MatrixBorderHeatProperties::zeros(size)),
            left: MatrixBorderHeatProperties::zeros(size),
            right: MatrixBorderHeatProperties::zeros(size),
        }
    }
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
        let mut top = MatrixBorderHeatProperties::zeros(coords.get_num_radial_lines());
        match &self.grids.top {
            TopNeighborGrids::Normal { t, tl, tr: _ } => {
                get_top_neighbor_grids_normal(coords, tl, t, &mut top);
                out.top = Some(top);
            }
            // In this case t1 and t0 are both half the size of target_chunk
            // TODO: Test this
            TopNeighborGrids::ChunkDoubling { t1, t0, .. } => {
                get_top_neighbor_grids_chunk_doubling(coords, t0, t1, &mut top);
                out.top = Some(top);
            }
            TopNeighborGrids::TopOfGrid => {
                out.top = None;
            }
        }
        let mut bottom = MatrixBorderHeatProperties::zeros(coords.get_num_radial_lines());
        match &self.grids.bottom {
            BottomNeighborGrids::Normal { b, .. } => {
                get_bottom_neighbor_grids_normal(coords, b, &mut bottom);
                out.bottom = Some(bottom);
            }
            // In this case bl and br are both twice the size of target_chunk
            BottomNeighborGrids::ChunkDoubling { bl, br } => {
                get_bottom_neighbor_grids_chunk_doubling(coords, bl, br, &mut bottom);
                out.bottom = Some(bottom);
            }
            BottomNeighborGrids::BottomOfGrid => {
                out.bottom = None;
            }
        }
        match &self.grids.left_right {
            LeftRightNeighborGrids::LR { l, r } => {
                get_left_right_neighbor_grids(l, r, &mut out);
            }
        }
        out
    }

    pub fn set_border_temps(
        &mut self,
        target_chunk: &mut ElementGrid,
        temps: ElementGridConvolutionNeighborTemperatures,
    ) {
        let coords = target_chunk.get_chunk_coords();
        match &mut self.grids.top {
            TopNeighborGrids::Normal { t, tl, tr } => {
                set_top_border_temps_normal(coords, tl, t, tr, temps.top.unwrap());
            }
            TopNeighborGrids::ChunkDoubling { t1, t0, .. } => {
                set_top_border_temps_chunk_doubling(coords, t0, t1, temps.top.unwrap());
            }
            TopNeighborGrids::TopOfGrid => {}
        }
        match &mut self.grids.bottom {
            BottomNeighborGrids::Normal { b, .. } => {
                set_bottom_border_temps_normal(coords, b, temps.bottom.unwrap());
            }
            BottomNeighborGrids::ChunkDoubling { bl, br } => {
                set_bottom_border_temps_chunk_doubling(coords, bl, br, temps.bottom.unwrap());
            }
            BottomNeighborGrids::BottomOfGrid => {}
        }
        match &mut self.grids.left_right {
            LeftRightNeighborGrids::LR { l, r } => {
                set_left_right_border_temps(l, r, temps.left, temps.right);
            }
        }
    }
}

fn set_top_border_temps_normal(
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    tl: &mut ElementGrid,
    t: &mut ElementGrid,
    tr: &mut ElementGrid,
    temps: MatrixBorderHeatProperties,
) {
    if t.get_chunk_coords().get_num_radial_lines() == coords.get_num_radial_lines() {
        set_top_border_temps_normal_no_cell_doubling(coords, t, temps);
    } else {
        set_top_border_temps_normal_cell_doubling(coords, tl, t, tr, temps);
    }
}

fn set_top_border_temps_normal_no_cell_doubling(
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    t: &mut ElementGrid,
    temps: MatrixBorderHeatProperties,
) {
    todo!()
}

fn set_top_border_temps_normal_cell_doubling(
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    tl: &mut ElementGrid,
    t: &mut ElementGrid,
    tr: &mut ElementGrid,
    temps: MatrixBorderHeatProperties,
) {
    todo!()
}

fn set_top_border_temps_chunk_doubling(
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    t0: &mut ElementGrid,
    t1: &mut ElementGrid,
    temps: MatrixBorderHeatProperties,
) {
    todo!()
}

fn set_bottom_border_temps_normal(
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    b: &mut ElementGrid,
    temps: MatrixBorderHeatProperties,
) {
    todo!()
}

fn set_bottom_border_temps_chunk_doubling(
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    bl: &mut ElementGrid,
    br: &mut ElementGrid,
    temps: MatrixBorderHeatProperties,
) {
    todo!()
}

fn set_left_right_border_temps(
    l: &mut ElementGrid,
    r: &mut ElementGrid,
    temps_l: MatrixBorderHeatProperties,
    temps_r: MatrixBorderHeatProperties,
) {
    todo!()
}

fn get_top_neighbor_grids_normal(
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    tl: &ElementGrid,
    t: &ElementGrid,
    this: &mut MatrixBorderHeatProperties,
) {
    if t.get_chunk_coords().get_num_radial_lines() == coords.get_num_radial_lines() {
        get_top_neighbor_grids_normal_no_cell_doubling(coords, t, this);
    } else {
        get_top_neighbor_grids_normal_cell_doubling(coords, tl, t, this);
    }
}

fn get_top_neighbor_grids_normal_no_cell_doubling(
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    t: &ElementGrid,
    this: &mut MatrixBorderHeatProperties,
) {
    for k in 0..coords.get_num_radial_lines() {
        let idx = JkVector { j: 0, k };
        let temp = t.get_heat_properties(idx);
        this[idx.to_ndarray_coords(coords).get_x()] = temp;
    }
}

fn get_top_neighbor_grids_normal_cell_doubling(
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    tl: &ElementGrid,
    t: &ElementGrid,
    this: &mut MatrixBorderHeatProperties,
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
            let temp = t.get_heat_properties(their_idx);
            this[our_idx.to_ndarray_coords(coords).get_x()] = temp;
        }
        // In this case we will average it with ourselves
        else {
            let temp = tl.get_heat_properties(their_idx);
            this[our_idx.to_ndarray_coords(coords).get_x()] =
                (temp + this[our_idx.to_ndarray_coords(coords).get_x()]) / 2.0;
        }
    }
}

fn get_top_neighbor_grids_chunk_doubling(
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    t0: &ElementGrid,
    t1: &ElementGrid,
    this: &mut MatrixBorderHeatProperties,
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
        let temp = t0.get_heat_properties(their_idx);
        let our_idx = JkVector {
            j: coords.get_num_concentric_circles() - 1,
            k: k / 2,
        };
        // In this case we will put the cell in the memory because
        // it comes first
        if k % 2 == 0 {
            this[our_idx.to_ndarray_coords(coords).get_x()] = temp;
        }
        // In this case we will average it with ourselves
        else {
            this[our_idx.to_ndarray_coords(coords).get_x()] =
                (temp + this[our_idx.to_ndarray_coords(coords).get_x()]) / 2.0;
        }
    }
    // Now lets do this from t1, iterating over its bottom border
    // and averaging that into every other one of our cells
    // startging from the middle of our radial lines
    for k in 0..t1.get_chunk_coords().get_num_radial_lines() {
        let their_idx = JkVector { j: 0, k };
        let temp = t1.get_heat_properties(their_idx);
        let our_idx = JkVector {
            j: coords.get_num_concentric_circles() - 1,
            k: k / 2 + coords.get_num_radial_lines() / 2,
        };
        // In this case we will put the cell in the memory because
        // it comes first
        if k % 2 == 0 {
            this[our_idx.to_ndarray_coords(coords).get_x()] = temp;
        }
        // In this case we will average it with ourselves
        else {
            this[our_idx.to_ndarray_coords(coords).get_x()] =
                (temp + this[our_idx.to_ndarray_coords(coords).get_x()]) / 2.0;
        }
    }
}

fn get_bottom_neighbor_grids_normal(
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    b: &ElementGrid,
    this: &mut MatrixBorderHeatProperties,
) {
    if coords.get_num_radial_lines() == b.get_chunk_coords().get_num_radial_lines() {
        get_bottom_neighbor_grids_normal_no_cell_doubling(coords, b, this);
    } else {
        get_bottom_neighbor_grids_normal_cell_doubling(coords, b, this);
    }
}

fn get_bottom_neighbor_grids_normal_cell_doubling(
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    b: &ElementGrid,
    this: &mut MatrixBorderHeatProperties,
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
        let temp = b.get_heat_properties(their_idx);
        this[our_idx.to_ndarray_coords(coords).get_x()] = temp;
    }
}

fn get_bottom_neighbor_grids_normal_no_cell_doubling(
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    b: &ElementGrid,
    this: &mut MatrixBorderHeatProperties,
) {
    for k in 0..coords.get_num_radial_lines() {
        let idx = JkVector {
            j: coords.get_num_concentric_circles() - 1,
            k,
        };
        let temp = b.get_heat_properties(idx);
        this[idx.to_ndarray_coords(coords).get_x()] = temp;
    }
}

fn get_bottom_neighbor_grids_chunk_doubling(
    coords: &crate::physics::fallingsand::mesh::chunk_coords::ChunkCoords,
    bl: &ElementGrid,
    br: &ElementGrid,
    this: &mut MatrixBorderHeatProperties,
) {
    // TODO: document this with pictures
    // TODO: Unit test
    let mut this = MatrixBorderHeatProperties::zeros(coords.get_num_radial_lines());
    // This is the case where the bottom neighbor is the bl chunk
    // And we are straddling the right side of the bl chunk
    if coords.get_chunk_idx().k % 2 == 0 {
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
            let temp = bl.get_heat_properties(their_idx);
            this[our_idx.to_ndarray_coords(coords).get_x()] = temp;
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
            let temp = br.get_heat_properties(their_idx);
            this[our_idx.to_ndarray_coords(coords).get_x()] = temp;
        }
    };
}

fn get_left_right_neighbor_grids(
    l: &ElementGrid,
    r: &ElementGrid,
    out: &mut ElementGridConvolutionNeighborTemperatures,
) {
    let coords = l.get_chunk_coords();
    let mut this = MatrixBorderHeatProperties::zeros(coords.get_num_concentric_circles());
    for j in 0..coords.get_num_concentric_circles() {
        let idx = JkVector { j, k: 0 };
        let temp = l.get_heat_properties(idx);
        this[idx.to_ndarray_coords(coords).get_y()] = temp;
    }
    out.left = this;

    let coords = r.get_chunk_coords();
    let mut this = MatrixBorderHeatProperties::zeros(coords.get_num_concentric_circles());
    for j in 0..coords.get_num_concentric_circles() {
        let idx = JkVector {
            j,
            k: coords.get_num_radial_lines() - 1,
        };
        let temp = r.get_heat_properties(idx);
        this[idx.to_ndarray_coords(coords).get_y()] = temp;
    }
    out.right = this;
}
