use crate::colors::{COLOR_BG, COLOR_PLAYER};
use crate::draw::{BTerm, Draw, Point};
use crate::field::FieldPosition;
use crate::state::{State, Stepper};

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
