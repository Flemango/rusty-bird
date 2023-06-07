use ggez::graphics::Image;
use ggez::graphics::Rect;
use ggez::glam::*;
use ggez::Context;
use ggez::GameResult;

use rand::Rng;

use crate::SCALE;
use crate::SCREEN_WIDTH;
use crate::SCREEN_HEIGHT;
use crate::PIPE_GAP;

pub struct Pipe {
    pub position: Vec3,
    pub sprite: Image,
    pub hitbox_down: Rect,
    pub hitbox_up: Rect
}

pub fn generate_pipe(ctx: &mut Context) -> GameResult<Pipe> {
    let mut rng = rand::thread_rng();
    let rand_num: f32 = rng.gen_range((86.)..=(unsafe{SCREEN_HEIGHT})/2.-86.);

    let x1: f32 = unsafe{SCREEN_WIDTH};
    let y1: f32 = unsafe{SCREEN_HEIGHT} - rand_num;
    let y2: f32 = y1 - PIPE_GAP;
    
    let sprite = Image::from_path(ctx, "/pipe.png")?;

    let hitbox_down = Rect::new(x1, y1, sprite.width() as f32 * SCALE, sprite.height() as f32 * SCALE);
    let hitbox_up = Rect::new(x1, y2, sprite.width() as f32 * SCALE, sprite.height() as f32 * SCALE);
    let pipe = Pipe {
        position: Vec3::new(x1, y1, y2),
        sprite,
        hitbox_down,
        hitbox_up,
    };

    Ok(pipe)
}
