# ᚱ Runefall

**Runefall** is an ultra-lightweight, high-performance terminal screensaver written in Rust. Inspired by classics like `unimatrix` and `cmatrix`, it brings an ancient, mystical aesthetic to your terminal using authentic runic characters and vibrant color palettes.

Designed with efficiency in mind, **Runefall** consumes minimal CPU resources (typically < 2%) while providing a rich, interactive visual experience.

---

## ✨ Features

- **Authentic Runic Alphabets**: Choose between Elder Futhark, Younger Futhark, Anglo-Saxon Futhorc, Ogham, or Mystic symbols.
- **Dynamic Visuals**: Characters shimmer and mutate as they fall, creating a living "rain" effect.
- **Uniform Distribution**: Advanced logic ensures the "rain" covers the entire screen evenly, avoiding static patterns or empty columns.
- **Interactive Controls**: Adjust speed, density, runes, and colors on the fly without restarting.
- **Blinking Rainbow Mode**: A chaotic, high-contrast mode inspired by the classic `cmatrix` blinking effect.
- **Discrete HUD**: A sleek, auto-hiding status bar keeps you informed of your current settings.

---

## 🚀 Installation & Running

### Prerequisites
- [Rust & Cargo](https://rustup.rs/)

### Build
To get the best performance (and lowest CPU usage), always compile in **release mode**:

```bash
cargo build --release
```

### Run
```bash
./target/release/runefall
```

---

## ⌨️ Interactive Controls (Hotkeys)

While the screensaver is running, you can use the following keys to customize your experience in real-time:

### 🛡️ Runic Sets
*   `a` : **All** (Mixed sets - Default)
*   `e` : **Elder Futhark** (Traditional Germanic)
*   `y` : **Younger Futhark** (Scandinavian/Viking)
*   `s` : **Anglo-Saxon** (Futhorc)
*   `o` : **Ogham** (Early Irish)
*   `m` : **Mystic** (Alchemical & Celestial symbols)

### 🎨 Color Palettes
*   `1` : **Arcane** (Mysterious Purples & Magentas)
*   `2` : **Emerald** (Vibrant Greens)
*   `3` : **Frost** (Glacial Blues & Cyans)
*   `4` : **Ember** (Fiery Oranges & Reds)
*   `5` : **Rainbow** (Smooth spectral gradients)
*   `0` : **Blinking Rainbow** (High-intensity chaotic bursts)

### ⚙️ Simulation Controls
*   `+` or `=` : **Increase Speed** (Higher FPS)
*   `-` : **Decrease Speed** (Lower FPS)
*   `]` : **Increase Density** (More columns of rain)
*   `[` : **Decrease Density** (Fewer columns of rain)
*   `i` : **Toggle Status Bar** (Force ON/OFF)
*   `Up Arrow` : **Scroll Up**
*   `Down Arrow` : **Scroll Down** (Default)
*   `Left Arrow` : **Scroll Left**
*   `Right Arrow` : **Scroll Right**
*   `q` or `Esc` : **Quit**

---

## 🛠️ CLI Options

You can also launch **Runefall** with specific settings using command-line arguments:

| Option | Shorthand | Description | Default |
| :--- | :--- | :--- | :--- |
| `--palette` | `-p` | Set initial color theme | `arcane` |
| `--fps` | `-f` | Frame rate limit (5-60) | `20` |
| `--density` | `-d` | Column density (0.1 - 1.0) | `0.4` |
| `--help` | `-h` | Show help and exit | - |

**Example:**
```bash
# Start with green runes at high density and 30 FPS
./target/release/runefall -p emerald -d 0.7 -f 30
```

---

## 🎨 Design Philosophy

**Runefall** uses a custom gradient engine that calculates character intensity and color on every tick. The "head" of each runic stream is rendered with a high-intensity glow, while the "tail" fades into the darkness of your terminal background.

The codebase is a single-threaded, non-blocking event loop that leverages `crossterm` for cross-platform terminal manipulation. By using `thread::sleep` carefully calculated against frame duration, we ensure that your fans won't start spinning while you're enjoying the runic rain.

---

## 📜 License

MIT License - Feel free to use, modify, and distribute as you see fit.
