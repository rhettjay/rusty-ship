fn check_collision(a: &Hitbox, b: &Hitbox) -> bool {
    let left_a = a.x;
    let right_a = a.x + a.width;
    let top_a = a.y;
    let bottom_a = a.y + a.height;

    let left_b = b.x;
    let right_b = b.x + b.width;
    let top_b = b.y;
    let bottom_b = b.y + b.height;

    // Check for collision by comparing the coordinates of the hitboxes.
    return right_a > left_b && left_a < right_b && bottom_a > top_b && top_a < bottom_b;
}

