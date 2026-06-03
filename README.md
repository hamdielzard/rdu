# rdu - A fast, tree-style disk usage viewer

A parallel disk usage analyzer with tree output. Faster than `du -sh` on a full
recursive walk while building and displaying a complete sorted size tree.

## Table of Contents

- [Requirements](#requirements)
- [Installation](#installation)
- [Uninstallation](#uninstallation)
- [Usage](#usage)
- [Shell Completions](#shell-completions)
- [Benchmarks](#benchmarks)
- [License](#license)

## Requirements

- Rust 1.70+ (`rustup` recommended)

## Installation

```sh
git clone https://github.com/hamdielzard/rdu
cd rdu
cargo install --path .
```

**Install location** 

`~/.cargo/bin/rdu` (macOS/Linux)

`%USERPROFILE%\.cargo\bin\rdu.exe` (Windows)

Or build manually without installing:

```sh
cargo build --release
# binary at ./target/release/rdu
```

Consider installing the [shell completions](#shell-completions) for your environment.

## Uninstallation

```sh
cargo uninstall rdu
```

See [Shell Completions](#shell-completions) for instructions on removing the shell completions.

## Usage

```
rdu [OPTIONS] [PATH]

Arguments:
  [PATH]  Directory to analyze [default: .]

Options:
  -d, --depth <DEPTH>  Maximum depth to display [default: 1]
  -r, --recursive      Recurse into all subdirectories without depth limit
      --no-color       Disable colored output
  -h, --help           Print help
  -V, --version        Print version
```

**Examples**

```sh
# Analyze current directory (depth 1)
rdu

# Analyze a specific path
rdu /opt/homebrew

# Full recursive tree
rdu --recursive /opt/homebrew

# Limit depth to 3 levels
rdu --depth 3 /opt/homebrew
```

## Shell Completions

Completions are generated at runtime via the `completions` subcommand.

```sh
# bash (bash-completion 2.x)
rdu completions bash > ~/.local/share/bash-completion/completions/rdu

# zsh
rdu completions zsh > ~/.zfunc/_rdu

# fish
rdu completions fish > ~/.config/fish/completions/rdu.fish

# PowerShell (Windows)
rdu completions powershell >> $PROFILE
```

**Removal**

```sh
# bash
rm ~/.local/share/bash-completion/completions/rdu

# zsh
rm ~/.zfunc/_rdu

# fish
rm ~/.config/fish/completions/rdu.fish

# PowerShell
# Remove the rdu completions line from $PROFILE
```

## Benchmarks

Measured with [`hyperfine`](https://github.com/sharkdp/hyperfine) on macOS
(Apple Silicon, M5 Pro). 3 warmup runs, 10 timed runs per command.

### Full recursive traversal (`rdu --recursive` vs `du -sh`)

Both tools perform a complete walk of the directory tree.

| Directory | Files | `rdu` mean | `du` mean | Delta |
|---|---:|---:|---:|---:|
| `/opt/homebrew` | 30,733 | **35.3 ms** +/- 0.7 | 73.3 ms +/- 0.3 | 2.1x |
| `~/.cargo/registry` | 1,620 | **8.2 ms** +/- 0.1 | 9.8 ms +/- 0.3 | 1.2x |
| `/Applications` | 257,295 | **542 ms** +/- 19 | 911 ms +/- 49 | 1.7x |
| `~/.rustup` | 54,529 | **77.5 ms** +/- 3.0 | 87.6 ms +/- 3.0 | 1.1x |

### Default depth traversal (`rdu` vs `du -sh`)

`rdu` default mode (`--depth 1`) walks one level and sums subtrees in parallel.
`du -sh` performs a full recursive walk regardless.

| Directory | Files | `rdu` mean | `du` mean | Delta |
|---|---:|---:|---:|---:|
| `/opt/homebrew` | 30,733 | **24.8 ms** +/- 1.2 | 68.0 ms +/- 0.5 | 2.7x |
| `~/.cargo/registry` | 1,620 | **7.5 ms** +/- 0.3 | 9.4 ms +/- 0.5 | 1.3x |
| `/Applications` | 257,295 | **441 ms** +/- 24 | 864 ms +/- 6 | 2.0x |
| `~/.rustup` | 54,529 | **62.1 ms** +/- 3.0 | 80.9 ms +/- 0.8 | 1.3x |

## License

See [LICENSE](LICENSE) for details.
