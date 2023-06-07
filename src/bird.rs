use ggez::graphics::Image;
use ggez::graphics::Rect;
use ggez::graphics::DrawParam;
use ggez::glam::Vec2;

use nalgebra as na;

use crate::SCALE;
use crate::SCREEN_HEIGHT;

pub struct Bird {
    pub position: Vec2,
    pub velocity: Vec2,
    pub sprite: Image,
    pub hitbox: Rect,
    pub jump_cd: f32,
}

pub fn initiate_player(sprite_path: Image) -> Bird {
    let x: f32 = 200.0;
    let y: f32 = (unsafe{SCREEN_HEIGHT})/2.-128.;

    let spr_bird: Image = sprite_path;//Image::from_path(ctx, "/bird.png")?;

    let hb_x: f32 = (x + spr_bird.width() as f32) /2.;
    let hb_y: f32 = (y + spr_bird.height() as f32) /2.;
    let hitbox: Rect = Rect::new(hb_x, hb_y, spr_bird.width() as f32 * SCALE * 0.9, spr_bird.height() as f32 * SCALE * 0.9);

    let bird = Bird {
        position: Vec2::new(x, y),
        velocity: Vec2::ZERO,
        sprite: spr_bird,
        hitbox: hitbox,
        jump_cd: 0.0,
    };

    return bird;
}

pub fn draw_player(player: &mut Bird) -> DrawParam {

    let dst: Vec2 = player.position;
        let angle: f32 = rescale_range(player.velocity.y, -7.0, 7.0, -0.6, 0.6);
        let drawparams = DrawParam::new()
            .dest(dst)
            .rotation(angle)
            .offset(Vec2::new(0.5, 0.5));

    return drawparams;
}

fn rescale_range(value: f32, old_min: f32, old_max: f32, new_min: f32, new_max: f32) -> f32 {
    use na::clamp;
    let old_range = old_max - old_min;
    let new_range = new_max - new_min;
    (((clamp(value, old_min, old_max) - old_min) * new_range) / old_range) + new_min
}
