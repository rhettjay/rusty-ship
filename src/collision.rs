use macroquad::prelude::*;

#[derive(Clone, Copy)]
pub struct Hitbox {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Hitbox {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }
}

pub fn check_collision(a: &Hitbox, b: &Hitbox) -> bool {
    a.x < b.x + b.w &&
    a.x + a.w > b.x &&
    a.y < b.y + b.h &&
    a.y + a.h > b.y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlap() {
        let a = Hitbox::new(0.0, 0.0, 10.0, 10.0);
        let b = Hitbox::new(5.0, 5.0, 10.0, 10.0);
        assert!(check_collision(&a, &b));
    }

    #[test]
    fn test_no_overlap() {
        let a = Hitbox::new(0.0, 0.0, 10.0, 10.0);
        let b = Hitbox::new(20.0, 20.0, 10.0, 10.0);
        assert!(!check_collision(&a, &b));
    }

    #[test]
    fn test_contained() {
        let a = Hitbox::new(0.0, 0.0, 20.0, 20.0);
        let b = Hitbox::new(5.0, 5.0, 5.0, 5.0);
        assert!(check_collision(&a, &b));
    }

    #[test]
    fn test_edge_touching() {
        let a = Hitbox::new(0.0, 0.0, 10.0, 10.0);
        let b = Hitbox::new(10.0, 0.0, 10.0, 10.0);
        assert!(!check_collision(&a, &b));
    }

    #[test]
    fn test_negative_coords() {
        let a = Hitbox::new(-5.0, -5.0, 10.0, 10.0);
        let b = Hitbox::new(0.0, 0.0, 10.0, 10.0);
        assert!(check_collision(&a, &b));
    }

    #[test]
    fn test_zero_size() {
        let a = Hitbox::new(5.0, 5.0, 0.0, 0.0);
        let b = Hitbox::new(5.0, 5.0, 10.0, 10.0);
        assert!(!check_collision(&a, &b));
    }
}