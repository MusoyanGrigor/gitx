# GitX: Modern Git Tree Explorer 🌳

GitX is a fast, terminal-based Git workflow tool built for developers who work with complex branch structures and large commit histories. It provides an intuitive way to explore repositories while acting as a seamless wrapper around standard Git commands.

<<<<<<< HEAD
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
---
=======
## ✨ Features
- **🌳 Interactive Tree Explorer** Navigate large commit histories with smooth keyboard-driven controls or beautiful IDE-style CLI graphs.
- **⏱️ Timeline Viewer** Chronological, filtered log views with full branch connectivity.
- **🔀 Branch Comparison** Visualize differences between branches, including unique commits and merge bases.
- **🔎 Jump to Reference** Instantly locate commits, branches, or tags.
- **⏪ Safe Undo** A robust safety net to easily unstage, discard, or reverse commits without memorizing complex Git commands.
- **⚡ Git Passthrough** Use `gitx` as a drop-in replacement for git:
  - `gitx status`
  - `gitx commit`
  - `gitx push`
>>>>>>> 78ef7d9 (docs: update README with new features, installation guides, and usage examples)

## 🛠 Installation

### 📦 Requirements
<<<<<<< HEAD

* [Rust](https://rust-lang.org) (v1.70+)
* Git installed on your system

---

### 🐧 Linux / 🍎 macOS
=======
- Rust (v1.70+)
- Git installed on your system
>>>>>>> 78ef7d9 (docs: update README with new features, installation guides, and usage examples)

### 🐧 Linux / 🍎 macOS
```bash
git clone https://github.com/MusoyanGrigor/gitx
cd gitx
cargo build --release

# Move binary to PATH (optional)
cp target/release/gitx /usr/local/bin/

# Alternatively, install globally using Cargo:
cargo install --path .
```

<<<<<<< HEAD
---

### 🪟 Windows

#### Option 1: Using PowerShell

=======
### 🪟 Windows

**Option 1: Using PowerShell**
>>>>>>> 78ef7d9 (docs: update README with new features, installation guides, and usage examples)
```powershell
git clone https://github.com/MusoyanGrigor/gitx
cd gitx
cargo build --release
```
<<<<<<< HEAD

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
=======
Binary will be located at: `target\release\gitx.exe`

You can:
- Run it directly
- Or add `target\release` to your PATH

**Option 2: Install globally (recommended)**
Move binary to a directory in your PATH, e.g.:
```powershell
move target\release\gitx.exe C:\Windows\System32\
```
*(Or any custom folder already in PATH)*
>>>>>>> 78ef7d9 (docs: update README with new features, installation guides, and usage examples)

## 📟 Usage

### 🌳 Explorer Mode
<<<<<<< HEAD

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

=======
```bash
gitx tree
gitx tree --cli --limit 20
gitx tree --filter "refactor"
```
**Keybindings (TUI):**
- `j` / `↓` — Move down
- `k` / `↑` — Move up
- `d` — Toggle detail pane
- `/` or `f` — Search/filter
- `J` — Jump to reference
- `Esc` — Clear filter
- `q` — Quit

### ⏱️ Timeline History
Chronological, terminal-optimized log viewing with smart graph alignment.
```bash
gitx timeline
gitx timeline --author "MusoyanGrigor"
gitx timeline --message "fix branch"
gitx timeline --limit 10
gitx timeline --no-merges
gitx timeline --merges
```

### ⏪ Safe Undo System
Provides a protective layer for common repository restructures.
```bash
gitx undo status              # Show what can be safely undone right now
gitx undo unstage             # Unstage staged changes (git reset)
gitx undo discard             # Discard unstaged changes entirely (git restore)
gitx undo clean -d -x         # Remove untracked files and ignored directories
gitx undo last-commit --soft  # Undo last commit but keep changes staged
gitx undo all                 # Nuclear option: restore entirely clean repo state
```

### 🔀 Branch Comparison
>>>>>>> 78ef7d9 (docs: update README with new features, installation guides, and usage examples)
```bash
gitx compare main feature/cool-stuff
```

<<<<<<< HEAD
---

### 🔎 Jump to Reference

=======
### 🔎 Jump to Reference
>>>>>>> 78ef7d9 (docs: update README with new features, installation guides, and usage examples)
```bash
gitx jump v1.0.0
```

<<<<<<< HEAD
---

### ⚡ Git Passthrough

=======
### ⚡ Git Passthrough
You can use `gitx` just like `git`:
>>>>>>> 78ef7d9 (docs: update README with new features, installation guides, and usage examples)
```bash
gitx status
gitx add .
gitx commit -m "message"
gitx push origin main
```

<<<<<<< HEAD
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
=======
## 🏗 Architecture
GitX is built with a modular architecture for scalability:
- `core` — Git logic using `git2`
- `models` — Domain entities (commits, branches, diffs)
- `tui` — Terminal UI (`ratatui` + `crossterm`)
- `commands` — CLI parsing and subcommands integration
- `forwarding` — Git command passthrough
- `utils` — Shared graphical and theme helpers

## 🤝 Contributing
Contributions are welcome! Feel free to open issues, suggest features, or submit pull requests.

## 📄 License
This project is open source and available under the MIT License.
>>>>>>> 78ef7d9 (docs: update README with new features, installation guides, and usage examples)
