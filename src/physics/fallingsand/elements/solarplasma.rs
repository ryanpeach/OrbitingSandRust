use super::element::{
    Element, ElementTakeOptions, ElementType, SetHeatOnZeroSpecificHeatError, StateOfMatter,
};
use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::data::element_grid::ElementGrid;
use crate::physics::fallingsand::mesh::coordinate_directory::CoordinateDir;
use crate::physics::fallingsand::util::vectors::JkVector;
use crate::physics::heat::components::{
    Compressability, Density, HeatEnergy, Length, SpecificHeat, ThermalConductivity,
    ThermodynamicTemperature,
};
use crate::physics::util::clock::Clock;
use bevy::render::color::Color;
use rand::Rng;

/// Literally nothing
#[derive(Copy, Clone, Debug)]
pub struct SolarPlasma {
    last_processed: Clock,
    heat: HeatEnergy,
}

impl SolarPlasma {
    /// Create a new SolarPlasma
    pub fn new(cell_width: Length) -> Self {
        let mut out = Self {
            last_processed: Clock::default(),
            heat: HeatEnergy::default(),
        };
        out.set_heat(
            out.get_default_temperature().heat_energy(
                out.get_specific_heat()
                    .heat_capacity(out.get_density().mass(cell_width)),
            ),
            Clock::default(),
        )
        .unwrap();
        out
    }
}

impl Element for SolarPlasma {
    fn get_type(&self) -> ElementType {
        ElementType::SolarPlasma
    }
    fn get_density(&self) -> Density {
        Density(100.0)
    }
    fn get_last_processed(&self) -> Clock {
        self.last_processed
    }
    fn _set_last_processed(&mut self, current_time: Clock) {
        self.last_processed = current_time;
    }
    fn get_state_of_matter(&self) -> StateOfMatter {
        StateOfMatter::Liquid
    }
    fn get_color(&self) -> Color {
        Color::ORANGE
    }
    fn _process(
        &mut self,
        pos: JkVector,
        coord_dir: &CoordinateDir,
        target_chunk: &mut ElementGrid,
        element_grid_conv: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) -> ElementTakeOptions {
        // Go down one cell
        let below = element_grid_conv.get_below_idx_from_center(target_chunk, coord_dir, &pos, 1);
        let element = {
            match below {
                Ok(below) => element_grid_conv.get(target_chunk, below),
                Err(err) => Err(err),
            }
        };
        // If we are still in the center chunk, first check if below is vacuum
        // If it is, swap with it
        // Then check if it is water
        // If it is, then either go left or right if they are vacuum
        // Otherwise check if left or right is vacuum
        // If it is, swap with one of them randomly
        match element {
            Ok(element) => {
                if element.get_state_of_matter() <= StateOfMatter::Gas {
                    self.try_swap_me(
                        below.unwrap(),
                        target_chunk,
                        element_grid_conv,
                        current_time,
                    )
                } else {
                    let new_idx_l =
                        element_grid_conv.get_left_right_idx_from_center(target_chunk, &pos, 1);
                    let new_idx_r =
                        element_grid_conv.get_left_right_idx_from_center(target_chunk, &pos, -1);
                    let element_l = {
                        match new_idx_l {
                            Ok(new_idx_l) => element_grid_conv.get(target_chunk, new_idx_l),
                            Err(err) => Err(err),
                        }
                    };
                    let element_r = {
                        match new_idx_r {
                            Ok(new_idx_r) => element_grid_conv.get(target_chunk, new_idx_r),
                            Err(err) => Err(err),
                        }
                    };

                    // Now decide if we go left or right
                    let mut rng = rand::thread_rng();
                    let rand_bool = rng.gen_bool(0.5);
                    match (element_l, element_r, rand_bool) {
                        (Ok(element_l), Ok(_), false) => {
                            if element_l.get_state_of_matter() <= StateOfMatter::Gas {
                                self.try_swap_me(
                                    new_idx_l.unwrap(),
                                    target_chunk,
                                    element_grid_conv,
                                    current_time,
                                )
                            } else {
                                ElementTakeOptions::PutBack
                            }
                        }
                        (Ok(_), Ok(element_r), true) => {
                            if element_r.get_state_of_matter() <= StateOfMatter::Gas {
                                self.try_swap_me(
                                    new_idx_r.unwrap(),
                                    target_chunk,
                                    element_grid_conv,
                                    current_time,
                                )
                            } else {
                                ElementTakeOptions::PutBack
                            }
                        }
                        (Ok(element_l), Err(_), _) => {
                            if element_l.get_state_of_matter() <= StateOfMatter::Gas {
                                self.try_swap_me(
                                    new_idx_l.unwrap(),
                                    target_chunk,
                                    element_grid_conv,
                                    current_time,
                                )
                            } else {
                                ElementTakeOptions::PutBack
                            }
                        }
                        (Err(_), Ok(element_r), _) => {
                            if element_r.get_state_of_matter() <= StateOfMatter::Gas {
                                self.try_swap_me(
                                    new_idx_r.unwrap(),
                                    target_chunk,
                                    element_grid_conv,
                                    current_time,
                                )
                            } else {
                                ElementTakeOptions::PutBack
                            }
                        }
                        (Err(_), Err(_), _) => ElementTakeOptions::PutBack,
                    }
                }
            }
            Err(_) => ElementTakeOptions::PutBack,
        }
    }
    fn box_clone(&self) -> Box<dyn Element> {
        Box::new(*self)
    }

    fn get_heat(&self) -> HeatEnergy {
        self.heat
    }

    fn set_heat(
        &mut self,
        heat: HeatEnergy,
        current_time: Clock,
    ) -> Result<(), SetHeatOnZeroSpecificHeatError> {
        self.heat = heat;
        self._set_last_processed(current_time);
        Ok(())
    }

    fn get_default_temperature(&self) -> ThermodynamicTemperature {
        ThermodynamicTemperature(10000.0)
    }

    fn get_specific_heat(&self) -> SpecificHeat {
        SpecificHeat(840.0 * 2.0) // Twice lava
    }

    fn get_thermal_conductivity(&self) -> ThermalConductivity {
        ThermalConductivity(1.0)
    }

    fn get_compressability(&self) -> Compressability {
        Compressability(100.0)
    }
}

#[cfg(test)]
mod test {
    mod heat {
        use std::time::Duration;

        use crate::physics::{
            fallingsand::{
                convolution::behaviors::ElementGridConvolutionNeighborTemperatures,
                elements::element::ElementType, util::vectors::JkVector,
            },
            heat::{
                components::{Length, ThermodynamicTemperature},
                math::PropogateHeatBuilder,
            },
            orbits::components::Mass,
            util::clock::Clock,
        };

        /// Determines how fast the heat diffuses
        #[test]
        fn test_sink_diffuses_to_zero_speed() {
            // Set up the builder
            let mut builder = PropogateHeatBuilder::new(5, 5, Length(1.0));
            builder.enable_compression(false);
            builder.total_mass_above(Mass(0.0));
            for j in 0..5 {
                for k in 0..5 {
                    builder.add(
                        JkVector::new(j, k),
                        &ElementType::SolarPlasma
                            .get_element(Length(1.0))
                            .box_clone(),
                    );
                }
            }
            let mut heat = builder.build(ElementGridConvolutionNeighborTemperatures {
                left: ThermodynamicTemperature(0.0),
                right: ThermodynamicTemperature(0.0),
                top: Some(ThermodynamicTemperature(0.0)),
                bottom: Some(ThermodynamicTemperature(0.0)),
            });

            const FRAME_RATE: u32 = 1;
            const N: u32 = 10233;
            let mut clock = Clock::default();
            for frame_cnt in 0..(N * FRAME_RATE) {
                assert!(
                    heat.get_temperature()[[2, 2]].abs() > 1.0,
                    "Took less than {} frames to cool down: {}",
                    N * FRAME_RATE,
                    frame_cnt
                );

                // Update the clock
                clock.update(Duration::from_secs_f32(1.0 / FRAME_RATE as f32));

                // Check that the heat is not yet near zero in the center
                let heat_energy = heat.get_temperature().clone();
                if frame_cnt % 1 == 0 {
                    println!(
                        "#{:?} Heat energy:\n{:?}",
                        frame_cnt / FRAME_RATE,
                        heat_energy
                    );
                }

                // Propogate the heat
                heat.propagate_heat(clock);
            }

            // Check that the heat is near zero in the center
            assert!(
                heat.get_temperature()[[2, 2]].abs() < 1.0,
                "Took longer than {} frames to cool down.",
                N * FRAME_RATE
            );
        }
    }
}
