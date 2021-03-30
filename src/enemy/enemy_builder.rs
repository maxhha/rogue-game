use crate::colors::{COLOR_RAT, RGBA};
use crate::draw::Point;
use crate::enemy::Enemy;

pub struct EnemyBuilder {
    draw_char: char,
    draw_color: RGBA,
    pos: Point,
    action_time: f64,
}

impl EnemyBuilder {
    pub fn rat() -> Self {
        Self {
            draw_char: 'r',
            draw_color: COLOR_RAT,
            pos: Point::zero(),
            action_time: 0.5,
        }
    }

    pub fn pos(mut self, pos: Point) -> Self {
        self.pos = pos;
        self
    }

    pub fn build(self) -> Enemy {
        Enemy::new(self.draw_char, self.draw_color, self.action_time, self.pos)
    }
}
