use crate::field::Field;
use bracket_terminal::prelude::{BTerm, GameState};

pub struct State {
    field: Field,
}

impl State {
    pub fn new() -> Self {
        State {
            field: Field::new(80, 25),
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        self.field.render_all(ctx);
    }
}
