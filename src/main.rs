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

    fn render_all(&self) -> Array2<DrawChar> {
        self.data.map(|x| match x {
            FieldCell::Empty => DrawChar::Char('.'),
            FieldCell::Wall => DrawChar::CharColored('█', color::Rgb(0x88, 0x66, 0x88)),
        })
    }

    fn render_with_light(&self, source: (i32, i32), distance: usize) -> Array2<DrawChar> {
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
        sprite
    }

    fn is_wall(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 {
            return true;
        }
        match self.data.get([y as usize, x as usize]) {
            None => true,
            Some(p) => match p {
                FieldCell::Empty => false,
                FieldCell::Wall => true,
            },
        }
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
    screen: &mut Array2<DrawChar>,
    blood_eff: &mut BloodEffect,
    field: &Field,
    field_pos: (usize, usize),
    player: &Player,
    enemies: &Vec<Enemy>,
) {
    let f_x = field_pos.0 as i32;
    let f_y = field_pos.1 as i32;

    screen.fill(DrawChar::Empty);

    let field_sprite = field.render_with_light(player.pos, player.lighting_power as usize);

    draw(screen, &field_sprite, field_pos);

    for en in enemies.iter() {
        let x = (en.pos.0 + f_x as i32) as usize;
        let y = (en.pos.1 + f_y as i32) as usize;
        if let DrawChar::Char(_) = screen[[y, x]] {
            screen[[y, x]] = en.symbol;
        } else if let DrawChar::CharColored(_, _) = screen[[y, x]] {
            screen[[y, x]] = en.symbol;
        }
    }

    screen[[
        (player.pos.1 + f_y as i32) as usize,
        (player.pos.0 + f_x as i32) as usize,
    ]] = DrawChar::Char('@');

    for c in blood_eff.get_draw_chars().into_iter() {
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

    write!(
        stdout,
        "{}Press q to exit.\n\rArrows to move and attack\n\rEnemies: {}",
        termion::cursor::Goto(1, 1),
        enemies.len(),
    )
    .unwrap();

    stdout.flush().unwrap();

    blood_eff.run();
}

#[derive(Clone, Copy)]
struct UnitInfo {
    attack: f64,
    defence: f64,
    luck: f64,
}

impl UnitInfo {
    fn kill_prob(&self, enemy: &UnitInfo) -> f64 {
        self.attack * (1.0 + self.luck) / enemy.defence / (1.0 + enemy.luck)
    }
}

#[derive(Clone, Copy)]
struct Enemy {
    symbol: DrawChar,
    pos: (i32, i32),
    error: (f64, f64),
    timer: i32,
    step: f64,
    step_speed: f64,
    stay_timer: i32,
    info: UnitInfo,
}

impl Enemy {
    fn process(
        &mut self,
        player: &mut Player,
        field: &Field,
        blood_eff: &mut BloodEffect,
        enemies: &Vec<Enemy>,
    ) {
        if self.timer > 0 {
            self.timer -= 1;
        }
        let d_x = (player.pos.0 - self.pos.0) as f64;
        let d_y = (player.pos.1 - self.pos.1) as f64;
        let l = (d_x * d_x + d_y * d_y).sqrt();
        if l > 8.0 || l <= 1.0 {
            self.error = (0.0, 0.0);

            if l <= 1.0 && self.timer <= 0 {
                self.timer = 2;
                blood_eff.spawn(
                    player.pos,
                    (d_x, d_y),
                    (2.0 * self.info.attack / 0.03) as usize,
                    0.6 * self.info.attack / 0.03,
                );
                if rand::random::<f64>() < self.info.kill_prob(&player.info) {
                    player.dead = true;
                }
            }
            return;
        }
        let d_x = d_x / l;
        let d_y = d_y / l;

        let stay_k = f64::max((2.17_f64).powf(2.0 * (self.stay_timer as f64)) - 1.0, 0.0);

        self.error.0 += d_x + stay_k * (rand::random::<f64>() - 0.5);
        self.error.1 += d_y + stay_k * (rand::random::<f64>() - 0.5);

        let l = self.error.0.powf(2.0) + self.error.1.powf(2.0);
        let l = l.sqrt();
        self.error.0 /= l;
        self.error.1 /= l;
        let mut next = self.pos;
        if self.error.0.abs() >= self.error.1.abs() {
            next.0 += self.error.0.signum() as i32;
            self.error.0 -= self.error.0.signum();
        } else {
            next.1 += self.error.1.signum() as i32;
            self.error.1 -= self.error.1.signum();
        }
        if !field.is_wall(next.0, next.1)
            && enemies
                .iter()
                .filter(|e| e.pos.0 == next.0 && e.pos.1 == next.1)
                .next()
                .is_none()
        {
            self.pos = next;
            self.stay_timer = i32::max(self.stay_timer - 1, 0);
        } else {
            self.stay_timer += 1;
        }
    }
}

struct Player {
    pos: (i32, i32),
    step: f64,
    step_speed: f64,
    dead: bool,
    lighting_power: i32,
    info: UnitInfo,
}

impl Player {
    fn action(
        &mut self,
        blood_eff: &mut BloodEffect,
        direction: (i32, i32),
        field: &Field,
        enemies: &mut Vec<Enemy>,
    ) -> bool {
        let (v_x, v_y) = direction;
        let next_x = self.pos.0 + v_x;
        let next_y = self.pos.1 + v_y;

        if !field.is_wall(next_x, next_y) {
            let first_enemy = enemies
                .iter()
                .enumerate()
                .filter(|x| x.1.pos.0 == next_x && x.1.pos.1 == next_y)
                .map(|x| x.0)
                .next();
            if let Some(enemy_i) = first_enemy {
                blood_eff.spawn((next_x, next_y), (v_x as f64, v_y as f64), 2, 0.6);
                if rand::random::<f64>() < self.info.kill_prob(&enemies[enemy_i].info) {
                    enemies.remove(enemy_i);
                }
                true
            } else {
                self.pos = (next_x, next_y);
                true
            }
        } else {
            false
        }
    }
}

fn game() -> bool {
    let (width, height) = terminal_size().unwrap();

    let screen_width = width as usize;
    let screen_height = height as usize;

    let mut screen = Array2::from_elem((screen_height, screen_width), DrawChar::Empty);

    let field_width = screen_width / 4;
    let field_height = screen_height / 2;
    let mut field = Field::rand_cave(field_width, field_height, 0.6, 1);

    let field_width = field_width * 2;
    field.stretch(2, 1);

    let f_x = (screen_width - field_width) / 2;
    let f_y = (screen_height - field_height) / 2;

    let mut empty_cells = field
        .get_empty()
        .into_iter()
        .map(|x| (x.0 as i32, x.1 as i32))
        .collect::<Vec<_>>();

    let mut blood_eff = BloodEffect::new();

    let mut player = Player {
        pos: empty_cells.remove(rand::random::<usize>() % empty_cells.len()),
        step: 0.0,
        step_speed: 1.0,
        dead: false,
        lighting_power: 5,
        info: UnitInfo {
            attack: 1.0,
            defence: 1.0,
            luck: 0.0,
        },
    };

    let mut enemies = Vec::new();
    let rats = 25;
    let goblins = 10;
    let trolls = 5;

    for _ in 0..rats {
        enemies.push(Enemy {
            symbol: DrawChar::CharColored('r', color::Rgb(0x66, 0x66, 0x77)),
            pos: empty_cells.remove(rand::random::<usize>() % empty_cells.len()),
            error: (0.0, 0.0),
            timer: 0,
            step: 0.0,
            step_speed: 2.0,
            stay_timer: 0,
            info: UnitInfo {
                attack: 0.01,
                defence: 1.0,
                luck: 0.0,
            },
        });
    }

    for _ in 0..goblins {
        enemies.push(Enemy {
            symbol: DrawChar::CharColored('G', color::Rgb(0x33, 0xff, 0x33)),
            pos: empty_cells.remove(rand::random::<usize>() % empty_cells.len()),
            error: (0.0, 0.0),
            timer: 0,
            step: 0.0,
            step_speed: 0.75,
            stay_timer: 0,
            info: UnitInfo {
                attack: 0.05,
                defence: 5.0,
                luck: 0.0,
            },
        });
    }

    for _ in 0..trolls {
        enemies.push(Enemy {
            symbol: DrawChar::CharColored('T', color::Rgb(0x44, 0xee, 0xee)),
            pos: empty_cells.remove(rand::random::<usize>() % empty_cells.len()),
            error: (0.0, 0.0),
            timer: 0,
            step: 0.0,
            step_speed: 0.2,
            stay_timer: 0,
            info: UnitInfo {
                attack: 0.25,
                defence: 25.0,
                luck: 0.0,
            },
        });
    }

    let start_enemies = enemies.len();

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
                match c {
                    Key::Char('q') => break,
                    Key::Char('r') => break,
                    _ => (),
                }
            }
        }
    });

    let stdin_channel = rx;

    let mut restart = false;

    'main: loop {
        let now = std::time::Instant::now();

        player.step += player.step_speed;
        let mut step_max = player.step;

        for en in enemies.iter_mut() {
            en.step += en.step_speed;
            if en.step > step_max {
                step_max = en.step;
            }
        }

        if player.step >= step_max {
            player.step = 0.0;

            loop {
                let now = std::time::Instant::now();

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
                            _ => (),
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

                let made_action = if v_x.abs() + v_y.abs() > 0 {
                    player.action(&mut blood_eff, (v_x, v_y), &field, &mut enemies)
                } else {
                    false
                };

                frame(
                    &mut stdout,
                    &mut screen,
                    &mut blood_eff,
                    &field,
                    (f_x, f_y),
                    &player,
                    &enemies,
                );

                sleep(i64::max(100 - now.elapsed().as_millis() as i64, 0) as u64);

                if made_action {
                    break;
                }
            }
        }

        for i in 0..enemies.len() {
            let mut en = enemies[i];
            if en.step < step_max {
                continue;
            }
            en.step = 0.0;
            en.process(&mut player, &field, &mut blood_eff, &enemies);
            enemies[i] = en;
        }

        frame(
            &mut stdout,
            &mut screen,
            &mut blood_eff,
            &field,
            (f_x, f_y),
            &player,
            &enemies,
        );

        sleep(i64::max(100 - now.elapsed().as_millis() as i64, 0) as u64);

        if player.dead {
            while player.lighting_power > 0 {
                frame(
                    &mut stdout,
                    &mut screen,
                    &mut blood_eff,
                    &field,
                    (f_x, f_y),
                    &player,
                    &enemies,
                );
                sleep(100);
                player.lighting_power -= 1;
                frame(
                    &mut stdout,
                    &mut screen,
                    &mut blood_eff,
                    &field,
                    (f_x, f_y),
                    &player,
                    &enemies,
                );
                sleep(100);
            }

            write!(
                stdout,
                "{}{}",
                termion::clear::All,
                termion::cursor::Goto(1, 1),
            )
            .unwrap();
            write!(
                stdout,
                "{}
                \r
               :|hwh     .m::whX  \r
             X*#ma$     X mUUUUUUww:  \r
                  .h    U UUUUUUUUUU:X \r
                        U #UUUUUUUUUU:X \r
               h -     |UX .RUUUUUUUUmm \r
                     XwUUU|  .UUUUUURmm \r
                    m T#UUUUwX..#mRRmmm \r
            .wu:iw*      #UUUU:    ..   \r
          X  mUUUU         T#UT  :Uw|X| \r
         %    #UUUm             .UUUUUU\r
               TUUUU8::    :ww     ##* \r
        -        .T#UUaawa*.UU      / \r
Uaam                  X|wUw            \r
      :%      h     wmUUUUTi     w|n+  \r
     X     .h  u  UUUBUUU w | TUUm \r
   Xa       .awTwo( *UUUwUThU   \r
i   XU.         .UUUBUwu( **URm \r
Rai               UUUUUBUUen",
                termion::cursor::Goto(1, screen_height as u16 - 19),
            )
            .unwrap();
            stdout.flush().unwrap();

            sleep(300);

            write!(
                stdout,
                "{}{}Kills: {}\n\rPress q to exit\n\r      r to restart",
                termion::clear::All,
                termion::cursor::Goto(1, 1),
                start_enemies - enemies.len(),
            )
            .unwrap();
            write!(
                stdout,
                "{}
                      :::!~!!!!!:.\r
                  .xUHWH!! !!?M88WHX:.\r
                .X*#M@$!!  !X!M$$$$$$WWx:.\r
               :!!!!!!?H! :!$!$$$$$$$$$$8X:\r
              !!~  ~:~!! :~!$!#$$$$$$$$$$8X:\r
             :!~::!H!<   ~.U$X!?R$$$$$$$$MM!\r
             ~!~!!!!~~ .:XW$$$U!!?$$$$$$RMM!\r
               !:~~~ .:!M\"T#$$$$WX??#MRRMMM!\r
               ~?WuxiW*`   `\"#$$$$8!!!!??!!!\r
             :X- M$$$$       `\"T#$T~!8$WUXU~\r
            :%`  ~#$$$m:        ~!~ ?$$$$$$\r
          :!`.-   ~T$$$$8xx.  .xWW- ~\"\"##*\"\r
.....   -~~:<` !    ~?T#$$@@W@*?$$      /`\r
W$@@M!!! .!~~ !!     .:XUW$W!~ `\"~:    :\r
#\"~~`.:x%`!!  !H:   !WM$$$$Ti.: .!WUn+!`\r
:::~:!!`:X~ .: ?H.!u \"$$$B$$$!W:U!T$$M~\r
.~~   :X@!.-~   ?@WTWo(\"*$$$W$TH$! `\r
Wi.~!X$?!-~    : ?$$$B$Wu(\"**$RM!\r
$R@i.~~ !     :   ~$$$$$B$$en:``\r
?MXT@Wx.~    :     ~\"##*$$$$M~",
                termion::cursor::Goto(1, screen_height as u16 - 20),
            )
            .unwrap();
            stdout.flush().unwrap();

            loop {
                loop {
                    match stdin_channel.try_recv() {
                        Ok(key) => match key {
                            Key::Char('q') => break 'main,
                            Key::Char('r') => {
                                restart = true;
                                break 'main;
                            }
                            _ => (),
                        },
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => break 'main,
                    }
                }
                sleep(100);
            }
        }
    }

    stdin_process.join().expect("Error join stdin process");

    write!(stdout, "{}", termion::cursor::Show).unwrap();

    restart
}

mod colors;
mod draw;
mod field;
mod player;
mod state;
use crate::state::State;
use bracket_terminal::prelude::*;

bracket_terminal::embedded_resource!(TILE_FONT, "../resources/vga8x16.png");

fn build_context() -> Result<BTerm, Box<dyn std::error::Error + Send + Sync>> {
    bracket_terminal::link_resource!(TILE_FONT, "resources/vga8x16.png");

    BTermBuilder::new()
        .with_dimensions(80, 25)
        .with_tile_dimensions(12, 24)
        .with_title("Rogue game")
        .with_font("vga8x16.png", 8, 16)
        .with_simple_console(80, 25, "vga8x16.png")
        .build()
}

fn main() -> BError {
    let context = build_context()?;

    let gs = State::new();
    main_loop(context, gs)
}

// fn main() {
//     loop {
//         if !game() {
//             break;
//         }
//     }
// }

fn sleep(millis: u64) {
    if millis == 0 {
        return;
    }
    let duration = time::Duration::from_millis(millis);
    thread::sleep(duration);
}
