use crate::colors::{COLOR_BG, COLOR_PLAYER};
use crate::draw::{BTerm, Draw, Point};
use crate::field::FieldPosition;
use crate::state::{State, Stepper, StepperStatus};
use bracket_terminal::prelude::VirtualKeyCode;

const MOVE_TIME: f64 = 1.0;

pub struct Player {
    pos: Point,
    clock: f64,
    pub view_radius: i32,
}

impl Player {
    pub fn new(pos: Point, view_radius: i32) -> Player {
        Player {
            pos,
            clock: 0.0,
            view_radius,
        }
    }

    fn action(&mut self, world: &State, direction: (i32, i32)) -> StepperStatus {
        let next_pos = self.pos + Point::from(direction);

        if world.field.is_wall(next_pos.x, next_pos.y) {
            return StepperStatus::Pending;
        }

        self.pos = next_pos;
        self.clock += MOVE_TIME;

        StepperStatus::Finished
    }

    fn process_key(&mut self, world: &State, key: VirtualKeyCode) -> StepperStatus {
        match key {
            VirtualKeyCode::W => self.action(world, (0, -1)),
            VirtualKeyCode::A => self.action(world, (-1, 0)),
            VirtualKeyCode::S => self.action(world, (0, 1)),
            VirtualKeyCode::D => self.action(world, (1, 0)),
            _ => StepperStatus::Pending,
        }
    }
}

impl FieldPosition for Player {
    fn pos(&self) -> Point {
        self.pos
    }
}

impl Draw for Player {
    fn draw(&self, ctx: &mut BTerm, pos: Point) {
        ctx.print_color(pos.x, pos.y, COLOR_PLAYER, COLOR_BG, "@")
    }
}

impl Stepper for Player {
    fn clock(&self) -> f64 {
        self.clock
    }

    fn process(&mut self, world: &State, ctx: &BTerm) -> StepperStatus {
        match ctx.key {
            None => StepperStatus::Finished,
            Some(key) => self.process_key(world, key),
        }
    }
}
