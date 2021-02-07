use crate::field::Field;
use bracket_pathfinding::prelude::*;
use bracket_terminal::prelude::{BTerm, GameState};
use std::collections::HashSet;

pub struct State {
    field: Field,
    fov: HashSet<Point>,
}

impl State {
    pub fn new() -> Self {
        let mut field = Field::cave(80, 25, 0.6, 1);

        let fov = field_of_view_set(Point::new(40, 12), 8, &field);

        State { field, fov }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        self.field.render_with_fov(ctx, &self.fov);
    }
}
