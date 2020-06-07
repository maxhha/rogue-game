use ndarray::{s, Array2};
use rand::Rng;
use std::io;
use terminal_size::{terminal_size, Height, Width};
use termion::{color, style};

#[derive(Clone)]
enum DrawChar {
    Empty,
    Char(char),
    CharColored(char, color::Rgb),
}

trait Drawable {
    fn get_draw_data(&self) -> &Array2<DrawChar>;
}

#[derive(Clone)]
enum FieldCell {
    Empty,
    Wall,
}

struct Field {
    sprite: Option<Array2<DrawChar>>,
    data: Array2<FieldCell>,
}

impl Field {
    fn new(width: usize, height: usize) -> Field {
        Field {
            sprite: None,
            data: Array2::from_elem((height, width), FieldCell::Empty),
        }
    }

    fn rand_cave(width: usize, height: usize, k: f32, smooth: usize) -> Field {
        let mut field = Field::new(width, height);

        field.fill_rand(k);
        field.set_bounds();

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
            self.data[[h - 1, x]] = FieldCell::Wall;
        }
        for y in 1..h - 1 {
            self.data[[y, 0]] = FieldCell::Wall;
            self.data[[y, w - 1]] = FieldCell::Wall;
        }
    }

    fn render_all(&mut self) {
        self.sprite = Some(self.data.map(|x| match x {
            FieldCell::Empty => DrawChar::Char('.'),
            FieldCell::Wall => DrawChar::CharColored('█', color::Rgb(0x88, 0x66, 0x88)),
        }));
    }

    fn render_with_light(&mut self, source: (i32, i32), distance: usize) {
        let (h, w) = self.data.dim();
        let mut sprite = Array2::from_elem((h, w), DrawChar::Empty);

        if source.0 >= 0 && source.1 >= 0 && source.0 < w as i32 && source.1 < h as i32 {
            let mut open = vec![(source.0 as usize, source.1 as usize, 0)];

            while open.len() > 0 {
                let (p_x, p_y, w) = open.remove(0);

                if w > distance {
                    continue;
                }

                if let Some(DrawChar::Empty) = sprite.get([p_y, p_x]) {
                    if let Some(FieldCell::Empty) = self.data.get([p_y, p_x]) {
                        sprite[[p_y, p_x]] = DrawChar::Char('.');
                        open.push((p_x + 1, p_y + 0, w + 1));
                        open.push((p_x + 0, p_y + 1, w + 1));
                        open.push((p_x - 1, p_y - 0, w + 1));
                        open.push((p_x - 0, p_y - 1, w + 1));
                    }
                    if let Some(FieldCell::Wall) = self.data.get([p_y, p_x]) {
                        sprite[[p_y, p_x]] =
                            DrawChar::CharColored('█', color::Rgb(0x88, 0x66, 0x88));
                    }
                }
            }
        }
        self.sprite = Some(sprite);
    }
}

impl Drawable for Field {
    fn get_draw_data(&self) -> &Array2<DrawChar> {
        self.sprite.as_ref().expect("Not rendered field")
    }
}

fn draw_colored<D: Drawable>(
    dest: &mut Array2<DrawChar>,
    src: &D,
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

fn draw<D: Drawable>(dest: &mut Array2<DrawChar>, src: &D, pos: (usize, usize)) {
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
    let mut field = Field::rand_cave(field_width, field_height, 0.66, 3);

    let field_width = field_width * 2;
    field.stretch(2, 1);

    field.render_all();

    let f_x = (screen_width - field_width) / 2;
    let f_y = (screen_height - field_height) / 2;

    draw(&mut screen, &field, (f_x, f_y));

    let reset = style::Reset;

    for row in screen.genrows() {
        for c in row {
            match c {
                DrawChar::Char(c) => {
                    print!("{}", c);
                }
                DrawChar::CharColored(c, color) => {
                    print!("{}{}{}", color::Fg(*color), c, reset);
                }
                _ => {
                    print!(" ");
                }
            }
        }
        print!("\n");
    }

    let mut user_input = String::new();

    io::stdin()
        .read_line(&mut user_input)
        .expect("Failed to read line");

    let user_input: Vec<i32> = user_input
        .trim()
        .split_whitespace()
        .map(|x| x.parse().expect("Wrong input."))
        .collect();

    let s_x = user_input[0];
    let s_y = user_input[1];

    field.render_with_light((s_x, s_y), 32);

    screen.fill(DrawChar::Empty);

    draw(&mut screen, &field, (f_x, f_y));
    screen[[(s_y as usize) + f_y, (s_x as usize) + f_x]] = DrawChar::Char('@');

    for row in screen.genrows() {
        for c in row {
            match c {
                DrawChar::Char(c) => {
                    print!("{}", c);
                }
                DrawChar::CharColored(c, color) => {
                    print!("{}{}{}", color::Fg(*color), c, reset);
                }
                _ => {
                    print!(" ");
                }
            }
        }
        print!("\n");
    }
}
