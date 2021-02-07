use crate::field::Field;
use bracket_terminal::prelude::{BTerm, GameState};

pub struct State {
    field: Field,
}

impl State {
    pub fn new() -> Self {
        let mut field = Field::cave(40, 25, 0.6, 1);
        field.scale(2, 1);

        State { field }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        self.field.render_all(ctx);
    }
}
