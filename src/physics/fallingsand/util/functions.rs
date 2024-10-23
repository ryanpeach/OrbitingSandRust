//! Mostly math functions
use bevy::math::Vec2;
use conv::ValueFrom;

/// A modulo that works for negative numbers
#[must_use]
pub fn modulo(x: isize, y: u32) -> u32 {
    let y_isize = isize::value_from(y).expect("u32 should always fit in isize on a 64-bit system.");
    u32::value_from(((x % y_isize) + y_isize) % y_isize)
        .expect("Since y is a u32, this will always fit in a u32")
}

/// Finds a point halfway between two points
#[must_use]
pub fn interpolate_points(p1: &Vec2, p2: &Vec2) -> Vec2 {
    Vec2::new((p1.x + p2.x) * 0.5, (p1.y + p2.y) * 0.5)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::unwrap_used,
        clippy::panic,
        clippy::float_cmp
    )]
    use super::*;

    #[test]
    fn test_interpolate_points() {
        let p1 = Vec2::new(0.0, 0.0);
        let p2 = Vec2::new(2.0, 2.0);
        let midpoint = interpolate_points(&p1, &p2);

        assert_eq!(midpoint.x, 1.0);
        assert_eq!(midpoint.y, 1.0);

        let p3 = Vec2::new(-2.0, -1.0);
        let p4 = Vec2::new(2.0, 3.0);
        let midpoint2 = interpolate_points(&p3, &p4);

        assert_eq!(midpoint2.x, 0.0);
        assert_eq!(midpoint2.y, 1.0);
    }
}
