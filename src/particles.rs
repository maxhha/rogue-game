use crate::colors::{COLOR_BG, COLOR_BLOOD, COLOR_PLAYER};
use crate::draw::{BTerm, Draw, Point};
use rand::Rng;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct BloodParticle {
    pos: Point,
    speed: (f64, f64),
    error: (f64, f64),
    destroy_after: Instant,
}

pub struct BloodParticlesEffect {
    particles: Vec<BloodParticle>,
    prev_process: Instant,
}

impl BloodParticle {
    fn time_left(&self) -> u128 {
        self.destroy_after
            .saturating_duration_since(Instant::now())
            .as_millis()
    }
}

impl BloodParticlesEffect {
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            prev_process: Instant::now(),
        }
    }

    pub fn spawn(&mut self, pos: Point, speed: (f64, f64), number: usize, power: f64) {
        let mut rng = rand::thread_rng();

        for _ in 0..number {
            let (sp_x, sp_y) = speed;
            let k = power * (rng.gen::<f64>() * 0.2 + 0.9) / 125.0;
            let (sp_x, sp_y) = (sp_x * k, sp_y * k);

            let phi = (rng.gen::<f64>() - 0.5) * 3.14;

            let speed = (
                sp_x * phi.cos() + sp_y * phi.sin(),
                sp_y * phi.cos() - sp_x * phi.sin(),
            );

            let duration = Duration::from_millis(rng.gen::<u64>() % 375 + 375);

            let particle = BloodParticle {
                pos,
                speed,
                error: (0.0, 0.0),
                destroy_after: Instant::now() + duration,
            };

            self.particles.push(particle)
        }
    }

    pub fn process(&mut self) {
        self.particles = self
            .particles
            .iter()
            .filter(|x| x.time_left() > 0)
            .map(|x| x.clone())
            .collect();

        let delta = self.prev_process.elapsed().as_millis() as f64;
        self.prev_process = Instant::now();

        for p in &mut self.particles {
            p.error.0 += p.speed.0 * delta;
            p.error.1 += p.speed.1 * delta;

            if p.error.0.abs() >= 0.5 {
                p.pos.x += p.error.0.signum() as i32;
                p.error.0 -= p.error.0.signum();
            }

            if p.error.1.abs() >= 0.5 {
                p.pos.y += p.error.1.signum() as i32;
                p.error.1 -= p.error.1.signum();
            }
        }
    }
}

impl Draw for BloodParticle {
    fn draw(&self, ctx: &mut BTerm, pos: Point) {
        let time_left = self.time_left();

        let ch = if time_left > 250 { '█' } else { '▒' };

        ctx.print_color(pos.x, pos.y, COLOR_BLOOD, COLOR_BG, ch);
    }
}

impl Draw for BloodParticlesEffect {
    fn draw(&self, ctx: &mut BTerm, pos: Point) {
        for p in &self.particles {
            p.draw(ctx, p.pos + pos);
        }
    }
}
