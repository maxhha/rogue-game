use crate::colors::{COLOR_BG, RGBA};
use crate::draw::{BTerm, DrawWithFov, Fov, Point};
use crate::field::FieldPosition;
use crate::state::{State, Stepper, StepperStatus};

const SPOT_DISTANCE: f64 = 8.0;
const ATTACK_DISTANCE: f64 = 1.0;

pub struct Enemy {
    draw_char: char,
    draw_color: RGBA,
    action_time: f64,
    pos: Point,
    clock: f64,
    move_intent: (f64, f64),
    staying_steps: i32,
}

fn normalize(x: f64, y: f64) -> (f64, f64) {
    let l = (x * x + y * y).sqrt();
    (x / l, y / l)
}

impl Enemy {
    pub fn new(draw_char: char, draw_color: RGBA, action_time: f64, pos: Point) -> Self {
        Self {
            clock: 0.0,
            pos,
            draw_char,
            draw_color,
            action_time,
            move_intent: (0.0, 0.0),
            staying_steps: 0,
        }
    }

    fn wonder_intent(&self) -> f64 {
        f64::max(
            (2.17_f64).powf(2.0 * (self.staying_steps as f64)) - 1.0,
            0.0,
        )
    }

    fn follow(&mut self, target: Point) {
        let dx = (target.x - self.pos.x) as f64;
        let dy = (target.y - self.pos.y) as f64;

        let (dx_norm, dy_norm) = normalize(dx, dy);

        self.move_intent.0 += dx_norm;
        self.move_intent.1 += dy_norm;
    }

    fn random_wondering(&mut self, intent: f64) {
        self.move_intent.0 += intent * (rand::random::<f64>() - 0.5);
        self.move_intent.1 += intent * (rand::random::<f64>() - 0.5);
    }

    fn action_move(&mut self, world: &State) {
        self.move_intent = normalize(self.move_intent.0, self.move_intent.1);
        let mut next = self.pos;

        if self.move_intent.0.abs() >= self.move_intent.1.abs() {
            next.x += self.move_intent.0.signum() as i32;
            self.move_intent.0 -= self.move_intent.0.signum();
        } else {
            next.y += self.move_intent.1.signum() as i32;
            self.move_intent.1 -= self.move_intent.1.signum();
        }

        if self.can_move(world, &next) {
            self.pos = next;
            self.staying_steps = i32::max(self.staying_steps - 1, 0);
        } else {
            self.staying_steps += 1;
        }
    }

    fn can_move(&self, world: &State, target: &Point) -> bool {
        !world.field.is_wall(target.x, target.y)
            && world
                .enemies
                .iter()
                .filter(|e| match e.try_borrow() {
                    Err(_) => false,
                    Ok(e) => e.pos == *target,
                })
                .next()
                .is_none()
    }
}

impl FieldPosition for Enemy {
    fn pos(&self) -> Point {
        self.pos
    }
}

impl DrawWithFov for Enemy {
    fn draw_with_fov(&self, ctx: &mut BTerm, fov: &Fov, pos: Point, fov_pos: Point) {
        if fov.contains(&fov_pos) {
            ctx.print_color(pos.x, pos.y, self.draw_color, COLOR_BG, self.draw_char)
        }
    }
}

impl Stepper for Enemy {
    fn clock(&self) -> f64 {
        self.clock
    }

    fn process(&mut self, world: &State, _ctx: &BTerm) -> StepperStatus {
        self.clock += self.action_time;

        let delta = world.player.borrow().pos() - self.pos;
        let distance_squared = delta.x.pow(2) + delta.y.pow(2);
        let distance = (distance_squared as f64).sqrt();

        if distance <= ATTACK_DISTANCE {
            StepperStatus::Finished
        } else if distance < SPOT_DISTANCE {
            let wonder_intent = self.wonder_intent();

            self.follow(world.player.borrow().pos());
            self.random_wondering(wonder_intent);

            self.action_move(world);

            StepperStatus::Finished
        } else {
            StepperStatus::Finished
        }
    }
}
