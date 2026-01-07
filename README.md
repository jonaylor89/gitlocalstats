# GitLocalStats

A high-performance local contribution graph for Git and Jujutsu (jj), written in Rust. It scans your local repositories and visualizes your commit history over the last 6 months directly in the terminal.

![demo](screenshots/demo.png)

## Installation & Building

Ensure you have [Rust](https://www.rust-lang.org/) installed.

```sh
# Clone and build
git clone https://github.com/jonaylor89/gitlocalstats.git
cd gitlocalstats
cargo build --release

# Run
./target/release/gitlocalstats --folder ~/Repos
```

## Usage

### CLI Arguments

```sh
gitlocalstats --folder <PATH> --email <EMAIL>
```

- `--folder`: The root directory to recursively scan for repositories.
- `--email`: Your email address to filter commits. Defaults to your global git config email.

### Configuration

The app loads defaults from `~/.config/gitlocalstats/config`:

```ini
folder=/Users/name/Repos
email=name@example.com
```

## Features

- **Fast**: Parallel directory scanning using `rayon`.
- **Git & Jujutsu**: Supports both standard Git and the new Jujutsu VCS.
- **Dependency Lite**: Optimized for fast compilation and small binary size.
- **Beautiful**: ANSI-colored contribution graph in your terminal.