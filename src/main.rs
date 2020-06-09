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

#[derive(Clone, Copy)]
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
                        open.push((p_x + 2, p_y + 0));
                        open.push((p_x + 0, p_y + 1));
                        open.push((p_x - 1, p_y - 0));
                        open.push((p_x - 2, p_y - 0));
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

#[derive(Clone)]
struct BloodEffectPoint {
    pos: (i32, i32),
    speed: (f64, f64),
    error: (f64, f64),
    lifetime: i32,
}

impl BloodEffectPoint {
    fn new(pos: (i32, i32), speed: (f64, f64), lifetime: i32) -> BloodEffectPoint {
        BloodEffectPoint {
            pos,
            speed,
            lifetime,
            error: (0.0, 0.0),
        }
    }
}

struct BloodEffect {
    points: Vec<BloodEffectPoint>,
}

impl BloodEffect {
    fn new() -> BloodEffect {
        BloodEffect { points: Vec::new() }
    }

    fn spawn(&mut self, pos: (i32, i32), speed: (f64, f64), number: usize, power: f64) {
        let mut rng = rand::thread_rng();

        for _ in 0..number {
            let (sp_x, sp_y) = speed;
            let k = power * (rng.gen::<f64>() * 0.2 + 0.9);
            let (sp_x, sp_y) = (sp_x * k, sp_y * k);
            let phi = (rng.gen::<f64>() - 0.5) * 3.14;
            let sp = (
                sp_x * phi.cos() + sp_y * phi.sin(),
                sp_y * phi.cos() - sp_x * phi.sin(),
            );
            self.points
                .push(BloodEffectPoint::new(pos, sp, rng.gen::<i32>() % 3 + 3));
        }
    }

    fn run(&mut self) {
        self.points = self
            .points
            .iter()
            .filter(|x| x.lifetime > 0)
            .map(|x| {
                let mut pos = x.pos;
                let mut error = x.error;
                error.0 += x.speed.0;
                error.1 += x.speed.1;
                if error.0.abs() >= 0.5 {
                    pos.0 += error.0.signum() as i32;
                    error.0 -= error.0.signum();
                }
                if error.1.abs() >= 0.5 {
                    pos.1 += error.1.signum() as i32;
                    error.1 -= error.1.signum();
                }
                BloodEffectPoint {
                    pos,
                    error,
                    speed: x.speed,
                    lifetime: x.lifetime - 1,
                }
            })
            .collect::<Vec<_>>();
    }

    fn get_draw_chars(&self) -> Vec<(i32, i32, DrawChar)> {
        self.points
            .iter()
            .fold(Vec::<&BloodEffectPoint>::new(), |store, a| {
                let mut new_store = store
                    .into_iter()
                    .filter(|x| x.pos != a.pos || x.lifetime < a.lifetime)
                    .collect::<Vec<_>>();
                new_store.push(a);
                new_store
            })
            .into_iter()
            .map(|x| {
                (
                    x.pos.0,
                    x.pos.1,
                    DrawChar::CharColored(
                        if x.lifetime < 2 { '▒' } else { '█' },
                        color::Rgb(0xEE, 0x33, 0x33),
                    ),
                )
            })
            .collect()
    }
}

fn draw(dest: &mut Array2<DrawChar>, src: &Array2<DrawChar>, pos: (usize, usize)) {
    for (p, v) in src.indexed_iter() {
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
                    write!(stdout, "{}", c).unwrap();
                }
                DrawChar::CharColored(c, color) => {
                    write!(stdout, "{}{}{}", color::Fg(*color), c, reset).unwrap();
                }
                _ => {
                    write!(stdout, " ").unwrap();
                }
            }
        }
    }
}

struct Enemy {
    symbol: DrawChar,
    pos: (i32, i32),
    error: (f64, f64),
}

impl Enemy {
    fn process(&mut self, world: &mut World) {
        let d_x = (world.player.0 - self.pos.0) as f64;
        let d_y = (world.player.1 - self.pos.1) as f64;
        let l = (d_x * d_x + d_y * d_y).sqrt();
        if l > 20.0 || l < 5.0 {
            self.error = (0.0,0.0);
            return;
        }
        let d_x = d_x / l;
        let d_y = d_y / l;
        self.error.0 += d_x;
        self.error.1 += d_y;
        if self.error.0.abs() >= self.error.1.abs() {
            self.pos.0 += self.error.0.signum() as i32;
            self.error.0 -= self.error.0.signum();
        }
        else {
            self.pos.1 += self.error.1.signum() as i32;
            self.error.1 -= self.error.1.signum();
        }
    }
}

struct World {
    blood_eff: BloodEffect,
    player: (i32, i32),
    field: Field,
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

    let mut empty_cells = field.get_empty().into_iter().map(|x| (x.0 as i32, x.1 as i32)).collect::<Vec<_>>();
    let (mut s_x, mut s_y) = empty_cells.remove(rand::random::<usize>() % empty_cells.len());

    let mut blood_eff = BloodEffect::new();

    let mut world = World {
        blood_eff,
        field,
        player: (s_x, s_y),
    };

    let mut enemies: Vec<Enemy> = Vec::new();

    enemies.push(Enemy {
        symbol: DrawChar::CharColored('G', color::Rgb(0x33, 0xff, 0x33)),
        pos: empty_cells.remove(rand::random::<usize>() % empty_cells.len()),
        error: (0.0,0.0),
    });

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

    let mut prev_dir = (0.0, 0.0);

    'main: loop {
        let mut v_x: i32 = 0;
        let mut v_y: i32 = 0;
        loop {
            match stdin_channel.try_recv() {
                Ok(key) => match key {
                    Key::Char('q') => break 'main,
                    Key::Left => v_x -= 1,
                    Key::Right => v_x += 1,
                    Key::Up => v_y -= 1,
                    Key::Down => v_y += 1,
                    Key::Char(' ') => {
                        world.blood_eff.spawn((s_x as i32, s_y as i32), (prev_dir.0, prev_dir.1), 2, 0.6)
                    }
                    _ => {}
                },
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break 'main,
            }
        }

        if v_x.abs() >= v_y.abs() {
            v_x = v_x.signum();
            v_y = 0;
        } else {
            v_y = v_y.signum();
            v_x = 0;
        }

        if v_x.abs() + v_y.abs() > 0 {
            prev_dir = (v_x as f64, v_y as f64)
        }

        s_x += v_x;
        s_y += v_y;

        world.player = (s_x, s_y);

        world.field.render_with_light((s_x as i32, s_y as i32), 5);

        screen.fill(DrawChar::Empty);

        draw(&mut screen, world.field.get_draw_data(), (f_x, f_y));

        for en in enemies.iter() {
            screen[[(en.pos.1 as usize) + f_y, (en.pos.0 as usize) + f_x]] = en.symbol;
        }

        screen[[(s_y as usize) + f_y, (s_x as usize) + f_x]] = DrawChar::Char('@');

        for c in world.blood_eff.get_draw_chars().into_iter() {
            let x = (c.0 + f_x as i32) as usize;
            let y = (c.1 + f_y as i32) as usize;
            if let Some(p) = screen.get_mut([y, x]) {
                *p = c.2;
            }
        }

        write!(
            stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
        )
        .unwrap();

        frame(&mut stdout, &screen);

        write!(stdout, "{}Press q to exit.", termion::cursor::Goto(1, 1)).unwrap();

        stdout.flush().unwrap();

        for en in enemies.iter_mut() {
            en.process(&mut world);
        }

        sleep(100);
        world.blood_eff.run();
    }

    stdin_process.join().expect("Error join stdin process");

    write!(stdout, "{}", termion::cursor::Show).unwrap();
}

fn sleep(millis: u64) {
    let duration = time::Duration::from_millis(millis);
    thread::sleep(duration);
}
