use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::mesh::coordinate_directory::CoordinateDir;
use crate::physics::fallingsand::data::element_grid::ElementGrid;
use crate::physics::fallingsand::elements::element::{Element, ElementTakeOptions, ElementType};
use crate::physics::fallingsand::util::vectors::JkVector;
use crate::physics::util::clock::Clock;
use ggez::graphics::Color;

/// Literally nothing
#[derive(Default, Copy, Clone, Debug)]
pub struct LeftFlier {
    last_processed: Clock,
}

impl Element for LeftFlier {
    fn get_type(&self) -> ElementType {
        ElementType::LeftFlier
    }
    fn get_last_processed(&self) -> Clock {
        self.last_processed
    }
    #[allow(clippy::borrowed_box)]
    fn get_color(&self) -> Color {
        Color::from_rgb(254, 254, 254)
    }
    fn process(
        &mut self,
        pos: JkVector,
        coord_dir: &CoordinateDir,
        target_chunk: &mut ElementGrid,
        element_grid_conv: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) -> ElementTakeOptions {
        // Doing this as a way to make sure I set last_processed AFTER I've done all the processing
        let out: ElementTakeOptions = {
            let left =
                element_grid_conv.get_left_right_idx_from_center(target_chunk, coord_dir, &pos, 1);
            match left {
                Ok(idx) => {
                    if let Ok(element) = element_grid_conv.get(target_chunk, idx) {
                        match element.get_type() {
                            ElementType::Vacuum => {
                                self.try_swap_me(idx, target_chunk, element_grid_conv, current_time)
                            }
                            _ => ElementTakeOptions::PutBack,
                        }
                    } else {
                        ElementTakeOptions::PutBack
                    }
                }
                Err(_) => ElementTakeOptions::PutBack,
            }
        };
        self.last_processed = current_time;
        out
    }
    fn box_clone(&self) -> Box<dyn Element> {
        Box::new(*self)
    }
}
