use ggez::{audio::Source,Context};
use ggez::audio::SoundSource;
use rand::distributions::OpenClosed01;
use rand::{thread_rng, Rng};
use Source as src;

pub struct Player {
    pub begin_snd: Source,
    pub jump_snd: Source,
    pub score_snd: Source,
    pub ouch_snd: Source,
}

impl Player {
    pub fn new(ctx: &mut Context) -> Self {
        let begin_snd: Source = src::new(ctx, "/begin_game.wav").unwrap();
        let jump_snd: Source = src::new(ctx, "/jump.wav").unwrap();
        let score_snd: Source =  src::new(ctx, "/score_point.wav").unwrap();
        let ouch_snd: Source =  src::new(ctx, "/ouch.wav").unwrap();

        Self {
            begin_snd,
            jump_snd,
            score_snd,
            ouch_snd,
        }
    }

    pub fn begin(&mut self, ctx: &mut Context) {
        self.begin_snd.play_detached(ctx).ok();
    }

    pub fn jump(&mut self, ctx: &mut Context) {
        let pitch: f32 = thread_rng().sample(OpenClosed01);
        self.score_snd.set_pitch(1.0 - pitch);

        self.jump_snd.play_detached(ctx).ok();
    }

    pub fn ouch(&mut self, ctx: &mut Context) {
        self.ouch_snd.play_detached(ctx).ok();
    }

    pub fn score(&mut self, ctx: &mut Context) {
        let pitch: f32 = thread_rng().sample(OpenClosed01);
        self.score_snd.set_pitch(1.0 + pitch);

        self.score_snd.play_detached(ctx).ok();
    }
}