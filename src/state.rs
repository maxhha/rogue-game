use bracket_terminal::prelude::{BTerm, GameState};

pub struct State {}

impl Default for State {
    fn default() -> Self {
        State {}
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
    }
}
