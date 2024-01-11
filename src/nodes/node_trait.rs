use crate::physics::util::vectors::WorldCoord;

pub trait NodeTrait {
    fn get_world_coord(&self) -> WorldCoord;
}
