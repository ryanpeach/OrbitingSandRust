use ggez::glam::Vec2;

/// This is like the "skip" method but it always keeps the first and last item
/// If it is larger than the number of items, it will just return the first and last item
/// If the step is not a multiple of the number of items, it will round down to the previous multiple
pub fn grid_iter(start: usize, end: usize, mut step: usize) -> Vec<usize> {
    let len = end - start;
    if len == 1 {
        // Return [0]
        return vec![start];
    }
    if step >= len {
        return vec![start, end - 1];
    }
    debug_assert_ne!(step, 0, "Step should not be 0.");

    fn valid_step(len: usize, step: usize) -> bool {
        step == 1 || step == len - 1 || (len - 1) % step == 0
    }

    while !valid_step(len, step) && step > 1 {
        step -= 1;
    }

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

/// Finds a point halfway between two points
pub fn interpolate_points(p1: &Vec2, p2: &Vec2) -> Vec2 {
    Vec2::new((p1.x + p2.x) * 0.5, (p1.y + p2.y) * 0.5)
}