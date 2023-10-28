use ggez::{
    glam::Mat4,
    graphics::{self, DrawParam},
    mint::{Point2, Vector2},
};

#[derive(Clone, Copy)]
pub struct Transform {
    pub dest: Point2<f32>,
    pub rotation: f32,
    pub scale: Vector2<f32>,
    pub offset: Point2<f32>,
}

impl Transform {
    pub fn to_matrix(&self) -> Mat4 {
        let offset = Point2::<f32> {
            x: self.offset.x / self.scale.x,
            y: self.offset.y / self.scale.y,
        };
        let (sinr, cosr) = self.rotation.sin_cos();
        let m00 = cosr * self.scale.x;
        let m01 = -sinr * self.scale.y;
        let m10 = sinr * self.scale.x;
        let m11 = cosr * self.scale.y;
        let m03 = offset.x * (-m00) - offset.y * m01 + self.dest.x;
        let m13 = offset.y * (-m11) - offset.x * m10 + self.dest.y;

        Mat4::from_cols_array(&[
            m00, m01, 0.0, m03, //
            m10, m11, 0.0, m13, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0, //
        ])
        .transpose()
    }

    pub fn apply_matrix(&self, parent_matrix: &Mat4) -> Mat4 {
        parent_matrix.mul_mat4(&self.to_matrix())
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            dest: Point2 { x: 0., y: 0. },
            rotation: 0.,
            scale: Vector2 { x: 1., y: 1. },
            offset: Point2 { x: 0., y: 0. },
        }
    }
}

impl From<Transform> for DrawParam {
    fn from(value: Transform) -> Self {
        DrawParam::default().transform(value.to_matrix())
    }
}

impl From<graphics::Transform> for Transform {
    fn from(value: graphics::Transform) -> Self {
        match value {
            graphics::Transform::Values {
                dest,
                rotation,
                scale,
                offset,
            } => Transform {
                dest,
                rotation,
                scale,
                offset,
            },
            graphics::Transform::Matrix(_) => {
                panic!("Cannot convert ggez::Transform to crate::Transform")
            }
        }
    }
}
