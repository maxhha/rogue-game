use crate::colors::{COLOR_BG, COLOR_RAT, RGBA};
use crate::draw::{BTerm, DrawWithFov, Fov, Point};
use crate::field::FieldPosition;
use crate::state::{State, Stepper, StepperStatus};

pub struct Enemy {
    draw_char: &'static str,
    draw_color: RGBA,
    action_time: f64,
    pos: Point,
    clock: f64,
}

pub struct EnemyBuilder {
    draw_char: &'static str,
    draw_color: RGBA,
    pos: Point,
    action_time: f64,
}

impl EnemyBuilder {
    pub fn rat() -> Self {
        Self {
            draw_char: "r",
            draw_color: COLOR_RAT,
            pos: Point::zero(),
            action_time: 0.5,
        }
    }

    pub fn pos(mut self, pos: Point) -> Self {
        self.pos = pos;
        self
    }

    pub fn build(self) -> Enemy {
        Enemy {
            clock: 0.0,
            pos: self.pos,
            draw_char: self.draw_char,
            draw_color: self.draw_color,
            action_time: self.action_time,
        }
    }
}

impl FieldPosition for Enemy {
    fn pos(&self) -> Point {
        self.pos
    }
}

impl DrawWithFov for Enemy {
    fn draw_with_fov(&self, ctx: &mut BTerm, fov: &Fov, pos: Point, fov_pos: Point) {
        if fov.contains(&fov_pos) {
            ctx.print_color(pos.x, pos.y, self.draw_color, COLOR_BG, self.draw_char)
        }
    }
}

impl Stepper for Enemy {
    fn clock(&self) -> f64 {
        self.clock
    }

    fn process(&mut self, _world: &State, _ctx: &BTerm) -> StepperStatus {
        self.clock += self.action_time;
        StepperStatus::Finished
    }
}
