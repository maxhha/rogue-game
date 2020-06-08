use ndarray::{s, Array2};
use rand::Rng;
use std::io::{stdin, stdout, Write};
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::{thread, time};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::terminal_size;
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
            let s_x = source.0;
            let s_y = source.1;
            let dist_sqr = (distance * distance) as i32;
            let mut open = vec![(s_x, s_y)];

            while open.len() > 0 {
                let (p_x, p_y) = open.remove(0);
                let w = (p_x - s_x) * (p_x - s_x) / 4 + (p_y - s_y) * (p_y - s_y);
                if w > dist_sqr {
                    continue;
                }

                if let Some(DrawChar::Empty) = sprite.get([p_y as usize, p_x as usize]) {
                    if let Some(FieldCell::Empty) = self.data.get([p_y as usize, p_x as usize]) {
                        sprite[[p_y as usize, p_x as usize]] = DrawChar::Char('.');
                        open.push((p_x + 1, p_y + 0));
                        open.push((p_x + 0, p_y + 1));
                        open.push((p_x - 1, p_y - 0));
                        open.push((p_x - 0, p_y - 1));
                    }
                    if let Some(FieldCell::Wall) = self.data.get([p_y as usize, p_x as usize]) {
                        sprite[[p_y as usize, p_x as usize]] =
                            DrawChar::CharColored('█', color::Rgb(0x88, 0x66, 0x88));
                    }
                }
            }
        }
        self.sprite = Some(sprite);
    }

    fn get_empty(&self) -> Vec<(usize, usize)> {
        let mut v = Vec::new();

        for (p, val) in self.data.indexed_iter() {
            if let FieldCell::Empty = val {
                v.push((p.1, p.0));
            }
        }
        v
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

fn frame(
    stdout: &mut termion::screen::AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>,
    screen: &Array2<DrawChar>,
) {
    let reset = style::Reset;

    for row in screen.genrows() {
        for c in row {
            match c {
                DrawChar::Char(c) => {
                    write!(stdout, "{}", c);
                }
                DrawChar::CharColored(c, color) => {
                    write!(stdout, "{}{}{}", color::Fg(*color), c, reset);
                }
                _ => {
                    write!(stdout, " ");
                }
            }
        }
    }
}

fn main() {
    let (width, height) = terminal_size().unwrap();

    let screen_width = width as usize;
    let screen_height = height as usize;

    let mut screen = Array2::from_elem((screen_height, screen_width), DrawChar::Empty);

    let field_width = 20;
    let field_height = 10;
    let mut field = Field::rand_cave(field_width, field_height, 0.66, 3);

    let field_width = field_width * 2;
    field.stretch(2, 1);

    let f_x = (screen_width - field_width) / 2;
    let f_y = (screen_height - field_height) / 2;

    let mut empty_cells = field.get_empty();
    let (mut s_x, mut s_y) = empty_cells.remove(rand::random::<usize>() % empty_cells.len());

    let mut stdout = termion::screen::AlternateScreen::from(stdout().into_raw_mode().unwrap());

    write!(
        stdout,
        "{}{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        termion::cursor::Hide
    )
    .unwrap();

    stdout.flush().unwrap();

    let (tx, rx) = mpsc::channel::<Key>();

    let stdin_process = thread::spawn(move || {
        let stdin = stdin();
        for c in stdin.keys() {
            if let Ok(c) = c {
                tx.send(c).unwrap();
                if let Key::Char('q') = c {
                    break;
                }
            }
        }
    });

    let stdin_channel = rx;

    loop {
        match stdin_channel.try_recv() {
            Ok(key) => match key {
                Key::Char('q') => break,
                Key::Left => s_x -= 1,
                Key::Right => s_x += 1,
                Key::Up => s_y -= 1,
                Key::Down => s_y += 1,
                _ => {}
            },
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => break,
        }

        field.render_with_light((s_x as i32, s_y as i32), 5);

        screen.fill(DrawChar::Empty);

        draw(&mut screen, &field, (f_x, f_y));
        screen[[(s_y as usize) + f_y, (s_x as usize) + f_x]] = DrawChar::Char('@');

        write!(
            stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
        )
        .unwrap();

        frame(&mut stdout, &screen);

        write!(stdout, "{}Press q to exit.", termion::cursor::Goto(1, 1),).unwrap();

        stdout.flush().unwrap();
        sleep(100);
    }

    stdin_process.join().expect("Error join stdin process");

    write!(stdout, "{}", termion::cursor::Show).unwrap();
}

fn sleep(millis: u64) {
    let duration = time::Duration::from_millis(millis);
    thread::sleep(duration);
}
