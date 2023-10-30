use ggegui::{
    egui::{self, Ui},
    Gui,
};
use ggez::{mint::Point2, Context};
use mint::Vector2;

use crate::{
    nodes::{camera::cam::Camera, celestial::Celestial},
    physics::{
        fallingsand::{
            elements::element::ElementType,
            util::vectors::{ChunkIjkVector, IjkVector, JkVector},
        },
        util::vectors::RelXyPoint,
    },
};

use super::gui_trait::WindowTrait;

pub struct CursorTooltip {
    world_coords: Point2<f32>,
    screen_coords: Point2<f32>,
    camera_zoom: Vector2<f32>,
    screen_size: Vector2<f32>,
    ijk_coords: IjkVector,
    chunk_coords: (ChunkIjkVector, JkVector),
    element_type: ElementType,
    gui: Gui,
}

impl CursorTooltip {
    pub fn new(ctx: &Context, camera: &Camera) -> Self {
        Self {
            camera_zoom: Vector2 { x: 1.0, y: 1.0 },
            ijk_coords: IjkVector::new(0, 0, 0),
            chunk_coords: (ChunkIjkVector::new(0, 0, 0), JkVector::new(0, 0)),
            element_type: ElementType::Vacuum,
            screen_coords: Point2 { x: 0.0, y: 0.0 },
            world_coords: Point2 { x: 0.0, y: 0.0 },
            screen_size: camera.screen_size,
            gui: Gui::new(ctx),
        }
    }

    pub fn update(&mut self, camera: &Camera, celestial: &Celestial) {
        self.camera_zoom = camera.get_zoom();
        let coordinate_dir = celestial.get_element_dir().get_coordinate_dir();
        self.world_coords = camera.screen_to_world_coords(self.screen_coords);
        self.ijk_coords = {
            match coordinate_dir.rel_pos_to_cell_idx(RelXyPoint(self.world_coords.into())) {
                Ok(coords) => coords,
                Err(coords) => coords,
            }
        };
        self.chunk_coords = coordinate_dir.cell_idx_to_chunk_idx(self.ijk_coords);
        self.element_type = celestial
            .get_element_dir()
            .get_element(self.ijk_coords)
            .get_type();
    }
}

impl WindowTrait for CursorTooltip {
    fn get_offset(&self) -> Point2<f32> {
        self.screen_coords
    }

    fn set_offset(&mut self, screen_coords: Point2<f32>) {
        if screen_coords.x > 0.
            && screen_coords.y > 0.
            && screen_coords.x < self.screen_size.x - 100.
            && screen_coords.y < self.screen_size.y - 100.
        {
            self.screen_coords = screen_coords;
        }
    }

    fn get_gui(&self) -> &Gui {
        &self.gui
    }

    fn get_gui_mut(&mut self) -> &mut Gui {
        &mut self.gui
    }

    fn get_alignment(&self) -> egui::Align2 {
        egui::Align2::LEFT_TOP
    }

    fn get_title(&self) -> &str {
        "Cursor Tooltip"
    }

    fn window(&mut self, ui: &mut Ui) {
        ui.label(format!("zoom: {:?}", self.camera_zoom));
        ui.label(format!(
            "IjkCoord: ({}, {}, {})",
            self.ijk_coords.i, self.ijk_coords.j, self.ijk_coords.k
        ));
        ui.label(format!(
            "ChunkIjkCoord: ({}, {}, {})",
            self.chunk_coords.0.i, self.chunk_coords.0.j, self.chunk_coords.0.k
        ));
        ui.label(format!(
            "JkCoord: ({}, {})",
            self.chunk_coords.1.j, self.chunk_coords.1.k
        ));
        ui.label(format!("Type: {:?}", self.element_type));
    }
}
