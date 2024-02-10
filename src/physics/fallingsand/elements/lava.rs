use super::element::{
    Element, ElementTakeOptions, ElementType, SetHeatOnZeroSpecificHeatError, StateOfMatter,
};
use super::movement::fluid::fluid_process;
use super::stone::Stone;
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

/// The temperature at which lava transitions to a solid or vice versa
pub const LAVA_STATE_TRANSITION_TEMPERATURE_K: ThermodynamicTemperature =
    ThermodynamicTemperature(1000.0);

/// Literally nothing
#[derive(Copy, Clone, Debug)]
pub struct Lava {
    last_processed: Clock,
    heat: HeatEnergy,
}

impl Lava {
    /// Create a new Stone
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

impl Element for Lava {
    fn get_type(&self) -> ElementType {
        ElementType::Lava
    }
    fn get_density(&self) -> Density {
        Density(1.0)
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
    // Gray
    fn get_color(&self) -> Color {
        Color::RED
    }
    // Stone does nothing
    fn _process(
        &mut self,
        pos: JkVector,
        coord_dir: &CoordinateDir,
        target_chunk: &mut ElementGrid,
        element_grid_conv: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) -> ElementTakeOptions {
        if self.get_temperature(coord_dir.get_cell_width()) < LAVA_STATE_TRANSITION_TEMPERATURE_K {
            let mut stone = Stone::new(coord_dir.get_cell_width());
            stone.set_heat(self.heat, current_time).unwrap();
            ElementTakeOptions::ReplaceWith(Box::new(stone))
        } else {
            fluid_process(
                self,
                pos,
                coord_dir,
                target_chunk,
                element_grid_conv,
                current_time,
            )
        }
    }
    fn box_clone(&self) -> Box<dyn Element> {
        Box::new(*self)
    }

    fn get_default_temperature(&self) -> ThermodynamicTemperature {
        ThermodynamicTemperature(1500.0)
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

    fn get_specific_heat(&self) -> SpecificHeat {
        SpecificHeat(840.0)
    }

    fn get_thermal_conductivity(&self) -> ThermalConductivity {
        ThermalConductivity(1.0)
    }

    fn get_compressability(&self) -> Compressability {
        Compressability(0.001)
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
                        &ElementType::Lava.get_element(Length(1.0)).box_clone(),
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
            const N: u32 = 4078;
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
