use bevy_ecs::system::Resource;
use ggez::{
    glam::{Mat4, Vec2, Vec3},
    graphics::{DrawParam, Rect},
    mint::{Point2, Vector2},
    Context,
};

use crate::physics::util::vectors::{ScreenCoord, WorldCoord};

use super::transform::Transform;

#[derive(Resource, Debug, Clone, Copy)]
pub struct Camera {
    pub offset: Point2<f32>,
    pub rotation: f32,
    pub scale: Vector2<f32>,
    pub position: WorldCoord,
    pub screen_size: Vector2<f32>,
}

impl Camera {
    pub fn new<V>(screen_size: V) -> Self
    where
        V: Into<Vector2<f32>>,
    {
        let ss = screen_size.into();
        Camera {
            offset: Point2 {
                x: ss.x / 2.,
                y: ss.y / 2.,
            },
            rotation: 0.,
            screen_size: ss,
            scale: Vector2 { x: 1., y: 1. },
            position: WorldCoord(Vec2 { x: 0., y: 0. }),
        }
    }
    pub fn to_matrix(&self) -> Mat4 {
        let (sinr, cosr) = self.rotation.sin_cos();
        let m00 = cosr * self.scale.x;
        let m01 = -sinr * self.scale.y;
        let m10 = sinr * self.scale.x;
        let m11 = cosr * self.scale.y;
        let m03 = self.position.0.x * (-m00) - self.position.0.y * m01 + self.offset.x;
        let m13 = self.position.0.y * (-m11) - self.position.0.x * m10 + self.offset.y;

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
        self.offset.x = point.x * self.scale.x;
        self.offset.y = point.y * self.scale.y;
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
        self.position.0.x -= delta.x / self.scale.x;
        self.position.0.y -= delta.y / self.scale.y;
    }

    pub fn world_coord_bounding_box(&self) -> Rect {
        let pos0 = self.screen_to_world_coords(ScreenCoord(Vec2 { x: 0., y: 0. }));
        let pos1 = self.screen_to_world_coords(ScreenCoord(Vec2 {
            x: self.screen_size.x,
            y: self.screen_size.y,
        }));
        Rect::new(pos0.0.x, pos0.0.y, pos1.0.x - pos0.0.x, pos1.0.y - pos0.0.y)
    }

    pub fn get_zoom(&self) -> Vector2<f32> {
        self.scale
    }

    pub fn set_zoom<V>(&mut self, scale: V)
    where
        V: Into<Vector2<f32>>,
    {
        self.scale = scale.into();
    }

    pub fn zoom<V>(&mut self, factor: V)
    where
        V: Into<Vector2<f32>>,
    {
        let factor: Vector2<f32> = factor.into();
        self.scale.x *= factor.x;
        self.scale.y *= factor.y;
    }

    pub fn zoom_center<V>(&mut self, ctx: &Context, factor: V)
    where
        V: Into<Vector2<f32>>,
    {
        let factor: Vector2<f32> = factor.into();
        let screen_rect = ctx.gfx.drawable_size();
        let screen_center = ScreenCoord(Vec2 {
            x: screen_rect.0 / 2.0,
            y: screen_rect.1 / 2.0,
        });
        let world_center = self.screen_to_world_coords(screen_center);
        self.position.0.x = world_center.0.x - (world_center.0.x - self.position.0.x) / factor.x;
        self.position.0.y = world_center.0.y - (world_center.0.y - self.position.0.y) / factor.y;
        self.scale.x *= factor.x;
        self.scale.y *= factor.y;
    }

    pub fn zoom_at_screen_coords<P, V>(&mut self, point: P, factor: V)
    where
        P: Into<Point2<f32>>,
        V: Into<Vector2<f32>>,
    {
        let point = ScreenCoord(point.into().into());
        let factor: Vector2<f32> = factor.into();
        let world_center = self.screen_to_world_coords(point);
        self.position.0.x = world_center.0.x - (world_center.0.x - self.position.0.x) / factor.x;
        self.position.0.y = world_center.0.y - (world_center.0.y - self.position.0.y) / factor.y;
        self.scale.x *= factor.x;
        self.scale.y *= factor.y;
    }

    pub fn rotate(&mut self, angle: f32) {
        self.rotation += angle;
    }

    pub fn set_rotation(&mut self, angle: f32) {
        self.rotation = angle;
    }
}

impl From<Camera> for DrawParam {
    fn from(value: Camera) -> Self {
        DrawParam::default().transform(value.to_matrix())
    }
}
