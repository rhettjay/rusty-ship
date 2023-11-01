use macroquad::{prelud::*, audio::{load_sound_from_bytes}};
use logic::game_logic;
struct ShipState  {
    player: graphics::Image,
    player_pos: mint::Point2<f32>,
}

impl ShipState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let player = graphics::Image::new(ctx, "/player.png")?;
        Ok(MainState {
            player,
            player_pos: mint::Point2 { x: 400.0, y: 300.0 },
        })
    }
}

impl event::EventHandler for ShipState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // Update game logic here
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());

        let dest_rect = graphics::Rect::new(self.player_pos.x, self.player_pos.y, 32.0, 32.0);
        graphics::draw(ctx, &self.player, (self.player_pos, dest_rect))?;

        graphics::present(ctx)?;
        Ok(())
    }
}

for bullet in player_bullets.iter() {
    for enemy in enemies.iter() {
        if check_collision(&bullet.hitbox, &enemy.hitbox) {
            // Handle the collision (e.g., remove bullet and enemy, update score).
        }
    }
}


fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("Rust Ship","w4rg4m35");
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut ShipState::new(ctx)?;
    event::run(ctx, event_loop, state)
}


