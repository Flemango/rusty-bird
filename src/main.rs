
use ggez::event::{self, EventHandler, MouseButton};
use ggez::glam::*;
use ggez::graphics::{self, Color, TextAlign, TextLayout};
use ggez::conf;
use ggez::{Context, ContextBuilder, GameResult};
use ggez::input::keyboard::KeyInput;
use ggez::input::keyboard::KeyCode;
use ggez::winit::dpi::PhysicalSize;
use ggez::conf::FullscreenType::*;

use winapi::um::winuser::*;

use std::env;
use std::path;
use std::cell::RefCell;
use std::rc::Rc;

mod bird;
use bird::Bird;
use bird::initiate_player;
use bird::draw_player;

mod pipe;
use pipe::Pipe;
use pipe::generate_pipe;

mod audio;
use audio::Player;

const INIT_SCREEN_WIDTH: f32 = 800.;
const INIT_SCREEN_HEIGHT: f32 = 600.;
static mut SCREEN_WIDTH: f32 = INIT_SCREEN_WIDTH;
static mut SCREEN_HEIGHT: f32 = INIT_SCREEN_WIDTH;
pub const SCALE: f32 = 3.0;
const DESIRED_FPS: u32 = 60;

const GRAVITY: f32 = 0.6;
const MAX_VELOCITY:f32 = 16.0;
const JUMP_VELOCITY: f32 = -13.5;
const JUMP_COOLDOWN: f32 = 0.25;

const PIPE_SPEED: f32 = 4.0;
const PIPE_GAP: f32 = 200.0;
const PIPE_WIDTH: f32 = 86.0;
const PIPE_FREQ: f32 = 96.0;

pub struct MuteBtn {
    sprite1: graphics::Image,
    sprite2: graphics::Image,
    position: Vec2,
    size: Vec2,
}

impl MuteBtn {
    fn new(ctx: &mut Context) -> Self {
        let sprite1 = graphics::Image::from_path(ctx, "/btn.png").unwrap();
        let sprite2 = graphics::Image::from_path(ctx, "/btn-muted.png").unwrap();
        //let sprite2 = graphics::Image::from_path(ctx, "/btn-muted.png").unwrap();

        let w: f32 = sprite1.width() as f32;
        let h: f32 = sprite1.height() as f32;

        //let mut sprite = Vec::new();
        //sprite.push(sprite);
        //sprite.push(sprite);

        let position = Vec2::new(16., 16.);

        let mutebtn = Self {
            sprite1,
            sprite2,
            position,
            size: Vec2::new(w, h),
        };

        return mutebtn;
    }
}

struct Mouse {
    pos_x: f32,
    pos_y: f32,
    mouse_down: bool,
}

struct MyGame {
    bird: Bird,
    pipes: Vec<Pipe>,
    score: u32,
    start: bool,
    game_over: bool,
    pipe_spawn_timer: f32,
    jump_cd: f32,
    bg: graphics::Image,
    sound_player: audio::Player,
    window_size: PhysicalSize<u32>,
    fullscreen: bool, 
    mute_btn: MuteBtn,
    mouse: Mouse,
}

//------------------------------------------------------------------
// create event ----------------------------------------------------
// -----------------------------------------------------------------
impl MyGame {
    fn new(ctx: &mut Context) -> GameResult<MyGame> {

        //init player
        let spr_bird = graphics::Image::from_path(ctx, "/bird.png")?;
        let bird: Bird = initiate_player(spr_bird);

        ctx.gfx.add_font(
            "font_regular",
            graphics::FontData::from_path(ctx, "/font.ttf")?,
        );
        ctx.gfx.add_font(
            "font_logo",
            graphics::FontData::from_path(ctx, "/logo.ttf")?,
        );

        let sound_player: Player = Player::new(ctx);
        let bg = graphics::Image::from_path(ctx, "/bg.png")?;

        let spr_btn = graphics::Image::from_path(ctx, "/btn.png")?;
        let spr_btnmute = graphics::Image::from_path(ctx, "/btn-muted.png")?;
        let mute_btn = MuteBtn::new(ctx);

        let mouse = Mouse {
            pos_x: 100.0,
            pos_y: 100.0,
            mouse_down: false,
        };

        let s = MyGame {
            bird,
            pipes: Vec::new(),
            score: 0,
            start: false,
            game_over: false,
            pipe_spawn_timer: 0.0,
            jump_cd: 0.0,
            bg,
            sound_player,
            window_size: PhysicalSize::new(INIT_SCREEN_WIDTH as u32, INIT_SCREEN_HEIGHT as u32),
            fullscreen: false,
            mute_btn,
            mouse,
        };

        Ok(s)
    }

    fn player_jump(&mut self, ctx: &mut Context) {
        if self.jump_cd <= 0.0 && !self.game_over {
            self.bird.velocity = Vec2::new(0.0, JUMP_VELOCITY);
            self.jump_cd = JUMP_COOLDOWN;

            self.sound_player.jump(ctx);
        }
    }

    fn player_update(&mut self) {
        if self.bird.velocity.y < MAX_VELOCITY {
            self.bird.velocity += Vec2::new(0.0, GRAVITY);
        }
        self.bird.position += self.bird.velocity;

        if self.bird.position.y>(unsafe{SCREEN_HEIGHT}) && !self.game_over {
            self.game_over = true;
        } 

        self.jump_cd -= 1./DESIRED_FPS as f32;

        // update rect hitboxes to the position
        self.bird.hitbox.x = self.bird.position.x - self.bird.sprite.width() as f32/2.;
        self.bird.hitbox.y = self.bird.position.y - self.bird.sprite.height() as f32/2.;
    }

    fn pipe_update(&mut self, ctx: &mut Context) -> GameResult {
        for pipe in &mut self.pipes {
            pipe.position.x -= PIPE_SPEED;
        }

        // spawn new pipes
        self.pipe_spawn_timer -= 1.;
        if self.pipe_spawn_timer <= 0. {
            let pipe = generate_pipe(ctx)?;
            self.pipes.push(pipe);

            self.pipe_spawn_timer = PIPE_FREQ;
        }

        // remove offscreen pipes
        self.pipes.retain(|pipe| pipe.position.x + PIPE_WIDTH > 0.0);

        for pipe in &mut self.pipes {
            // update rect hitboxes to the position
            pipe.hitbox_down.x = pipe.position.x;
            pipe.hitbox_down.y = pipe.position.y;
            pipe.hitbox_up.x = pipe.position.x;
            pipe.hitbox_up.y = pipe.position.z - pipe.sprite.height() as f32 * SCALE;

            // check collisions
            if !self.game_over && (self.bird.hitbox.overlaps(&pipe.hitbox_down) || self.bird.hitbox.overlaps(&pipe.hitbox_up)) {
                self.game_over = true;
                self.sound_player.ouch(ctx);
            }

            // add score when passing through the gap
            if !self.game_over && (self.bird.position.x == pipe.position.x) {
                self.score += 1;
                self.sound_player.score(ctx);
            }
        }
        Ok(())
    }
}

impl EventHandler for MyGame 
{
    //------------------------------------------------------------------
    // update event ----------------------------------------------------
    // -----------------------------------------------------------------
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while ctx.time.check_update_time(DESIRED_FPS) {
            if !self.start {

                self.bird.position.x = (unsafe{SCREEN_WIDTH})/4.;
                self.bird.position.y = (unsafe{SCREEN_HEIGHT})/2.;

            } else {
                // player movement handling
                self.player_update();

                // pipe handling
                self.pipe_update(ctx).ok();
            }

            //get realtime window size
            self.window_size = ctx.gfx.window().inner_size();
            unsafe {
                SCREEN_WIDTH = self.window_size.width as f32;
                SCREEN_HEIGHT = self.window_size.height as f32;
            }
        }
        Ok(())
    }

    //------------------------------------------------------------------
    // draw event ------------------------------------------------------
    // -----------------------------------------------------------------

    fn draw(&mut self, ctx: &mut Context) -> GameResult {

        let scale = Vec2::new(SCALE, SCALE);

        let mut canvas =  graphics::Canvas::from_frame(ctx, Color::from_rgb(182, 241, 255));
        canvas.set_sampler(graphics::Sampler::nearest_clamp());

        canvas.draw(
            &self.bg, graphics::DrawParam::new()
            .offset(Vec2::new(0.5, 0.45))
            .dest(Vec2::new(unsafe{SCREEN_HEIGHT/2.},unsafe{SCREEN_HEIGHT/2.}))
            .scale(scale),
        );

        let logo_text = graphics::Text::new("RUSTY BIRD")
            .set_font("font_logo")
            .set_scale(64.)
            .set_layout(TextLayout {
                h_align: TextAlign::Middle,
                v_align: TextAlign::Middle})
            .clone();

        let start_text = graphics::Text::new("Press Enter/Space to start!\nUse Space to jump.")
            .set_font("font_regular")
            .set_scale(24.)
            .set_layout(TextLayout {
                h_align: TextAlign::Middle,
                v_align: TextAlign::Middle})
            .clone();

        let score_str: String = format!("Score: {}", self.score);
        let score_text = graphics::Text::new(score_str)
            .set_font("font_regular")
            .set_scale(16.)
            .set_layout(TextLayout {
                h_align: TextAlign::Begin,
                v_align: TextAlign::Begin})
            .clone();

        let gameover_text = graphics::Text::new("Game Over.\nPress any key to close the game\n")
            .set_font("font_regular")
            .set_scale(24.)
            .set_layout(TextLayout {
                h_align: TextAlign::Middle,
                v_align: TextAlign::Middle})
            .clone();

        let scale = Vec2::new(SCALE, SCALE);
        let dst = self.bird.position;

        // draw bird
        let drawparams = draw_player(&mut self.bird);
        canvas.draw(
            &self.bird.sprite, drawparams
            .dest(dst)
            .scale(scale),
        );

        // draw pipes
        for pipe in &self.pipes {
            let dst1: Vec2 = Vec2::new(pipe.position.x, pipe.position.y);
            let dst2: Vec2 = Vec2::new(pipe.position.x, pipe.position.z);
            canvas.draw(
                &pipe.sprite,
                graphics::DrawParam::new()
                .dest(dst1)
                .scale(scale),
            );
            canvas.draw(
                &pipe.sprite,
                graphics::DrawParam::new()
                .dest(dst2)
                .scale(Vec2::new(SCALE,-SCALE)),
            );
        }

        if !self.start {
            canvas.draw(
                &start_text,
                graphics::DrawParam::from(vec2((unsafe{SCREEN_WIDTH})/2., (unsafe{SCREEN_HEIGHT/2.+SCREEN_HEIGHT/4.})))
                    .color(Color::from((255, 255, 255, 255))),
            );
            canvas.draw(
                &logo_text,
                graphics::DrawParam::from(vec2((unsafe{SCREEN_WIDTH})/2., (unsafe{SCREEN_HEIGHT})/3.))
                    .color(Color::from((255, 0, 77, 255))),
            );

            if !self.sound_player.muted {
                canvas.draw(
                &self.mute_btn.sprite1,
                graphics::DrawParam::new()
                .dest(Vec2::new(16.,16.))
                .scale(scale));
            } else {
                canvas.draw(
                    &self.mute_btn.sprite2,
                    graphics::DrawParam::new()
                    .dest(Vec2::new(16.,16.))
                    .scale(scale));
            }

        } else {
            canvas.draw(
                &score_text,
                graphics::DrawParam::from(vec2(16.,16.))
                .color(Color::from((255, 255, 255, 255))),
            );
        }

        if self.game_over {
            canvas.draw(
                &gameover_text,
                graphics::DrawParam::from(vec2((unsafe{SCREEN_WIDTH})/2., (unsafe{SCREEN_HEIGHT})/2.))
                    .color(Color::from((0, 0, 0, 255))),
            );
        }
        
        canvas.finish(ctx)
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        self.mouse.mouse_down = true;
        //self.sound_player.muted = !self.sound_player.muted;
        if !self.start {
            if self.mouse.pos_x >= 16. &&
            self.mouse.pos_x <= 64. &&
            self.mouse.pos_y >= 16. &&
            self.mouse.pos_y <= 64. {
                self.sound_player.muted = !self.sound_player.muted;
            }
        }
        // let btnx = self.mute_btn.position.x;
        // let btny = self.mute_btn.position.y;
        // let btnw = (self.mute_btn.position.x+self.mute_btn.size.x)*SCALE;
        // let btnh = (self.mute_btn.position.y+self.mute_btn.size.y)*SCALE;//self.mute_btn.size.y;

        // let testx = self.mouse.pos_x;
        // let test = self.sound_player.muted;
        // println!("Button x: {btnx}, y: {btny}, w: {btnw}, h: {btnh}");
        // println!("Mouse button pressed: {button:?}, x: {testx}, y: {y}");
        // println!("Muted: {test}");
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        x: f32,
        y: f32,
        xrel: f32,
        yrel: f32,
    ) -> GameResult {
        
            self.mouse.pos_x = x;
            self.mouse.pos_y = y;
        
        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult {
        self.mouse.mouse_down = false;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        input: KeyInput,
        _repeated: bool,
    ) -> GameResult {

        if self.game_over {
            ctx.request_quit();
        }

        match input.keycode {
            Some(KeyCode::Return) => {
                if !self.start {
                    self.start = true;
                    self.sound_player.begin(ctx);
                }
            }

            Some(KeyCode::F11) => {
                if !self.fullscreen {
                    unsafe {
                        let screen_width = GetSystemMetrics(SM_CXSCREEN);
                        let screen_height = GetSystemMetrics(SM_CYSCREEN);
                        ctx.gfx.set_drawable_size(screen_width as f32, screen_height as f32).ok();
                    }
                    ctx.gfx.set_fullscreen(True).ok();
                }
                else {
                    ctx.gfx.set_fullscreen(Windowed).ok();
                    conf::WindowMode::default();
                    ctx.gfx.set_drawable_size(INIT_SCREEN_WIDTH, INIT_SCREEN_HEIGHT).ok();
                }
                self.fullscreen = !self.fullscreen;
            }
            
            Some(KeyCode::Space) => {
                if !self.start {
                    self.start = true;
                    self.sound_player.begin(ctx);
                } else {
                    self.player_jump(ctx);
                }
            }
            Some(KeyCode::Escape) => ctx.request_quit(),
            _ => (), // Do nothing
        }
        Ok(())
    }

}

pub fn main() -> GameResult {

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ContextBuilder::new("rustybird", "ggez")
        .window_setup(conf::WindowSetup::default().title("Rusty bird!"))
        .window_mode(conf::WindowMode::default().dimensions(INIT_SCREEN_WIDTH, INIT_SCREEN_HEIGHT).resizable(true))
        .add_resource_path(resource_dir);

        let (mut ctx, event_loop) = cb.build()?;

        let game = MyGame::new(&mut ctx)?;

    // Run!
    event::run(ctx, event_loop, game);
}