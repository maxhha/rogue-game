use crate::draw::{Draw, DrawWithFov};
use crate::field::{Field, FieldPosition};
use crate::player::Player;

use bracket_pathfinding::prelude::*;
use bracket_terminal::prelude::{BTerm, GameState};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

pub struct State {
    pub field: Field,
    fov: HashSet<Point>,
    player: Rc<RefCell<Player>>,
    prev_player_pos: Point,
    current_stepper: Option<Rc<RefCell<dyn Stepper>>>,
}

impl State {
    pub fn new() -> Self {
        let field = Field::cave(80, 25, 0.6, 1);
        let player = Player::new(Point::new(40, 12), 8);
        let fov = field_of_view_set(player.pos(), player.view_radius, &field);

        State {
            field,
            fov,
            current_stepper: None,
            prev_player_pos: player.pos(),
            player: Rc::new(RefCell::new(player)),
        }
    }

    fn update_fov(&mut self) {
        let player = self.player.borrow();
        let pos = player.pos();

        self.fov = field_of_view_set(pos, player.view_radius, &self.field);
        self.prev_player_pos = pos;
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        if let None = self.current_stepper {
            let stepper = Rc::clone(&self.player);
            self.current_stepper = Some(stepper)
        }

        if let Some(stepper) = &self.current_stepper {
            stepper.borrow_mut().process(&self, ctx);
        }

        if self.prev_player_pos != self.player.borrow().pos() {
            self.update_fov();
        }

        self.field
            .draw_with_fov(ctx, &self.fov, Point::zero(), Point::zero());

        self.player.borrow().draw(ctx, self.player.borrow().pos());
    }
}

pub trait Stepper {
    fn clock(&self) -> f64;
    fn process(&mut self, world: &State, ctx: &BTerm);
}
