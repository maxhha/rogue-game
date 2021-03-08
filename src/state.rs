use crate::draw::{Draw, DrawWithFov};
use crate::enemy::{Enemy, Rat};
use crate::field::{Field, FieldPosition};
use crate::player::Player;

use bracket_pathfinding::prelude::*;
use bracket_terminal::prelude::{BTerm, GameState};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

pub enum StepperStatus {
    Finished,
    Pending,
}

pub trait Stepper {
    fn clock(&self) -> f64;
    fn process(&mut self, world: &State, ctx: &BTerm) -> StepperStatus;
}

pub struct State {
    pub field: Field,
    fov: HashSet<Point>,
    player: Rc<RefCell<Player>>,
    prev_player_pos: Point,
    enemies: Vec<Rc<RefCell<dyn Enemy>>>,
    current_stepper: Option<Rc<RefCell<dyn Stepper>>>,
}

fn create_enemies(empty_cells: &mut Vec<Point>) -> Vec<Rc<RefCell<dyn Enemy>>> {
    (0..25)
        .into_iter()
        .map(|_| {
            let i = rand::random::<usize>() % empty_cells.len();
            let pos = empty_cells.remove(i);
            Rc::new(RefCell::new(Rat::new(pos))) as Rc<RefCell<dyn Enemy>>
        })
        .collect()
}

impl State {
    pub fn new() -> Self {
        let field = Field::cave(80, 25, 0.6, 1);
        let player = Player::new(Point::new(40, 12), 8);
        let fov = field_of_view_set(player.pos(), player.view_radius, &field);

        let mut empty_cells = field.empty_cells();
        empty_cells.retain(|&x| x != player.pos());

        let enemies = create_enemies(&mut empty_cells);

        State {
            field,
            fov,
            enemies,
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

    fn process_stepper(&mut self, ctx: &mut BTerm) {
        let status = match &self.current_stepper {
            Some(stepper) => stepper.borrow_mut().process(&self, ctx),
            _ => StepperStatus::Finished,
        };

        if let StepperStatus::Finished = status {
            self.current_stepper = None;
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        if let None = self.current_stepper {
            let stepper = Rc::clone(&self.player);
            self.current_stepper = Some(stepper)
        }

        self.process_stepper(ctx);

        if self.prev_player_pos != self.player.borrow().pos() {
            self.update_fov();
        }

        self.field
            .draw_with_fov(ctx, &self.fov, Point::zero(), Point::zero());

        for enemy in &self.enemies {
            let enemy = enemy.borrow();
            enemy.draw_with_fov(ctx, &self.fov, enemy.pos(), enemy.pos());
        }

        self.player.borrow().draw(ctx, self.player.borrow().pos());
    }
}
