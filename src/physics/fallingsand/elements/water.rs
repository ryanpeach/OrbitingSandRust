use super::element::{
    Element, ElementTakeOptions, ElementType, SetHeatOnZeroSpecificHeatError, StateOfMatter,
};
use super::movement::fluid::fluid_process;
use crate::physics::fallingsand::convolution::behaviors::ElementGridConvolutionNeighbors;
use crate::physics::fallingsand::data::element_grid::ElementGrid;
use crate::physics::fallingsand::mesh::coordinate_directory::CoordinateDir;
use crate::physics::fallingsand::util::vectors::JkVector;
use crate::physics::heat::components::{
    Compressability, Density, HeatEnergy, Length, SpecificHeat, ThermalConductivity,
    ThermodynamicTemperature, ROOM_TEMPERATURE_K,
};
use crate::physics::util::clock::Clock;
use bevy::render::color::Color;

/// Literally nothing
#[derive(Copy, Clone, Debug)]
pub struct Water {
    last_processed: Clock,
    heat: HeatEnergy,
}

impl Water {
    /// Create a new Water
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

impl Element for Water {
    fn get_type(&self) -> ElementType {
        ElementType::Water
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
    fn get_color(&self) -> Color {
        Color::BLUE
    }
    fn _process(
        &mut self,
        pos: JkVector,
        coord_dir: &CoordinateDir,
        target_chunk: &mut ElementGrid,
        element_grid_conv: &mut ElementGridConvolutionNeighbors,
        current_time: Clock,
    ) -> ElementTakeOptions {
        fluid_process(
            self,
            pos,
            coord_dir,
            target_chunk,
            element_grid_conv,
            current_time,
        )
    }
    fn box_clone(&self) -> Box<dyn Element> {
        Box::new(*self)
    }

    fn get_default_temperature(&self) -> ThermodynamicTemperature {
        ROOM_TEMPERATURE_K
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
        SpecificHeat(830.0 / 300.0)
    }

    fn get_thermal_conductivity(&self) -> ThermalConductivity {
        ThermalConductivity(1.0)
    }

    fn get_compressability(&self) -> Compressability {
        Compressability(0.0)
    }
}

#[cfg(test)]
mod test {
    mod heat {
        

        use crate::physics::{
            fallingsand::{
                elements::element::ElementType,
            },
            heat::{
                math::{PropogateHeat},
            },
        };

        /// Determines how fast the heat diffuses
        #[test]
        fn test_sink_diffuses_to_zero_speed() {
            PropogateHeat::test_heat_disipation_rate_in_space(1730, 1, ElementType::Water);
        }
    }
}
