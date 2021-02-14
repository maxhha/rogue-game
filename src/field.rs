use crate::draw::{Draw, DrawWithFov, Fov};
use bracket_pathfinding::prelude::*;
use bracket_terminal::prelude::{BTerm, RGBA};
use rand::Rng;
use std::collections::HashSet;

const COLOR_WALL: RGBA = RGBA {
    r: 0.5333,
    g: 0.4,
    b: 0.5333,
    a: 1.0,
};

const COLOR_EMPTY: RGBA = RGBA {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};

const COLOR_BG: RGBA = RGBA {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.0,
};

#[derive(Clone, Copy)]
enum FieldCell {
    Empty,
    Wall,
}

impl FieldCell {
    fn smoothed(&self, n_neighbour_walls: u32) -> Self {
        match self {
            Self::Empty => {
                if n_neighbour_walls > 4 {
                    Self::Wall
                } else {
                    Self::Empty
                }
            }
            Self::Wall => {
                if n_neighbour_walls > 4 {
                    Self::Wall
                } else {
                    Self::Empty
                }
            }
        }
    }
}

impl Draw for FieldCell {
    fn draw(&self, ctx: &mut BTerm, pos: Point) {
        match self {
            Self::Empty => ctx.print_color(pos.x, pos.y, COLOR_EMPTY, COLOR_BG, "."),
            Self::Wall => ctx.print_color(pos.x, pos.y, COLOR_WALL, COLOR_BG, "█"),
        }
    }
}

pub struct Field {
    width: usize,
    height: usize,
    data: Vec<Vec<FieldCell>>,
}

impl Field {
    /// Create field filled with empty cells
    pub fn new(width: usize, height: usize) -> Field {
        Field {
            width,
            height,
            data: (0..height)
                .into_iter()
                .map(|_| std::iter::repeat(FieldCell::Empty).take(width).collect())
                .collect(),
        }
    }

    /// Randomly fill field. k - probability of empty space
    fn fill_rand(&mut self, k: f32) {
        let mut rng = rand::thread_rng();

        for row in self.data.iter_mut() {
            for cell in row.iter_mut() {
                *cell = if rng.gen::<f32>() > k {
                    FieldCell::Wall
                } else {
                    FieldCell::Empty
                };
            }
        }
    }

    /// Set walls at the borders of a field
    fn set_borders(&mut self) {
        for x in 0..self.width {
            self.data[0][x] = FieldCell::Wall;
            self.data[self.height - 1][x] = FieldCell::Wall;
        }
        for y in 1..self.height - 1 {
            self.data[y][0] = FieldCell::Wall;
            self.data[y][self.width - 1] = FieldCell::Wall;
        }
    }

    fn count_neighbour_walls(&self, x: usize, y: usize) -> u32 {
        let mut n = 0;

        for row in &self.data[y - 1..=y + 1] {
            for cell in &row[x - 1..=x + 1] {
                if let FieldCell::Wall = cell {
                    n += 1;
                }
            }
        }

        n
    }

    /// Smooth field walls, so they look less random
    fn smooth(&mut self) {
        let mut new_data = self.data.clone();

        for y in 1..self.height - 1 {
            let row = &self.data[y];
            let new_row = &mut new_data[y];

            for x in 1..self.width - 1 {
                let n_walls = self.count_neighbour_walls(x, y);

                new_row[x] = row[x].smoothed(n_walls);
            }
        }

        self.data = new_data;
    }

    /// Create cave field
    pub fn cave(width: usize, height: usize, prob_empty_cell: f32, smooth_repeats: usize) -> Field {
        let mut field = Field::new(width, height);

        field.fill_rand(prob_empty_cell);
        field.set_borders();

        for _ in 0..smooth_repeats {
            field.smooth();
        }

        field
    }

    pub fn is_wall(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 {
            return true;
        }

        let x = x as usize;
        let y = y as usize;

        if x >= self.width || y >= self.height {
            return true;
        }

        match self.data[y as usize][x as usize] {
            FieldCell::Wall => true,
            FieldCell::Empty => false,
        }
    }
}

impl Draw for Field {
    /// Prints to contex all cells in field
    fn draw(&self, ctx: &mut BTerm, base: Point) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.data[y][x].draw(ctx, Point::new(x, y) + base);
            }
        }
    }
}

impl DrawWithFov for Field {
    fn draw_with_fov(&self, ctx: &mut BTerm, fov: &Fov, pos: Point, fov_pos: Point) {
        for p in fov {
            let p = *p - fov_pos;

            if p.x < 0 || p.y < 0 {
                continue;
            }

            let x = p.x as usize;
            let y = p.y as usize;

            if x >= self.width || y >= self.height {
                continue;
            }

            self.data[y][x].draw(ctx, p + pos);
        }
    }
}

impl Algorithm2D for Field {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Field {
    fn is_opaque(&self, idx: usize) -> bool {
        let x = idx % self.width;
        let y = idx / self.width;
        self.is_wall(x as i32, y as i32)
    }
}
