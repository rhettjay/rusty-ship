// Canonical AABB overlap test for the whole game.
//
// Every entity exposes its bounds as an (x, y, w, h) tuple; all collision
// checks go through this one function so the semantics (touching edges do
// NOT count as overlap) are defined exactly once.

pub type Rect = (f32, f32, f32, f32);

pub fn rects_overlap(a: Rect, b: Rect) -> bool {
    a.0 < b.0 + b.2 && a.0 + a.2 > b.0 && a.1 < b.1 + b.3 && a.1 + a.3 > b.1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlap() {
        assert!(rects_overlap(
            (0.0, 0.0, 10.0, 10.0),
            (5.0, 5.0, 10.0, 10.0)
        ));
    }

    #[test]
    fn test_no_overlap() {
        assert!(!rects_overlap(
            (0.0, 0.0, 10.0, 10.0),
            (20.0, 20.0, 10.0, 10.0)
        ));
    }

    #[test]
    fn test_contained() {
        assert!(rects_overlap((0.0, 0.0, 20.0, 20.0), (5.0, 5.0, 5.0, 5.0)));
    }

    #[test]
    fn test_edge_touching() {
        assert!(!rects_overlap(
            (0.0, 0.0, 10.0, 10.0),
            (10.0, 0.0, 10.0, 10.0)
        ));
    }

    #[test]
    fn test_negative_coords() {
        assert!(rects_overlap(
            (-5.0, -5.0, 10.0, 10.0),
            (0.0, 0.0, 10.0, 10.0)
        ));
    }

    #[test]
    fn test_zero_size() {
        assert!(!rects_overlap((5.0, 5.0, 0.0, 0.0), (5.0, 5.0, 10.0, 10.0)));
    }
}
