use ggez::glam::Vec2;

/// A modulo that works for negative numbers
pub fn modulo(x: isize, y: usize) -> usize {
    let y_isize = y as isize;
    (((x % y_isize) + y_isize) % y_isize) as usize
}

/// Tests if a number is a power of 2
/// I found it's important that some values are powers of two in order to enable grid_iter to work
pub fn is_pow_2(n: usize) -> bool {
    n != 0 && (n & (n - 1)) == 0
}

/// Tests if a step is valid for a grid_iter
/// A valid step is 1, len - 1, or a factor of len - 1
/// We convert things less than 1 to 1, or things greater than len - 1 to len - 1
pub fn valid_step(len: usize, step: usize) -> bool {
    step <= 1 || step >= len - 1 || (len - 1) % step == 0
}

/// Finds a point halfway between two points
pub fn interpolate_points(p1: &Vec2, p2: &Vec2) -> Vec2 {
    Vec2::new((p1.x + p2.x) * 0.5, (p1.y + p2.y) * 0.5)
}

/// This is like the "skip" method but it always keeps the first and last item
/// If it is larger than the number of items, it will just return the first and last item
/// If the step is not a multiple of the number of items, it will round down to the previous multiple
pub fn grid_iter(start: usize, end: usize, step: usize) -> Vec<usize> {
    let len = end - start;
    if len <= 1 {
        // Return [0]
        return vec![start];
    }
    if step >= len {
        return vec![start, end - 1];
    }
    debug_assert_ne!(step, 0, "Step should not be 0.");

    debug_assert!(
        valid_step(len, step),
        "Step should be 1, len - 1, or a factor of len - 1. len: {}, step: {}",
        len,
        step
    );

    let start_item = start;
    let end_item = end - 1;

    let mut out = Vec::new();
    out.push(start_item);
    for i in (start_item + step..end_item).step_by(step) {
        if i % step == 0 && i != 0 && i != len - 1 {
            out.push(i);
        }
    }
    out.push(end_item);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_element() {
        let v: Vec<_> = grid_iter(0, 1, 16);
        assert_eq!(v, vec![0]);
    }

    #[test]
    fn test_two_elements() {
        let v: Vec<_> = grid_iter(0, 2, 16);
        assert_eq!(v, vec![0, 1]);
    }

    #[test]
    fn test_basic() {
        let v: Vec<_> = grid_iter(0, 11, 2);
        assert_eq!(v, vec![0, 2, 4, 6, 8, 10]);
    }

    #[test]
    fn test_step_one() {
        let v: Vec<_> = grid_iter(0, 11, 1);
        assert_eq!(v, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    /// At a large step size, we should just get the first and last elements
    #[test]
    fn test_large_step() {
        let v: Vec<_> = grid_iter(0, 10, 20);
        assert_eq!(v, vec![0, 9]);
    }

    #[test]
    fn test_basic_5() {
        let v: Vec<_> = grid_iter(0, 5, 2);
        assert_eq!(v, vec![0, 2, 4]);
    }

    /// In this case, because three doesnt work, we automatically round down to 2
    #[test]
    fn test_round_7() {
        let v: Vec<_> = grid_iter(0, 7, 3);
        assert_eq!(v, vec![0, 3, 6]);
    }

    #[test]
    fn test_is_pow_2() {
        assert!(is_pow_2(1));
        assert!(is_pow_2(2));
        assert!(is_pow_2(4));
        assert!(is_pow_2(8));
        assert!(!is_pow_2(0));
        assert!(!is_pow_2(3));
        assert!(!is_pow_2(6));
    }

    #[test]
    fn test_valid_step() {
        // Tests for len = 10
        assert!(valid_step(10, 1)); // 1 is valid for any len
        assert!(valid_step(10, 9)); // len - 1 is valid for any len
        assert!(valid_step(10, 3)); // 3 is a factor of len - 1
        assert!(!valid_step(10, 2)); // 2 is not a factor of len - 1 and not within the valid range
        assert!(!valid_step(10, 8)); // 8 is not a factor of len - 1 and not within the valid range
    }

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
