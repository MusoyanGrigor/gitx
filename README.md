# GitX: Modern Git Tree Explorer 🌳

GitX is a fast, terminal-based Git workflow tool built for developers who work with complex branch structures and large commit histories. It provides an intuitive way to explore repositories while acting as a seamless wrapper around standard Git commands.

---

## ✨ Features

* 🌳 **Interactive Tree Explorer**
  Navigate large commit histories with smooth keyboard-driven controls.

* 🔀 **Branch Comparison**
  Visualize differences between branches, including unique commits and merge bases.

* 🔎 **Jump to Reference**
  Instantly locate commits, branches, or tags.

* ⚡ **Git Passthrough**
  Use `gitx` as a drop-in replacement for `git`:

  ```bash
  gitx status
  gitx commit
  gitx push
  ```

* 🧱 **Modern Architecture**
  Built in Rust with modular design for future TUI/GUI and plugin support.

---

## 🛠 Installation

### 📦 Requirements

* [Rust](https://rust-lang.org) (v1.70+)
* Git installed on your system

---

### 🐧 Linux / 🍎 macOS

```bash
git clone https://github.com/MusoyanGrigor/gitx
cd gitx
cargo build --release

# Move binary to PATH (optional)
cp target/release/gitx /usr/local/bin/
```

---

### 🪟 Windows

#### Option 1: Using PowerShell

```powershell
git clone https://github.com/MusoyanGrigor/gitx
cd gitx
cargo build --release
```

Binary will be located at:

```
target\release\gitx.exe
```

You can:

* Run it directly
* Or add `target\release` to your PATH

---

#### Option 2: Install globally (recommended)

Move binary to a directory in your PATH, e.g.:

```powershell
move target\release\gitx.exe C:\Windows\System32\
```

(Or any custom folder already in PATH)

---

## 📟 Usage

### 🌳 Explorer Mode

```bash
gitx tree
gitx tree --filter "refactor"
```

#### Keybindings

* `j` / `↓` — Move down
* `k` / `↑` — Move up
* `d` — Toggle detail pane
* `/` or `f` — Search/filter
* `J` — Jump to reference
* `Esc` — Clear filter
* `q` — Quit

---

### 🔀 Branch Comparison

```bash
gitx compare main feature/cool-stuff
```

---

### 🔎 Jump to Reference

```bash
gitx jump v1.0.0
```

---

### ⚡ Git Passthrough

```bash
gitx status
gitx add .
gitx commit -m "message"
gitx push origin main
```

---

## 🏗 Architecture

GitX is built with a modular architecture for scalability:

* **core** — Git logic using `git2`
* **models** — Domain entities (commits, branches, diffs)
* **tui** — Terminal UI (`ratatui` + `crossterm`)
* **forwarding** — Git command passthrough
* **utils** — Shared helpers

---

## 🤝 Contributing

Contributions are welcome!
Feel free to open issues, suggest features, or submit pull requests.

---

## 📄 License

This project is open source and available under the **MIT License**.
