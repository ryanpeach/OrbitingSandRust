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

        use crate::physics::{
            fallingsand::elements::element::ElementType, heat::math::PropogateHeat,
        };

        /// Determines how fast the heat diffuses
        #[test]
        fn test_sink_diffuses_to_zero_speed() {
            PropogateHeat::test_heat_disipation_rate_in_space(4078, 1, ElementType::Lava);
        }
    }
}
