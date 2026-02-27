use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{self, Color, SetForegroundColor},
    terminal::{self, ClearType},
};
use rand::Rng;
use std::io::{self, Write};
use std::time::{Duration, Instant};

// ── Runic character sets ──────────────────────────────────────────────

const ELDER_FUTHARK: &[char] = &[
    'ᚠ', 'ᚢ', 'ᚦ', 'ᚨ', 'ᚱ', 'ᚲ', 'ᚷ', 'ᚹ', 'ᚺ', 'ᚾ', 'ᛁ', 'ᛃ', 'ᛇ', 'ᛈ', 'ᛉ', 'ᛊ', 'ᛋ', 'ᛏ', 'ᛒ',
    'ᛖ', 'ᛗ', 'ᛚ', 'ᛜ', 'ᛝ', 'ᛞ', 'ᛟ',
];

const YOUNGER_FUTHARK: &[char] = &[
    'ᚠ', 'ᚢ', 'ᚦ', 'ᚬ', 'ᚱ', 'ᚴ', 'ᚼ', 'ᚾ', 'ᛁ', 'ᛅ', 'ᛋ', 'ᛏ', 'ᛒ', 'ᛘ', 'ᛚ', 'ᛦ',
];

const ANGLO_SAXON: &[char] = &[
    'ᚠ', 'ᚢ', 'ᚦ', 'ᚩ', 'ᚱ', 'ᚳ', 'ᚷ', 'ᚹ', 'ᚻ', 'ᚾ', 'ᛁ', 'ᛄ', 'ᛇ', 'ᛈ', 'ᛉ', 'ᛋ', 'ᛏ', 'ᛒ', 'ᛖ',
    'ᛗ', 'ᛚ', 'ᛝ', 'ᛟ', 'ᛡ', 'ᛣ', 'ᛥ',
];

const OGHAM: &[char] = &[
    'ᚁ', 'ᚂ', 'ᚃ', 'ᚄ', 'ᚅ', 'ᚆ', 'ᚇ', 'ᚈ', 'ᚉ', 'ᚊ', 'ᚋ', 'ᚌ', 'ᚍ', 'ᚎ', 'ᚏ', 'ᚐ', 'ᚑ', 'ᚒ', 'ᚓ',
    'ᚔ', 'ᚕ', 'ᚖ', 'ᚗ', 'ᚘ', 'ᚙ', 'ᚚ',
];

const MYSTIC: &[char] = &[
    '☽', '☾', '✧', '✦', '◈', '◇', '⁂', '⊕', '⊗', '⊛', '⌘', '⍟', '♅', '♆', '♇', '⚝', '✡', '⬡', '⬢',
    '⏣', '⏥', '◉', '◎', '⦿',
];

#[derive(Clone, Copy, PartialEq)]
enum RuneSet {
    All,
    Elder,
    Younger,
    Anglo,
    Ogham,
    Mystic,
}

impl RuneSet {
    fn name(&self) -> &'static str {
        match self {
            RuneSet::All => "All",
            RuneSet::Elder => "Elder Futhark",
            RuneSet::Younger => "Younger Futhark",
            RuneSet::Anglo => "Anglo-Saxon",
            RuneSet::Ogham => "Ogham",
            RuneSet::Mystic => "Mystic",
        }
    }
}

fn random_rune(rng: &mut impl Rng, set: RuneSet) -> char {
    let chosen_set = match set {
        RuneSet::All => {
            let all_sets: &[&[char]] =
                &[ELDER_FUTHARK, YOUNGER_FUTHARK, ANGLO_SAXON, OGHAM, MYSTIC];
            all_sets[rng.gen_range(0..all_sets.len())]
        }
        RuneSet::Elder => ELDER_FUTHARK,
        RuneSet::Younger => YOUNGER_FUTHARK,
        RuneSet::Anglo => ANGLO_SAXON,
        RuneSet::Ogham => OGHAM,
        RuneSet::Mystic => MYSTIC,
    };
    chosen_set[rng.gen_range(0..chosen_set.len())]
}

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Down,
    Up,
    Left,
    Right,
}

impl Direction {
    fn max_lanes(&self, cols: u16, rows: u16) -> u16 {
        match self {
            Direction::Down | Direction::Up => cols,
            Direction::Left | Direction::Right => rows,
        }
    }

    fn max_pos(&self, cols: u16, rows: u16) -> u16 {
        match self {
            Direction::Down | Direction::Up => rows,
            Direction::Left | Direction::Right => cols,
        }
    }

    // Convert abstract (lane, pos) to screen (x, y)
    fn to_screen(&self, lane: u16, pos: i32, cols: u16, rows: u16) -> Option<(u16, u16)> {
        match self {
            Direction::Down => {
                if pos >= 0 && pos < rows as i32 {
                    Some((lane, pos as u16))
                } else {
                    None
                }
            }
            Direction::Up => {
                if pos >= 0 && pos < rows as i32 {
                    Some((lane, rows.saturating_sub(1).saturating_sub(pos as u16)))
                } else {
                    None
                }
            }
            Direction::Right => {
                if pos >= 0 && pos < cols as i32 {
                    Some((pos as u16, lane))
                } else {
                    None
                }
            }
            Direction::Left => {
                if pos >= 0 && pos < cols as i32 {
                    Some((cols.saturating_sub(1).saturating_sub(pos as u16), lane))
                } else {
                    None
                }
            }
        }
    }
}

// ── Color palettes ────────────────────────────────────────────────────

#[derive(Clone, Copy)]
enum Palette {
    Arcane,
    Emerald,
    Frost,
    Ember,
    Rainbow,
    BlinkingRainbow,
}

impl Palette {
    fn name(&self) -> &'static str {
        match self {
            Palette::Arcane => "Arcane",
            Palette::Emerald => "Emerald",
            Palette::Frost => "Frost",
            Palette::Ember => "Ember",
            Palette::Rainbow => "Rainbow",
            Palette::BlinkingRainbow => "Blink",
        }
    }

    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "emerald" | "green" => Palette::Emerald,
            "frost" | "blue" | "cyan" => Palette::Frost,
            "ember" | "red" | "fire" => Palette::Ember,
            "rainbow" | "multi" => Palette::Rainbow,
            "blinking" | "blink" | "cmatrix" => Palette::BlinkingRainbow,
            _ => Palette::Arcane,
        }
    }

    /// Return a color for a trail cell. `intensity` goes from 1.0 (head) to 0.0 (tail).
    /// `column_seed` is used for rainbow hue offset.
    fn color(&self, intensity: f32, column_seed: u8, global_tick: u64, coordinate: i32) -> Color {
        let i = intensity.clamp(0.0, 1.0);
        match self {
            Palette::Arcane => {
                // Purple/magenta gradient — bright magenta head → deep indigo tail
                let r = (180.0 * i + 40.0 * (1.0 - i)) as u8;
                let g = (60.0 * i + 10.0 * (1.0 - i)) as u8;
                let b = (255.0 * i + 80.0 * (1.0 - i)) as u8;
                Color::Rgb { r, g, b }
            }
            Palette::Emerald => {
                let r = (50.0 * i) as u8;
                let g = (255.0 * i + 30.0 * (1.0 - i)) as u8;
                let b = (80.0 * i + 10.0 * (1.0 - i)) as u8;
                Color::Rgb { r, g, b }
            }
            Palette::Frost => {
                let r = (100.0 * i) as u8;
                let g = (200.0 * i + 40.0 * (1.0 - i)) as u8;
                let b = (255.0 * i + 60.0 * (1.0 - i)) as u8;
                Color::Rgb { r, g, b }
            }
            Palette::Ember => {
                let r = (255.0 * i + 60.0 * (1.0 - i)) as u8;
                let g = (120.0 * i * i) as u8; // quadratic for warm glow
                let b = (30.0 * i) as u8;
                Color::Rgb { r, g, b }
            }
            Palette::Rainbow => {
                // Rotate hue based on column_seed + intensity
                let hue = ((column_seed as f32 / 255.0) * 360.0 + intensity * 60.0) % 360.0;
                let (r, g, b) = hsl_to_rgb(hue, 0.9, 0.25 + 0.45 * i);
                Color::Rgb { r, g, b }
            }
            Palette::BlinkingRainbow => {
                // Highly saturated random hue based on coordinate and time for extreme blinking
                let pseudo = (global_tick
                    .wrapping_add(coordinate as u64)
                    .wrapping_add(column_seed as u64))
                .wrapping_mul(1103515245);
                let hue = (pseudo % 360) as f32;
                let (r, g, b) = hsl_to_rgb(hue, 1.0, 0.4 + 0.3 * i);
                Color::Rgb { r, g, b }
            }
        }
    }
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hp = h / 60.0;
    let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match hp as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    (
        ((r1 + m) * 255.0) as u8,
        ((g1 + m) * 255.0) as u8,
        ((b1 + m) * 255.0) as u8,
    )
}

// ── Stream (rain drop) ───────────────────────────────────────────────

struct Stream {
    lane: u16,
    pos: i32, // head position along the direction (increases over time)
    speed: u8,
    tick_counter: u8,
    trail_len: u16,
    color_seed: u8,
    active: bool,
    chars: Vec<char>,
}

impl Stream {
    fn new(lane: u16, max_pos: u16, rng: &mut impl Rng, rune_set: RuneSet) -> Self {
        let trail_len = rng.gen_range(4..=max_pos.saturating_sub(2).max(6));
        let speed = rng.gen_range(1..=4_u8);
        let mut chars = Vec::with_capacity(trail_len as usize);
        for _ in 0..trail_len {
            chars.push(random_rune(rng, rune_set));
        }
        Stream {
            lane,
            pos: -(rng.gen_range(0..(max_pos as i32).max(1))),
            speed,
            tick_counter: 0,
            trail_len,
            color_seed: rng.gen(),
            active: true,
            chars,
        }
    }

    fn reset(&mut self, lane: u16, max_pos: u16, rng: &mut impl Rng, rune_set: RuneSet) {
        self.lane = lane;
        self.pos = -(rng.gen_range(0..(max_pos as i32).max(1)));
        self.speed = rng.gen_range(1..=4);
        self.tick_counter = 0;
        self.trail_len = rng.gen_range(4..=max_pos.saturating_sub(2).max(6));
        self.color_seed = rng.gen();
        self.chars.clear();
        for _ in 0..self.trail_len {
            self.chars.push(random_rune(rng, rune_set));
        }
        self.active = true;
    }

    fn tick(&mut self, max_pos: u16, rng: &mut impl Rng, rune_set: RuneSet) {
        self.tick_counter += 1;
        if self.tick_counter >= self.speed {
            self.tick_counter = 0;
            self.pos += 1;

            if !self.chars.is_empty() && rng.gen_ratio(1, 5) {
                let idx = rng.gen_range(0..self.chars.len());
                self.chars[idx] = random_rune(rng, rune_set);
            }

            if self.pos - self.trail_len as i32 > max_pos as i32 {
                self.active = false;
            }
        }
    }
}

// ── Rendering ─────────────────────────────────────────────────────────

struct Renderer {
    cols: u16,
    rows: u16,
    direction: Direction,
    streams: Vec<Stream>,
    palette: Palette,
    rune_set: RuneSet,
    density: f32, // fraction of max lanes that have active rain
    global_tick: u64,
    show_status: bool,
    status_timer: u64, // ticks remaining to show status
    status_clear_needed: bool,
    fps: u64,
}

impl Renderer {
    fn new(palette: Palette, density: f32, fps: u64) -> io::Result<Self> {
        let (cols, rows) = terminal::size()?;
        let direction = Direction::Down;
        let rune_set = RuneSet::All;

        let mut renderer = Renderer {
            cols,
            rows,
            direction,
            streams: Vec::new(),
            palette,
            rune_set,
            density,
            global_tick: 0,
            show_status: true,
            status_timer: fps * 3,
            status_clear_needed: false,
            fps,
        };

        renderer.resize(cols, rows);
        Ok(renderer)
    }

    fn resize(&mut self, new_cols: u16, new_rows: u16) {
        self.cols = new_cols;
        self.rows = new_rows;
        let mut rng = rand::thread_rng();
        let max_lanes = self.direction.max_lanes(self.cols, self.rows);
        let max_pos = self.direction.max_pos(self.cols, self.rows);
        let target = ((max_lanes as f32 * self.density) as usize).max(1);

        self.streams.clear();
        let mut available: Vec<u16> = (0..max_lanes).collect();
        for _ in 0..target.min(max_lanes as usize) {
            if available.is_empty() {
                break;
            }
            let idx = rng.gen_range(0..available.len());
            let lane = available.swap_remove(idx);
            self.streams
                .push(Stream::new(lane, max_pos, &mut rng, self.rune_set));
        }
    }

    fn tick(&mut self) {
        self.global_tick = self.global_tick.wrapping_add(1);
        if self.status_timer > 0 {
            self.status_timer -= 1;
            if self.status_timer == 0 {
                self.status_clear_needed = true;
            }
        }

        let mut rng = rand::thread_rng();
        let max_lanes = self.direction.max_lanes(self.cols, self.rows);
        let max_pos = self.direction.max_pos(self.cols, self.rows);

        let mut occupied = vec![false; max_lanes as usize];
        for stream in &mut self.streams {
            stream.tick(max_pos, &mut rng, self.rune_set);
            if stream.active && (stream.lane as usize) < occupied.len() {
                occupied[stream.lane as usize] = true;
            }
        }

        let mut free_lanes: Vec<u16> = (0..max_lanes).filter(|&l| !occupied[l as usize]).collect();

        for stream in &mut self.streams {
            if !stream.active {
                let new_lane = if !free_lanes.is_empty() {
                    let idx = rng.gen_range(0..free_lanes.len());
                    free_lanes.swap_remove(idx)
                } else {
                    rng.gen_range(0..max_lanes.max(1))
                };

                stream.reset(new_lane, max_pos, &mut rng, self.rune_set);
                if (new_lane as usize) < occupied.len() {
                    occupied[new_lane as usize] = true;
                }
            }
        }
    }

    fn change_density(&mut self, delta: f32) {
        self.density = (self.density + delta).clamp(0.05, 1.0);
        self.resize(self.cols, self.rows);
    }

    fn change_direction(&mut self, new_dir: Direction) {
        if self.direction != new_dir {
            self.direction = new_dir;
            self.status_clear_needed = true;
            self.resize(self.cols, self.rows);
        }
    }

    fn poke_status(&mut self) {
        self.status_timer = self.fps * 3; // show for 3 seconds
        self.status_clear_needed = false;
    }

    fn render(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        for stream in &self.streams {
            if !stream.active {
                continue;
            }
            for i in 0..stream.trail_len as i32 {
                let current_pos = stream.pos - i;
                if let Some((x, y)) =
                    self.direction
                        .to_screen(stream.lane, current_pos, self.cols, self.rows)
                {
                    let intensity = 1.0 - (i as f32 / stream.trail_len as f32);
                    let color = self.palette.color(
                        intensity,
                        stream.color_seed,
                        self.global_tick,
                        stream.pos,
                    );
                    let ch = stream.chars.get(i as usize).copied().unwrap_or('ᚠ');

                    queue!(
                        stdout,
                        cursor::MoveTo(x, y),
                        SetForegroundColor(color),
                        style::Print(ch)
                    )?;
                }
            }

            // Erase old trail
            if let Some((x, y)) = self.direction.to_screen(
                stream.lane,
                stream.pos - stream.trail_len as i32,
                self.cols,
                self.rows,
            ) {
                queue!(stdout, cursor::MoveTo(x, y), style::Print(' '))?;
            }

            // Head glow
            if let Some((x, y)) =
                self.direction
                    .to_screen(stream.lane, stream.pos, self.cols, self.rows)
            {
                let head_color = match self.palette {
                    Palette::Arcane => Color::Rgb {
                        r: 230,
                        g: 180,
                        b: 255,
                    },
                    Palette::Emerald => Color::Rgb {
                        r: 180,
                        g: 255,
                        b: 200,
                    },
                    Palette::Frost => Color::Rgb {
                        r: 200,
                        g: 240,
                        b: 255,
                    },
                    Palette::Ember => Color::Rgb {
                        r: 255,
                        g: 220,
                        b: 150,
                    },
                    Palette::Rainbow => Color::Rgb {
                        r: 255,
                        g: 255,
                        b: 255,
                    },
                    Palette::BlinkingRainbow => {
                        let pseudo = (self
                            .global_tick
                            .wrapping_add(stream.pos as u64)
                            .wrapping_add(stream.color_seed as u64))
                        .wrapping_mul(1103515245);
                        let hue = (pseudo % 360) as f32;
                        let (r, g, b) = hsl_to_rgb(hue, 1.0, 0.8);
                        Color::Rgb { r, g, b }
                    }
                };
                let head_ch = stream.chars.first().copied().unwrap_or('ᛟ');
                queue!(
                    stdout,
                    cursor::MoveTo(x, y),
                    SetForegroundColor(head_color),
                    style::Print(head_ch)
                )?;
            }
        }

        let status = format!(
            " 🔮 {} | 🎨 {} | ⚡ {} FPS | Density: {:.2} ",
            self.rune_set.name(),
            self.palette.name(),
            self.fps,
            self.density
        );

        if self.show_status && self.status_timer > 0 && self.rows > 0 {
            // Draw discrete status bar at bottom right
            let x = self.cols.saturating_sub(status.len() as u16);
            let y = self.rows - 1;

            // Fade the text slightly when it's about to disappear
            let brightness = if self.status_timer < self.fps {
                50 + (100 * self.status_timer / self.fps) as u8
            } else {
                150
            };

            queue!(
                stdout,
                cursor::MoveTo(x, y),
                SetForegroundColor(Color::Rgb {
                    r: brightness,
                    g: brightness,
                    b: brightness
                }),
                style::Print(&status)
            )?;
        } else if self.status_clear_needed && self.rows > 0 {
            let x = self.cols.saturating_sub(status.len() as u16);
            let y = self.rows - 1;
            let spaces = " ".repeat(status.len());
            queue!(stdout, cursor::MoveTo(x, y), style::Print(&spaces))?;
            self.status_clear_needed = false;
        }

        stdout.flush()
    }
}

// ── CLI parsing ───────────────────────────────────────────────────────

struct Config {
    palette: Palette,
    fps: u64,
    density: f32,
}

fn parse_args() -> Config {
    let args: Vec<String> = std::env::args().collect();
    let mut palette = Palette::Arcane;
    let mut fps: u64 = 20;
    let mut density: f32 = 0.4;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--palette" | "-p" => {
                if i + 1 < args.len() {
                    palette = Palette::from_str(&args[i + 1]);
                    i += 1;
                }
            }
            "--fps" | "-f" => {
                if i + 1 < args.len() {
                    fps = args[i + 1].parse().unwrap_or(20).clamp(5, 60);
                    i += 1;
                }
            }
            "--density" | "-d" => {
                if i + 1 < args.len() {
                    density = args[i + 1].parse::<f32>().unwrap_or(0.4).clamp(0.1, 1.0);
                    i += 1;
                }
            }
            "--help" | "-h" => {
                println!("runefall — Ultra-light runic terminal screensaver");
                println!();
                println!("USAGE: runefall [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!("  -p, --palette <NAME>   Color palette: arcane, emerald, frost, ember, rainbow");
                println!("                         (default: arcane)");
                println!("  -f, --fps <N>          Target frames per second, 5-60 (default: 20)");
                println!("  -d, --density <N>      Column density 0.1-1.0 (default: 0.4)");
                println!("  -h, --help             Show this help");
                println!();
                println!("Press 'q' or Ctrl+C to exit.");
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    Config {
        palette,
        fps,
        density,
    }
}

// ── Main ──────────────────────────────────────────────────────────────

fn main() -> io::Result<()> {
    let config = parse_args();
    let frame_duration = Duration::from_millis(1000 / config.fps);

    let mut stdout = io::stdout();

    // Enter alternate screen, hide cursor, enable raw mode
    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        cursor::Hide,
        terminal::Clear(ClearType::All)
    )?;

    let mut renderer = Renderer::new(config.palette, config.density, config.fps)?;

    let result = run_loop(&mut stdout, &mut renderer, frame_duration);

    // Cleanup: always restore terminal state
    execute!(
        stdout,
        SetForegroundColor(Color::Reset),
        terminal::Clear(ClearType::All),
        cursor::Show,
        terminal::LeaveAlternateScreen
    )?;
    terminal::disable_raw_mode()?;

    result
}

fn run_loop(
    stdout: &mut io::Stdout,
    renderer: &mut Renderer,
    mut frame_duration: Duration,
) -> io::Result<()> {
    loop {
        let frame_start = Instant::now();

        // Poll for events (non-blocking)
        if event::poll(Duration::ZERO)? {
            match event::read()? {
                Event::Key(KeyEvent { code, .. }) => {
                    match code {
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('c') if event::poll(Duration::ZERO).is_ok() => {
                            // Ctrl+C is handled by raw mode, but just in case
                        }
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            let millis = frame_duration.as_millis().saturating_sub(5).max(10);
                            frame_duration = Duration::from_millis(millis as u64);
                            renderer.fps = 1000 / millis as u64;
                        }
                        KeyCode::Char('-') => {
                            let millis = frame_duration.as_millis().saturating_add(5).min(200);
                            frame_duration = Duration::from_millis(millis as u64);
                            renderer.fps = 1000 / millis as u64;
                        }

                        KeyCode::Char('[') => renderer.change_density(-0.05),
                        KeyCode::Char(']') => renderer.change_density(0.05),

                        KeyCode::Char('1') => renderer.palette = Palette::Arcane,
                        KeyCode::Char('2') => renderer.palette = Palette::Emerald,
                        KeyCode::Char('3') => renderer.palette = Palette::Frost,
                        KeyCode::Char('4') => renderer.palette = Palette::Ember,
                        KeyCode::Char('5') => renderer.palette = Palette::Rainbow,
                        KeyCode::Char('0') => renderer.palette = Palette::BlinkingRainbow,

                        // Runic sets
                        KeyCode::Char('a') => renderer.rune_set = RuneSet::All,
                        KeyCode::Char('e') => renderer.rune_set = RuneSet::Elder,
                        KeyCode::Char('y') => renderer.rune_set = RuneSet::Younger,
                        KeyCode::Char('s') => renderer.rune_set = RuneSet::Anglo,
                        KeyCode::Char('o') => renderer.rune_set = RuneSet::Ogham,
                        KeyCode::Char('m') => renderer.rune_set = RuneSet::Mystic,

                        // Directions
                        KeyCode::Up => {
                            execute!(stdout, terminal::Clear(ClearType::All)).ok();
                            renderer.change_direction(Direction::Up);
                        }
                        KeyCode::Down => {
                            execute!(stdout, terminal::Clear(ClearType::All)).ok();
                            renderer.change_direction(Direction::Down);
                        }
                        KeyCode::Left => {
                            execute!(stdout, terminal::Clear(ClearType::All)).ok();
                            renderer.change_direction(Direction::Left);
                        }
                        KeyCode::Right => {
                            execute!(stdout, terminal::Clear(ClearType::All)).ok();
                            renderer.change_direction(Direction::Right);
                        }

                        // UI toggles
                        KeyCode::Char('i') => {
                            renderer.show_status = !renderer.show_status;
                            if renderer.show_status {
                                renderer.poke_status();
                            } else {
                                renderer.status_clear_needed = true;
                            }
                        }

                        _ => {}
                    }
                    if code != KeyCode::Char('i') {
                        // Any other keypoke wakes up the status UI
                        renderer.poke_status();
                    }
                }
                Event::Resize(w, h) => {
                    execute!(stdout, terminal::Clear(ClearType::All))?;
                    renderer.resize(w, h);
                }
                _ => {}
            }
        }

        // Update
        renderer.tick();

        // Render
        renderer.render(stdout)?;

        // Sleep to maintain target FPS and save CPU
        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }
}
