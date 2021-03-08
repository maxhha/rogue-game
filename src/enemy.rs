use crate::colors::{COLOR_BG, COLOR_RAT};
use crate::draw::{BTerm, DrawWithFov, Fov, Point};
use crate::field::FieldPosition;

pub trait Enemy: FieldPosition + DrawWithFov {}

pub struct Rat {
    pos: Point,
}

impl Rat {
    pub fn new(pos: Point) -> Rat {
        Rat { pos }
    }
}

impl FieldPosition for Rat {
    fn pos(&self) -> Point {
        self.pos
    }
}

impl DrawWithFov for Rat {
    fn draw_with_fov(&self, ctx: &mut BTerm, fov: &Fov, pos: Point, fov_pos: Point) {
        if fov.contains(&fov_pos) {
            ctx.print_color(pos.x, pos.y, COLOR_RAT, COLOR_BG, "r")
        }
    }
}

impl Enemy for Rat {}
