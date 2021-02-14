use crate::draw::{Draw, DrawWithFov};
use crate::field::{Field, FieldPosition};
use crate::player::Player;

use bracket_pathfinding::prelude::*;
use bracket_terminal::prelude::{BTerm, GameState};
use std::collections::HashSet;

pub struct State {
    field: Field,
    fov: HashSet<Point>,
    player: Player,
}

impl State {
    pub fn new() -> Self {
        let field = Field::cave(80, 25, 0.6, 1);
        let player = Player::new(Point::new(40, 12), 8);
        let fov = field_of_view_set(player.pos(), player.view_radius, &field);

        State { field, fov, player }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        self.field
            .draw_with_fov(ctx, &self.fov, Point::zero(), Point::zero());

        self.player.draw(ctx, self.player.pos());
    }
}

pub trait Stepper {
    fn clock(&self) -> f64;
    fn process(&mut self, wld: &mut State);
}
