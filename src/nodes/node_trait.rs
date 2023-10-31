use mint::Point2;

pub trait NodeTrait {
    fn get_world_coord(&self) -> Point2<f32>;
}
