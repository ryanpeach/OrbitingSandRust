#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use bevy::app;

use crate::physics::{
    fallingsand::{
        data::element_grid::ElementGrid,
        elements::{
            element::{Element, ElementType},
            vacuum::Vacuum,
        },
        mesh::{
            chunk_coords::{self, ChunkCoords},
            coordinate_directory::CoordinateDir,
        },
        util::{
            grid::Grid,
            vectors::{ChunkIjkVector, JkVector},
        },
    },
    util::clock::Clock,
};

/// A representation of the position of an element in a layer
pub struct ElementLayerPosition {
    pub chunk_idx: ChunkIjkVector,
    pub element_idx: JkVector,
}

impl ElementLayerPosition {
    /// Convert to an absolute position in the layer
    pub fn to_layer_relative(&self, coords: &CoordinateDir) -> JkVector {
        let chunk_coords = coords.get_chunk_at_idx(self.chunk_idx);
        let layer_relative_start_j = chunk_coords.get_start_concentric_circle_layer_relative();
        let layer_relative_start_k = chunk_coords.get_start_radial_line();
        JkVector {
            j: layer_relative_start_j + self.element_idx.j,
            k: layer_relative_start_k + self.element_idx.k,
        }
    }
}

/// A representation of the coordinates of the chunks in a layer
pub struct Layer {
    coord_dir: CoordinateDir,
    layer_num: usize,
    layer_chunks: Grid<ElementGrid>,
}

/// Simple getters and setters
impl Layer {
    /// Create a new layer with the given chunks
    /// We take ownership of the chunks, you can get them back with `into_inner`
    pub fn new(
        layer_num: usize,
        layer_chunks: Grid<ElementGrid>,
        coord_dir: CoordinateDir,
    ) -> Self {
        Self {
            layer_num,
            layer_chunks,
            coord_dir,
        }
    }

    /// Get the chunks in the layer
    pub fn into_inner(self) -> Grid<ElementGrid> {
        self.layer_chunks
    }
}

/// Fill Operations
impl Layer {
    /// Apply the given action to each element in the layer
    pub fn apply_to_each_element<F>(&mut self, mut action: F, clock: Clock)
    where
        F: FnMut(&Layer, &mut ElementGrid, ElementLayerPosition, Clock) -> (),
        Clock: Copy, // Assuming Clock can be copied, adjust accordingly
    {
        for j in 0..self.layer_chunks.get_height() {
            for k in 0..self.layer_chunks.get_width() {
                let chunk_idx = ChunkIjkVector {
                    i: self.layer_num,
                    j,
                    k,
                };
                let chunk = self.layer_chunks.get_mut(chunk_idx.to_jk_vector());
                let coords = chunk.get_chunk_coords();
                for j_ in 0..coords.get_num_concentric_circles() {
                    for k_ in 0..coords.get_num_radial_lines() {
                        let element_idx = JkVector { j: j_, k: k_ };
                        action(
                            &self,
                            chunk,
                            ElementLayerPosition {
                                chunk_idx,
                                element_idx,
                            },
                            clock,
                        ); // Assuming `self.clock` is accessible, adjust as needed
                    }
                }
            }
        }
    }

    /// Fill the layer with the given element
    pub fn fill(&mut self, element: &dyn Element, clock: Clock) {
        self.apply_to_each_element(
            |_, chunk, position, clock| chunk.set(position.element_idx, element.box_clone(), clock),
            clock,
        )
    }

    /// Fill the layer in with the given element randomly with a probability of `probability`
    pub fn fill_rand(&mut self, element: &dyn Element, probability: f64, clock: Clock) {
        self.apply_to_each_element(
            |_, chunk, position, clock| {
                if rand::random::<f64>() < probability {
                    chunk.set(position.element_idx, element.box_clone(), clock)
                }
            },
            clock,
        )
    }
}

/// Mask Operations
/// These usually destroy the other layer and mutate this layer
impl Layer {
    /// Randomly select elements from the other layer with a given probability and put them in this layer
    pub fn randomly_select(&mut self, other: Layer, probability: f64, clock: Clock) {
        self.apply_to_each_element(
            |_, chunk, position, clock| {
                if rand::random::<f64>() < probability {
                    chunk.set(
                        position.element_idx,
                        other
                            .layer_chunks
                            .get(position.chunk_idx.to_jk_vector())
                            .replace(position.element_idx, Box::new(Vacuum::default()), clock),
                        clock,
                    )
                }
            },
            clock,
        )
    }

    /// Replace all vaccum elements in this layer with elements from the other layer
    pub fn replace_vacuum(&mut self, other: Layer, clock: Clock) {
        self.apply_to_each_element(
            |_, chunk, position, clock| {
                if chunk.get(position.element_idx).get_type() == ElementType::Vacuum {
                    chunk.set(
                        position.element_idx,
                        other
                            .layer_chunks
                            .get(position.chunk_idx.to_jk_vector())
                            .replace(position.element_idx, Box::new(Vacuum::default()), clock),
                        clock,
                    )
                }
            },
            clock,
        )
    }
}

/// Procedural generation
/// Generate lines that can be added together to form a shape
/// Then you can fill under and over the shape
impl Layer {
    /// Given a vector of j indexes relative to the layer:
    ///   Fill anything under the shape with element A
    ///   Fill anything over the shape with element B
    pub fn fill_shape(
        &mut self,
        shape: ndarray::Array1<f32>,
        under_element: &dyn Element,
        over_element: &dyn Element,
        clock: Clock,
    ) {
        self.apply_to_each_element(
            |this, chunk, position, clock| {
                for (k, threshold) in shape.iter().enumerate() {
                    let layer_relative = position.to_layer_relative(&this.coord_dir);
                    if layer_relative.k == k {
                        if layer_relative.j < *(threshold as usize) {
                            chunk.set(position.element_idx, under_element.box_clone(), clock)
                        } else {
                            chunk.set(position.element_idx, over_element.box_clone(), clock)
                        }
                    }
                }
            },
            clock,
        )
    }

    /// Generate a sinusoid with the following conditions
    /// Amplitude is a number between 0 and 1
    ///   where 0 is no sinusoidal curve (straight down the middle)
    ///   and 1 is a full sinusoidal curve (peak is at the top of the layer, trough is at the bottom)
    /// Frequency is a number where 1 is one sinusoid per chunk
    /// Offset is a number between 0 and 1 where 1 is a period
    /// With FFT you can make almost any shape with a combination of sinusoids
    /// TODO: Test this
    pub fn sin(&self, amplitude: f32, frequency: f32, offset: f32) -> ndarray::Array1<f32> {
        let j_max = amplitude
            * self
                .coord_dir
                .get_layer_num_concentric_circles(self.layer_num) as f32;
        let j_min = (1.0 - amplitude)
            * self
                .coord_dir
                .get_layer_num_concentric_circles(self.layer_num) as f32;
        let k_max = self.coord_dir.get_layer_num_radial_lines(self.layer_num) as f32;
        let k_min = 0.0;
        let k_chunks = self.coord_dir.get_layer_num_radial_chunks(self.layer_num) as f32;
        let mut out =
            ndarray::Array1::zeros(self.coord_dir.get_layer_num_radial_lines(self.layer_num));
        let period_at_frequency_1 = k_max / k_chunks;
        let offset = offset * period_at_frequency_1;
        for (k, item) in out.iter_mut().enumerate() {
            let k = k as f32;
            let k = k * period_at_frequency_1 / frequency;
            *item = j_min + (j_max - j_min) * (1.0 + (k + offset).sin()) / 2.0;
        }
        out
    }
}

/// 2d functions
impl Layer {
    pub fn probabilistic_y_gradient(
        &self,
        start_probability: f32,
        end_probability: f32,
    ) -> ndarray::Array2<f32> {
        todo!("Implement this");
    }

    pub fn perlin_noise(
        &self,
        frequency: f32,
        amplitude: f32,
        offset: f32,
    ) -> ndarray::Array2<f32> {
        todo!("Implement this");
    }
}
