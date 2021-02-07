use bracket_terminal::prelude::{BTerm, RGBA};
use rand::Rng;

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
    fn render(&self, ctx: &mut BTerm, x: i32, y: i32) {
        match self {
            Self::Empty => ctx.print_color(x, y, COLOR_EMPTY, COLOR_BG, "."),
            Self::Wall => ctx.print_color(x, y, COLOR_WALL, COLOR_BG, "█"),
        }
    }

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
                n += match cell {
                    FieldCell::Wall => 1,
                    _ => 0,
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

    fn set_rect(&mut self, val: FieldCell, x1: usize, y1: usize, x2: usize, y2: usize) {
        for y in y1..y2 {
            let row = &mut self.data[y];

            for cell in &mut row[x1..x2] {
                *cell = val;
            }
        }
    }

    /// Scale field
    pub fn scale(&mut self, x_scale: usize, y_scale: usize) {
        let mut new_field = Field::new(self.width * x_scale, self.height * y_scale);

        for y in 0..self.height {
            let row = &self.data[y];
            for x in 0..self.width {
                new_field.set_rect(
                    row[x],
                    x * x_scale,
                    y * y_scale,
                    (x + 1) * x_scale,
                    (y + 1) * y_scale,
                )
            }
        }

        *self = new_field
    }

    /// Prints to contex all cells in field
    pub fn render_all(&self, ctx: &mut BTerm) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.data[y][x].render(ctx, x as i32, y as i32);
            }
        }
    }
}
