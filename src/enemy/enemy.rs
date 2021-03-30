use crate::colors::{COLOR_BG, RGBA};
use crate::draw::{BTerm, DrawWithFov, Fov, Point};
use crate::field::FieldPosition;
use crate::state::{State, Stepper, StepperStatus};

pub struct Enemy {
    draw_char: char,
    draw_color: RGBA,
    action_time: f64,
    pos: Point,
    clock: f64,
}

impl Enemy {
    pub fn new(draw_char: char, draw_color: RGBA, action_time: f64, pos: Point) -> Self {
        Self {
            clock: 0.0,
            pos,
            draw_char,
            draw_color,
            action_time,
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
