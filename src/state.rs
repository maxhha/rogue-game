use crate::draw::{Draw, DrawWithFov};
use crate::enemy::{Enemy, EnemyBuilder};
use crate::field::{Field, FieldPosition};
use crate::particles::BloodParticlesEffect;
use crate::player::Player;

use bracket_pathfinding::prelude::*;
use bracket_terminal::prelude::{BTerm, GameState, VirtualKeyCode};
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
    screen_width: u64,
    screen_height: u64,
    pub field: Field,
    fov: HashSet<Point>,
    pub player: Rc<RefCell<Player>>,
    prev_player_pos: Point,
    pub enemies: Vec<Rc<RefCell<Enemy>>>,
    current_stepper: Option<Rc<RefCell<dyn Stepper>>>,
    pub blood_effect: RefCell<BloodParticlesEffect>,
}

fn remove_random<E>(v: &mut Vec<E>) -> E {
    v.remove(rand::random::<usize>() % v.len())
}

fn create_enemies(empty_cells: &mut Vec<Point>) -> Vec<Rc<RefCell<Enemy>>> {
    (0..25)
        .into_iter()
        .map(|_| {
            let pos = remove_random(empty_cells);
            let rat = EnemyBuilder::rat().pos(pos).build();
            Rc::new(RefCell::new(rat))
        })
        .collect()
}

impl State {
    pub fn new(screen_width: u64, screen_height: u64) -> Self {
        let field = Field::cave(80, 25, 0.6, 1);
        let mut empty_cells = field.empty_cells();

        let player = Player::new(remove_random(&mut empty_cells), 8);
        let fov = field_of_view_set(player.pos(), player.view_radius, &field);

        let enemies = create_enemies(&mut empty_cells);

        let blood_effect = BloodParticlesEffect::new();

        State {
            screen_width,
            screen_height,
            field,
            fov,
            enemies,
            current_stepper: None,
            prev_player_pos: player.pos(),
            player: Rc::new(RefCell::new(player)),
            blood_effect: RefCell::new(blood_effect),
        }
    }

    fn update_fov(&mut self) {
        let player = self.player.borrow();
        let pos = player.pos();

        self.fov = field_of_view_set(pos, player.view_radius, &self.field);
        self.prev_player_pos = pos;
    }

    fn next_stepper(&self) -> Option<Rc<RefCell<dyn Stepper>>> {
        let mut stepper = Rc::clone(&self.player) as Rc<RefCell<dyn Stepper>>;
        let mut min_clock = stepper.borrow().clock();

        for enemy in &self.enemies {
            let clock = enemy.borrow().clock();

            if clock < min_clock {
                min_clock = clock;
                stepper = enemy.clone() as Rc<RefCell<dyn Stepper>>;
            }
        }

        Some(stepper)
    }

    fn process(&self, ctx: &mut BTerm) {
        if let Some(VirtualKeyCode::Q) = ctx.key {
            ctx.quit()
        }
    }

    fn process_stepper(&mut self, ctx: &mut BTerm) {
        if let None = self.current_stepper {
            self.current_stepper = self.next_stepper();
        }

        let status = match &self.current_stepper {
            Some(stepper) => stepper.borrow_mut().process(&self, ctx),
            _ => StepperStatus::Finished,
        };

        if let StepperStatus::Finished = status {
            self.current_stepper = None;
        }
    }

    fn is_player_current_stepper(&self) -> bool {
        match &self.current_stepper {
            Some(stepper) => {
                let player = self.player.clone() as Rc<RefCell<dyn Stepper>>;
                Rc::ptr_eq(stepper, &player)
            }
            None => false,
        }
    }

    fn draw_wait(&self, ctx: &mut BTerm) {
        ctx.print_centered(self.screen_height - 2, "[wait]")
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();

        self.process(ctx);
        self.blood_effect.borrow_mut().process();
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
        self.blood_effect.borrow().draw(ctx, Point::zero());

        if !self.is_player_current_stepper() {
            self.draw_wait(ctx)
        }
    }
}
