//! Coordinates a tool accepts. Enshwick refuses positions below 0 on any axis
//! and above 1024 m (`terrain.md`); the map is 10240 m a side in enshwick's
//! `MapFrame` example. The 10240 edge is not checked live yet (decision 0001).

pub const EDGE: f64 = 10240.0;
pub const HEIGHT: f64 = 1024.0;

/// Whether `p` is a finite point inside the world.
#[must_use]
pub fn in_world(p: [f64; 3]) -> bool {
    let [x, y, z] = p;
    p.iter().all(|v| v.is_finite())
        && (0.0..=EDGE).contains(&x)
        && (0.0..=HEIGHT).contains(&y)
        && (0.0..=EDGE).contains(&z)
}

/// The box from `a` to `b` when both corners are in the world and no side is
/// longer than `max_side` metres.
#[must_use]
pub fn small_box(a: [f64; 3], b: [f64; 3], max_side: f64) -> bool {
    in_world(a) && in_world(b) && (0..3).all(|i| (a[i] - b[i]).abs() <= max_side)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_inside_pass() {
        assert!(in_world([0.0, 0.0, 0.0]));
        assert!(in_world([5120.0, 300.0, 5120.0]));
        assert!(in_world([EDGE, HEIGHT, EDGE]));
    }

    #[test]
    fn points_outside_or_not_numbers_fail() {
        for p in [
            [-0.1, 0.0, 0.0],
            [0.0, 1024.5, 0.0],
            [10241.0, 0.0, 0.0],
            [f64::NAN, 0.0, 0.0],
            [0.0, f64::INFINITY, 0.0],
            [1e9, 0.0, 1e9],
        ] {
            assert!(!in_world(p), "{p:?}");
        }
    }

    #[test]
    fn boxes_are_capped() {
        assert!(small_box([100.0, 10.0, 100.0], [164.0, 74.0, 164.0], 64.0));
        assert!(!small_box([100.0, 10.0, 100.0], [165.0, 10.0, 100.0], 64.0));
        assert!(!small_box([100.0, 10.0, 100.0], [-1.0, 10.0, 100.0], 64.0));
    }
}
