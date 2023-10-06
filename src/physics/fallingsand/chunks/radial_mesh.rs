use super::core::CoreChunk;
use super::partial_layer::PartialLayerChunk;

pub struct RadialMesh {
    cell_radius: f32,
    nb_layers: usize,
    fist_nb_radial_lines: usize,
    second_nb_concentric_circles: usize,
    _core_chunk: CoreChunk,
    _partial_chunks: Vec<PartialLayerChunk>,
}

impl RadialMesh {
    pub fn new(
        cell_radius: f32,
        nb_layers: usize,
        fist_num_radial_lines: usize,
        second_num_concentric_circles: usize,
    ) -> Self {
        if nb_layers <= 1 {
            panic!("RadialMesh::new: nb_layers must be greater than 1");
        }
        let _core_chunk = CoreChunk::new(cell_radius, fist_num_radial_lines);
        let mut _partial_chunks: Vec<PartialLayerChunk> = Vec::new();

        // These variables will help us keep track of the current layer
        let mut num_radial_lines = fist_num_radial_lines * 2;
        let mut num_concentric_circles = second_num_concentric_circles;
        let mut start_concentric_circle_absolute = 1;
        let mut layer_num = 1;

        // Handle the first few layers
        loop {
            if layer_num >= nb_layers {
                break;
            }
            let next_layer = PartialLayerChunk::new(
                cell_radius,
                0,
                num_radial_lines,
                num_radial_lines,
                num_concentric_circles,
                0,
                start_concentric_circle_absolute,
            );
            _partial_chunks.push(next_layer);

            // Modify the variables
            start_concentric_circle_absolute += num_concentric_circles;
            num_radial_lines *= 2;
            num_concentric_circles *= 2;
            layer_num += 1;

            // At 64x64, break
            if (num_radial_lines * num_concentric_circles) > 4096 {
                break;
            }
        }

        // Handle the second set of layers, which just subdivide around the grid
        let mut num_radial_chunks = 2;
        loop {
            if layer_num >= nb_layers {
                break;
            }

            // TODO: Check this
            for i in 0..num_radial_chunks {
                let next_layer = PartialLayerChunk::new(
                    cell_radius,
                    i,
                    num_radial_lines,
                    num_radial_lines,
                    num_concentric_circles,
                    0,
                    start_concentric_circle_absolute,
                );
                _partial_chunks.push(next_layer);
            }

            // Modify the variables
            start_concentric_circle_absolute += num_concentric_circles;
            num_radial_lines *= 2;
            num_concentric_circles *= 2;
            layer_num += 1;

            // At 64x64, multiply the number of radial chunks by 2
            if (num_radial_lines / num_radial_chunks * num_concentric_circles) > 4096 {
                num_radial_chunks *= 2;
            }

            // If our width is smaller than our height, break
            if num_radial_lines / num_radial_chunks < num_concentric_circles {
                break;
            }
        }

        // Handle the third set of layers, which just subdivide both around the grid and up/down the grid
        let mut num_concentric_chunks = 2;
        loop {
            if layer_num >= nb_layers {
                break;
            }

            // TODO: Check this
            for i in 0..num_radial_chunks {
                for j in 0..num_concentric_chunks {
                    let next_layer = PartialLayerChunk::new(
                        cell_radius,
                        i,
                        num_radial_lines,
                        num_radial_lines,
                        num_concentric_circles,
                        j,
                        start_concentric_circle_absolute,
                    );
                    _partial_chunks.push(next_layer);
                }
            }

            // Modify the variables
            start_concentric_circle_absolute += num_concentric_circles;
            num_radial_lines *= 2;
            num_concentric_circles *= 2;
            layer_num += 1;

            // At 64x64, multiply the number of concentric chunks and radial chunks by 2
            if (num_radial_lines / num_radial_chunks * num_concentric_circles
                / num_concentric_chunks)
                > 4096
            {
                num_radial_chunks *= 2;
                num_concentric_chunks *= 2;
            }
        }

        Self {
            cell_radius,
            nb_layers,
            fist_nb_radial_lines,
            second_nb_concentric_circles,
            _core_chunk,
            _partial_chunks,
        }
    }
}
