# GitX: Modern Git Tree Explorer 🌳

GitX is a fast, terminal-based Git workflow tool built for developers who work with complex branch structures and large commit histories. It provides an intuitive way to explore repositories while acting as a seamless wrapper around standard Git commands.

---

## ✨ Features

* 🌳 **Interactive Tree Explorer**
  Navigate large commit histories with smooth keyboard-driven controls or CLI graph views.

* ⏱️ **Timeline Viewer**
  Chronological, filtered log views with clean branch visualization.

* 🔀 **Branch Comparison**
  Visualize differences between branches, including unique commits and merge bases.

* 🔎 **Jump to Reference**
  Instantly locate commits, branches, or tags.

* ⏪ **Safe Undo System**
  Easily undo changes without memorizing complex Git commands.

* ⚡ **Git Passthrough**
  Use `gitx` as a drop-in replacement for `git`:

  ```bash
  gitx status
  gitx commit
  gitx push
  ```

---

## 🛠 Installation

### 📦 Requirements

* Rust (v1.70+)
* Git installed

---

### 🐧 Linux / 🍎 macOS

```bash
git clone https://github.com/MusoyanGrigor/gitx
cd gitx
cargo build --release

# Move binary to PATH (optional)
cp target/release/gitx /usr/local/bin/

# Or install globally
cargo install --path .
```

---

### 🪟 Windows

#### PowerShell

```powershell
git clone https://github.com/MusoyanGrigor/gitx
cd gitx
cargo build --release
```

Binary location:

```
target\release\gitx.exe
```

You can:

* Run it directly
* Or add `target\release` to PATH

#### Install globally

```powershell
move target\release\gitx.exe C:\Windows\System32\
```

---

## 📟 Usage

### 🌳 Explorer Mode

```bash
gitx tree
gitx tree --filter "refactor"
gitx tree --cli --limit 20
```

#### Keybindings (TUI)

* `j` / `↓` — Move down
* `k` / `↑` — Move up
* `d` — Toggle detail pane
* `/` or `f` — Search/filter
* `J` — Jump to reference
* `Esc` — Clear filter
* `q` — Quit

---

### ⏱️ Timeline

```bash
gitx timeline
gitx timeline --author "your-name"
gitx timeline --message "fix"
gitx timeline --limit 10
gitx timeline --no-merges
gitx timeline --merges
```

---

### ⏪ Undo System

```bash
gitx undo status
gitx undo unstage
gitx undo discard
gitx undo clean -d -x
gitx undo last-commit --soft
gitx undo all
```

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

GitX is built with a modular architecture:

* `core` — Git logic using `git2`
* `models` — Domain entities
* `tui` — Terminal UI (`ratatui` + `crossterm`)
* `commands` — CLI parsing and commands
* `forwarding` — Git passthrough
* `utils` — Shared helpers

---

## 🤝 Contributing

Contributions are welcome!
Open issues, suggest features, or submit pull requests 🚀

---

## 📄 License

This project is open source and licensed under the **MIT License**.
