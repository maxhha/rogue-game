use crate::colors::{COLOR_BG, COLOR_PLAYER};
use crate::draw::{BTerm, Draw, Point};
use crate::field::FieldPosition;
use crate::state::{State, Stepper};
use bracket_terminal::prelude::VirtualKeyCode;

const MOVE_CLOCK: f64 = 1.0;

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

    fn action(&mut self, world: &State, direction: (i32, i32)) {
        let next_pos = self.pos + Point::from(direction);

        if !world.field.is_wall(next_pos.x, next_pos.y) {
            self.pos = next_pos
        }
    }

    fn process_key(&mut self, world: &State, key: VirtualKeyCode) {
        match key {
            VirtualKeyCode::W => self.action(world, (0, -1)),
            VirtualKeyCode::A => self.action(world, (-1, 0)),
            VirtualKeyCode::S => self.action(world, (0, 1)),
            VirtualKeyCode::D => self.action(world, (1, 0)),
            _ => {}
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

    fn process(&mut self, world: &State, ctx: &BTerm) {
        match ctx.key {
            None => {}
            Some(key) => self.process_key(world, key),
        }
    }
}
