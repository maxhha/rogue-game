pub use bracket_pathfinding::prelude::Point;
pub use bracket_terminal::prelude::BTerm;
use std::collections::HashSet;

pub type Fov = HashSet<Point>;

pub trait Draw {
    fn draw(&self, ctx: &mut BTerm, pos: Point);
}

pub trait DrawWithFov {
    fn draw_with_fov(&self, ctx: &mut BTerm, fov: &Fov, pos: Point, fov_pos: Point);
}
