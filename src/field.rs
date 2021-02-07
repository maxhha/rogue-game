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

#[derive(Clone)]
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

    /// Create cave field
    pub fn cave(width: usize, height: usize, prob_empty_cell: f32, smooth_repeats: usize) -> Field {
        let mut field = Field::new(width, height);

        field.fill_rand(prob_empty_cell);
        field.set_borders();

        // for _ in 0..smooth_repeats {
        //     field.smooth();
        // }

        field
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

    /// Prints to contex all cells in field
    pub fn render_all(&self, ctx: &mut BTerm) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.data[y][x].render(ctx, x as i32, y as i32);
            }
        }
    }
}
