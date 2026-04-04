# GitX: Modern Git Tree Explorer 🌳

GitX is a powerful, terminal-based Git workflow tool designed for developers who handle complex branch topologies and need high-speed navigation and repository exploration. It doubles as a Git-compatible wrapper, forwarding standard commands to the system `git` when they aren't GitX-native.

## 🚀 Key Features

- **Interactive Tree Explorer**: Navigate through large commit histories with intuitive keyboard shortcuts.
- **Branch Comparison**: Focus on the divergence between two branches, identifying unique commits and the merge base.
- **Jump to Reference**: Instantly find and inspect specific commits, branches, or tags.
- **Git Passthrough**: Use `gitx` as your primary CLI command; standard git operations like `gitx status` or `gitx commit` are forwarded seamlessly.
- **Modern Architecture**: Built with a decoupled Rust core, ready for future TUI/GUI or plugin expansion.

## 🛠 Installation

Requirements:
- [Rust](https://rust-lang.org) (v1.70+)
- `libgit2` (linked via `git2-rs`)

```bash
git clone https://github.com/your-repo/gitx
cd gitx
cargo build --release
# Optionally move binary to PATH
cp target/release/gitx /usr/local/bin/
```

## 📟 Usage

### Explorer Mode
```bash
gitx tree
# Filter commits on launch
gitx tree --filter "refactor"
```
*Nav Keybindings*:
- `j` / `Down`: Move down
- `k` / `Up`: Move up
- `d`: Toggle detail pane
- `/` or `f`: Open search/filter prompt
- `J`: Open jump-to-ref prompt (branch, tag, or hash)
- `Esc`: Clear current filter or return to normal mode
- `q`: Quit

### Branch Comparison
```bash
gitx compare main feature/cool-stuff
```

### Jump to Ref
```bash
gitx jump v1.0.0
```

### Forwarding
```bash
gitx status
gitx add src/main.rs
gitx push origin main
```

## 🏗 Architecture Summary

GitX is designed in layered modules for extensibility:
- **`core`**: Pure Git logic using `git2`. No UI dependencies.
- **`models`**: Domain entities representing commits, branches, and diffs.
- **`tui`**: Terminal rendering using `ratatui` + `crossterm`.
- **`forwarding`**: Command interceptor for standard git CLI functions.
- **`utils`**: Common helpers for formatting and system access.

## 🚧 Roadmap

- [ ] **v0.2.0**: Timeline playback mode to replay commit sequences.
- [ ] **v0.3.0**: Interactive search/filter bar within TUI.
- [ ] **v0.4.0**: JSON/Markdown export of branch comparison data.
- [ ] **v1.0.0**: Plugin system for custom workflow automation.

## 📄 License
MIT / Apache-2.0
