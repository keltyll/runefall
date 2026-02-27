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
    fn color(&self, intensity: f32, column_seed: u8, global_tick: u64, row: i32) -> Color {
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
                    .wrapping_add(row as u64)
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

// ── Column (rain drop) ───────────────────────────────────────────────

struct Column {
    x: u16,
    y: i32,           // head position (can be negative = not yet visible)
    speed: u8,        // ticks between each step (1 = fastest)
    tick_counter: u8, // counts up to speed
    trail_len: u16,   // length of visible trail
    color_seed: u8,   // for rainbow palette variation
    active: bool,
    chars: Vec<char>, // pre-generated trail characters
}

impl Column {
    fn new(x: u16, rows: u16, rng: &mut impl Rng, rune_set: RuneSet) -> Self {
        let trail_len = rng.gen_range(4..=rows.saturating_sub(2).max(6));
        let speed = rng.gen_range(1..=4_u8);
        let mut chars = Vec::with_capacity(trail_len as usize);
        for _ in 0..trail_len {
            chars.push(random_rune(rng, rune_set));
        }
        Column {
            x,
            y: -(rng.gen_range(0..rows as i32)),
            speed,
            tick_counter: 0,
            trail_len,
            color_seed: rng.gen(),
            active: true,
            chars,
        }
    }

    fn reset(&mut self, x: u16, rows: u16, rng: &mut impl Rng, rune_set: RuneSet) {
        self.x = x;
        self.y = -(rng.gen_range(0..(rows as i32).max(1)));
        self.speed = rng.gen_range(1..=4);
        self.tick_counter = 0;
        self.trail_len = rng.gen_range(4..=rows.saturating_sub(2).max(6));
        self.color_seed = rng.gen();
        self.chars.clear();
        for _ in 0..self.trail_len {
            self.chars.push(random_rune(rng, rune_set));
        }
        self.active = true;
    }

    fn tick(&mut self, rows: u16, rng: &mut impl Rng, rune_set: RuneSet) {
        self.tick_counter += 1;
        if self.tick_counter >= self.speed {
            self.tick_counter = 0;
            self.y += 1;

            // Randomly mutate one character in the trail for visual shimmer
            if !self.chars.is_empty() && rng.gen_ratio(1, 5) {
                let idx = rng.gen_range(0..self.chars.len());
                self.chars[idx] = random_rune(rng, rune_set);
            }

            // If entire trail has scrolled past the bottom, reset
            if self.y - self.trail_len as i32 > rows as i32 {
                self.active = false;
            }
        }
    }
}

// ── Rendering ─────────────────────────────────────────────────────────

struct Renderer {
    cols: u16,
    rows: u16,
    columns: Vec<Column>,
    palette: Palette,
    rune_set: RuneSet,
    density: f32, // fraction of terminal columns that have active rain (0..1)
    global_tick: u64,
    show_status: bool,
    status_timer: u64, // ticks remaining to show status
    status_clear_needed: bool,
    fps: u64,
}

impl Renderer {
    fn new(palette: Palette, density: f32, fps: u64) -> io::Result<Self> {
        let (cols, rows) = terminal::size()?;
        let mut rng = rand::thread_rng();
        let active_count = ((cols as f32 * density) as usize).max(1);
        let rune_set = RuneSet::All;

        let mut columns = Vec::new();
        // Distribute rain columns across the terminal width
        let mut available: Vec<u16> = (0..cols).collect();
        for _ in 0..active_count.min(cols as usize) {
            if available.is_empty() {
                break;
            }
            let idx = rng.gen_range(0..available.len());
            let x = available.swap_remove(idx);
            columns.push(Column::new(x, rows, &mut rng, rune_set));
        }

        Ok(Renderer {
            cols,
            rows,
            columns,
            palette,
            rune_set,
            density,
            global_tick: 0,
            show_status: true,
            status_timer: fps * 3, // show for 3 seconds initially
            status_clear_needed: false,
            fps,
        })
    }

    fn resize(&mut self, new_cols: u16, new_rows: u16) {
        self.cols = new_cols;
        self.rows = new_rows;
        let mut rng = rand::thread_rng();
        let target = ((new_cols as f32 * self.density) as usize).max(1);

        // Rebuild columns
        self.columns.clear();
        let mut available: Vec<u16> = (0..new_cols).collect();
        for _ in 0..target.min(new_cols as usize) {
            if available.is_empty() {
                break;
            }
            let idx = rng.gen_range(0..available.len());
            let x = available.swap_remove(idx);
            self.columns
                .push(Column::new(x, new_rows, &mut rng, self.rune_set));
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

        // Track which columns currently have active drops to avoid spawning on top of existing ones
        let mut occupied = vec![false; self.cols as usize];
        for col in &mut self.columns {
            col.tick(self.rows, &mut rng, self.rune_set);
            if col.active && (col.x as usize) < occupied.len() {
                occupied[col.x as usize] = true;
            }
        }

        // Gather all free X coordinates
        let mut free_x: Vec<u16> = (0..self.cols).filter(|&x| !occupied[x as usize]).collect();

        // Re-activate dead columns
        for col in &mut self.columns {
            if !col.active {
                // Pick a new random X coordinate that isn't currently occupied if possible
                let new_x = if !free_x.is_empty() {
                    let idx = rng.gen_range(0..free_x.len());
                    free_x.swap_remove(idx)
                } else {
                    rng.gen_range(0..self.cols.max(1))
                };

                col.reset(new_x, self.rows, &mut rng, self.rune_set);
                if (new_x as usize) < occupied.len() {
                    occupied[new_x as usize] = true;
                }
            }
        }
    }

    fn change_density(&mut self, delta: f32) {
        self.density = (self.density + delta).clamp(0.05, 1.0);
        self.resize(self.cols, self.rows); // Rebuild columns with new density
    }

    fn poke_status(&mut self) {
        self.status_timer = self.fps * 3; // show for 3 seconds
        self.status_clear_needed = false;
    }

    fn render(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        // Render each column's trail
        for col in &self.columns {
            if !col.active {
                continue;
            }
            for i in 0..col.trail_len as i32 {
                let row = col.y - i;
                if row < 0 || row >= self.rows as i32 {
                    continue;
                }
                let intensity = 1.0 - (i as f32 / col.trail_len as f32);
                let color = self
                    .palette
                    .color(intensity, col.color_seed, self.global_tick, row);
                let ch = col.chars.get(i as usize).copied().unwrap_or('ᚠ');

                queue!(
                    stdout,
                    cursor::MoveTo(col.x, row as u16),
                    SetForegroundColor(color),
                    style::Print(ch)
                )?;
            }

            // Clear the cell just above the tail (erase old trail)
            let erase_row = col.y - col.trail_len as i32;
            if erase_row >= 0 && erase_row < self.rows as i32 {
                queue!(
                    stdout,
                    cursor::MoveTo(col.x, erase_row as u16),
                    style::Print(' ')
                )?;
            }

            // Bright white head character (glow effect)
            if col.y >= 0 && col.y < self.rows as i32 {
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
                            .wrapping_add(col.y as u64)
                            .wrapping_add(col.color_seed as u64))
                        .wrapping_mul(1103515245);
                        let hue = (pseudo % 360) as f32;
                        let (r, g, b) = hsl_to_rgb(hue, 1.0, 0.8);
                        Color::Rgb { r, g, b }
                    }
                };
                let head_ch = col.chars.first().copied().unwrap_or('ᛟ');
                queue!(
                    stdout,
                    cursor::MoveTo(col.x, col.y as u16),
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
