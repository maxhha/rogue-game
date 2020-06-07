use ndarray::{s, Array2};
use rand::Rng;
use terminal_size::{terminal_size, Height, Width};
use termion::{color, style};

#[derive(Clone)]
enum DrawChar {
    Empty,
    Char(char),
    CharColored(char, color::Rgb),
}

trait Drawable {
    fn get_draw_data(&self) -> Array2<DrawChar>;
}

#[derive(Clone)]
enum FieldCell {
    Empty,
    Wall,
}

struct Field {
    data: Array2<FieldCell>,
}

impl Field {
    fn new(width: usize, height: usize) -> Field {
        Field {
            data: Array2::from_elem((height, width), FieldCell::Empty),
        }
    }

    fn rand_cave(width: usize, height: usize, k: f32, smooth: usize) -> Field {
        let mut field = Field::new(width, height);

        field.fill_rand(k);

        for _ in 0..smooth {
            field.smooth();
        }

        field
    }

    fn fill_rand(&mut self, k: f32) {
        let mut rng = rand::thread_rng();

        for x in self.data.iter_mut() {
            *x = if rng.gen::<f32>() > k {
                FieldCell::Wall
            } else {
                FieldCell::Empty
            };
        }
    }

    fn smooth(&mut self) {
        let (h, w) = self.data.dim();
        let mut new_data = self.data.clone();

        for (pos, val) in self.data.indexed_iter() {
            let (y, x) = pos;

            if x == 0 || y == 0 || x == w - 1 || y == h - 1 {
                continue;
            }

            let sum = self
                .data
                .slice(s![y - 1..=y + 1, x - 1..=x + 1])
                .into_iter()
                .fold(0, |a, b| {
                    a + match b {
                        FieldCell::Wall => 1,
                        _ => 0,
                    }
                });

            new_data[[y, x]] = if let FieldCell::Wall = val {
                if sum > 4 {
                    FieldCell::Wall
                } else {
                    FieldCell::Empty
                }
            } else {
                if sum > 4 {
                    FieldCell::Wall
                } else {
                    FieldCell::Empty
                }
            }
        }
        self.data = new_data;
    }

    fn stretch(&mut self, x_scale: usize, y_scale: usize) {
        let (h, w) = self.data.dim();
        let mut new_data = Array2::from_elem((h * y_scale, w * x_scale), FieldCell::Empty);

        for (pos, val) in self.data.indexed_iter() {
            let (y, x) = pos;
            for dx in 0..x_scale {
                for dy in 0..y_scale {
                    new_data[[y * y_scale + dy, x * x_scale + dx]] = (*val).clone();
                }
            }
        }
        self.data = new_data;
    }

    fn size(&self) -> (usize, usize) {
        self.data.dim()
    }

    fn set_bounds(&mut self) {
        let (h, w) = self.data.dim();
        for x in 0..w {
            self.data[[0, x]] = FieldCell::Wall;
            self.data[[h-1, x]] = FieldCell::Wall;
        }
        for y in 1..h-1 {
            self.data[[y, 0]] = FieldCell::Wall;
            self.data[[y, w - 1]] = FieldCell::Wall;
        }
    }
}

impl Drawable for Field {
    fn get_draw_data(&self) -> Array2<DrawChar> {
        self.data.map(|x| match x {
            FieldCell::Empty => DrawChar::Char('.'),
            FieldCell::Wall => DrawChar::Char('█'),
        })
    }
}

fn draw_colored<D: Drawable>(
    dest: &mut Array2<DrawChar>,
    src: D,
    pos: (usize, usize),
    color: color::Rgb,
) {
    for (p, v) in src.get_draw_data().indexed_iter() {
        let x = p.1 + pos.0;
        let y = p.0 + pos.1;

        match (dest.get_mut([y, x]), v) {
            (Some(p), &DrawChar::Char(c)) => *p = DrawChar::CharColored(c, color),
            (Some(p), &DrawChar::CharColored(c, _)) => *p = DrawChar::CharColored(c, color),
            _ => (),
        }
    }
}

fn draw<D: Drawable>(dest: &mut Array2<DrawChar>, src: D, pos: (usize, usize)) {
    for (p, v) in src.get_draw_data().indexed_iter() {
        let x = p.1 + pos.0;
        let y = p.0 + pos.1;

        match (dest.get_mut([y, x]), v) {
            (Some(p), &DrawChar::Char(c)) => *p = DrawChar::Char(c),
            (Some(p), &DrawChar::CharColored(c, color)) => *p = DrawChar::CharColored(c, color),
            _ => (),
        }
    }
}

fn main() {
    let (Width(width), Height(height)) = terminal_size().expect("Error terminal size.");

    let screen_width = width as usize;
    let screen_height = height as usize;

    let mut screen = Array2::from_elem((screen_height, screen_width), DrawChar::Empty);

    let field_width = 20;
    let field_height = 10;
    let mut field = Field::rand_cave(field_width, field_height, 0.6, 2);
    field.set_bounds();

    let field_width = field_width * 2;
    field.stretch(2, 1);

    let f_x = (screen_width - field_width) / 2;
    let f_y = (screen_height - field_height) / 2;

    draw_colored(&mut screen, field, (f_x, f_y), color::Rgb(0x88, 0x66, 0x88));

    for row in screen.genrows() {
        for c in row {
            match c {
                DrawChar::Char(c) => {
                    print!("{}", c);
                },
                DrawChar::CharColored(c, color) => {
                    print!("{}{}{}", color::Fg(*color), c, style::Reset);
                },
                _ => print!(" "),
            }
        }
        print!("\n");
    }

    //
    // let field: Vec<Vec<i32>> = smooth(smooth(field));
    //
    // let wall = format!(
    //     "{}██{}",
    //     color::Fg(color::Rgb(0x88, 0x66, 0x88)),
    //     style::Reset
    // );
    // let space = format!(
    //     "{}  {}",
    //     color::Bg(color::Rgb(0x11, 0x00, 0x11)),
    //     style::Reset
    // );
    //
    // for row in field.iter() {
    //     for v in row.iter() {
    //         print!(
    //             "{}",
    //             match v {
    //                 1 => &wall,
    //                 _ => &space,
    //             }
    //         )
    //     }
    //     print!("\n");
    // }
}
