use bevy::ecs::system::Resource;
use glam::{Mat4, Vec2, Vec3};
use mint::{Point2, Vector2};

use crate::physics::util::vectors::{Rect, ScreenCoord, WorldCoord};

use super::transform::Transform;

#[derive(Default, Debug, Clone, Copy)]
pub struct FPS(pub f64);

#[derive(Debug, Clone, Copy)]
pub struct CameraZoom(pub Vec2);

#[derive(Debug, Clone, Copy)]
pub struct ScreenSize(pub Vec2);

#[derive(Debug, Clone, Copy)]
pub struct CameraRotation(pub f32);

#[derive(Resource, Debug, Clone, Copy)]
pub struct Camera {
    pub offset: ScreenCoord,
    pub rotation: CameraRotation,
    pub scale: CameraZoom,
    pub position: WorldCoord,
    pub screen_size: ScreenSize,
    pub fps: FPS,
}

impl Camera {
    pub fn new(screen_size: ScreenSize) -> Self {
        let ss = screen_size;
        Camera {
            offset: ScreenCoord(Vec2 {
                x: ss.0.x / 2.,
                y: ss.0.y / 2.,
            }),
            rotation: CameraRotation(0.),
            screen_size: ss,
            scale: CameraZoom(Vec2 { x: 1., y: 1. }),
            position: WorldCoord(Vec2 { x: 0., y: 0. }),
            fps: FPS(0.),
        }
    }
    pub fn to_matrix(&self) -> Mat4 {
        let (sinr, cosr) = self.rotation.0.sin_cos();
        let m00 = cosr * self.scale.0.x;
        let m01 = -sinr * self.scale.0.y;
        let m10 = sinr * self.scale.0.x;
        let m11 = cosr * self.scale.0.y;
        let m03 = self.position.0.x * (-m00) - self.position.0.y * m01 + self.offset.0.x;
        let m13 = self.position.0.y * (-m11) - self.position.0.x * m10 + self.offset.0.y;

        Mat4::from_cols_array(&[
            m00, m01, 0.0, m03, //
            m10, m11, 0.0, m13, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0, //
        ])
        .transpose()
    }

    pub fn apply_matrix<T>(&self, object: T) -> Mat4
    where
        T: Into<Transform>,
    {
        let object: Transform = object.into();

        self.to_matrix().mul_mat4(&object.to_matrix())
    }

    pub fn world_to_screen_coords(&self, point: WorldCoord) -> ScreenCoord {
        let point = Vec3::new(point.0.x, point.0.y, 0.);
        let screen_point = self.to_matrix().transform_point3(point);
        ScreenCoord(Vec2 {
            x: screen_point.x,
            y: screen_point.y,
        })
    }

    pub fn screen_to_world_coords(&self, point: ScreenCoord) -> WorldCoord {
        let inverse_matrix = self.to_matrix().inverse();
        let point = Vec3::new(point.0.x, point.0.y, 0.);
        let world_point = inverse_matrix.transform_point3(point);
        WorldCoord(Vec2 {
            x: world_point.x,
            y: world_point.y,
        })
    }

    pub fn set_position<P>(&mut self, point: P)
    where
        P: Into<Point2<f32>>,
    {
        let point: Point2<f32> = point.into();
        self.position.0.x = point.x;
        self.position.0.y = point.y;
    }

    pub fn set_offset<P>(&mut self, point: P)
    where
        P: Into<Point2<f32>>,
    {
        let point: Point2<f32> = point.into();
        self.offset.0.x = point.x * self.scale.0.x;
        self.offset.0.y = point.y * self.scale.0.y;
    }

    pub fn move_by_world_coords<P>(&mut self, delta: P)
    where
        P: Into<Point2<f32>>,
    {
        let delta: Point2<f32> = delta.into();
        self.position.0.x -= delta.x;
        self.position.0.y -= delta.y;
    }

    pub fn move_by_screen_coords<P>(&mut self, delta: P)
    where
        P: Into<Point2<f32>>,
    {
        let delta: Point2<f32> = delta.into();
        self.position.0.x -= delta.x / self.scale.0.x;
        self.position.0.y -= delta.y / self.scale.0.y;
    }

    pub fn world_coord_bounding_box(&self) -> Rect {
        let pos0 = self.screen_to_world_coords(ScreenCoord(Vec2 { x: 0., y: 0. }));
        let pos1 = self.screen_to_world_coords(ScreenCoord(Vec2 {
            x: self.screen_size.0.x,
            y: self.screen_size.0.y,
        }));
        Rect::new(pos0.0.x, pos0.0.y, pos1.0.x - pos0.0.x, pos1.0.y - pos0.0.y)
    }

    pub fn get_zoom(&self) -> CameraZoom {
        self.scale
    }

    pub fn set_zoom<V>(&mut self, scale: CameraZoom) {
        self.scale = scale;
    }

    pub fn zoom<V>(&mut self, factor: V)
    where
        V: Into<Vector2<f32>>,
    {
        let factor: Vector2<f32> = factor.into();
        self.scale.0.x *= factor.x;
        self.scale.0.y *= factor.y;
    }

    pub fn zoom_center<V>(&mut self, factor: V)
    where
        V: Into<Vector2<f32>>,
    {
        let factor: Vector2<f32> = factor.into();
        let screen_rect = self.screen_size;
        let screen_center = ScreenCoord(Vec2 {
            x: screen_rect.0.x / 2.0,
            y: screen_rect.0.y / 2.0,
        });
        let world_center = self.screen_to_world_coords(screen_center);
        self.position.0.x = world_center.0.x - (world_center.0.x - self.position.0.x) / factor.x;
        self.position.0.y = world_center.0.y - (world_center.0.y - self.position.0.y) / factor.y;
        self.scale.0.x *= factor.x;
        self.scale.0.y *= factor.y;
    }

    pub fn zoom_at_screen_coords<P, V>(&mut self, point: P, factor: V)
    where
        P: Into<Point2<f32>>,
        V: Into<Vector2<f32>>,
    {
        let point: Point2<f32> = point.into();
        let point = ScreenCoord(Vec2::new(point.x, point.y));
        let factor: Vector2<f32> = factor.into();
        let world_center = self.screen_to_world_coords(point);
        self.position.0.x = world_center.0.x - (world_center.0.x - self.position.0.x) / factor.x;
        self.position.0.y = world_center.0.y - (world_center.0.y - self.position.0.y) / factor.y;
        self.scale.0.x *= factor.x;
        self.scale.0.y *= factor.y;
    }

    pub fn rotate(&mut self, angle: f32) {
        self.rotation.0 += angle;
    }

    pub fn set_rotation(&mut self, angle: f32) {
        self.rotation.0 = angle;
    }

    // pub fn as_draw_param(&self) -> DrawParam {
    //     DrawParam::default().transform(self.to_matrix())
    // }
}

// impl From<Camera> for DrawParam {
//     fn from(value: Camera) -> Self {
//         DrawParam::default().transform(value.to_matrix())
//     }
// }
