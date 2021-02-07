use bracket_terminal::prelude::{BTerm, RGBA};

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
            _ => ctx.print_color(x, y, COLOR_EMPTY, COLOR_BG, "."),
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

    /// Prints to contex all cells in field
    pub fn render_all(&self, ctx: &mut BTerm) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.data[y][x].render(ctx, x as i32, y as i32);
            }
        }
    }
}
