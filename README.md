<h1 align="left"><img src="assets/logo.png" width="64" alt="elio logo" align="absmiddle" />&nbsp;elio</h1>

A dual-pane fork of [Elio](https://github.com/elio-fm/elio), the terminal file manager.

## Description

This repository is a fork of Elio that adds a second file pane. The second pane opens in the preview column, so two directories can stay visible and usable at the same time. Closing it restores the preview.

Navigation, file operations, previews, themes, and the rest of Elio behave as in the upstream project. Upstream documentation remains the reference for those features: [elio-fm.github.io/docs](https://elio-fm.github.io/docs/).

## Installation

Install this fork from source. Published packages and `cargo install elio` distribute upstream Elio, which does not include the second pane.

Rust 1.98.1 or newer is required. [rustup](https://rustup.rs) installs Rust and the `cargo` build tool. Git is required to download the repository.

```bash
git clone https://github.com/mardentrodemi/elio.git
cd elio
cargo install --path .
```

`git clone` downloads the repository. `cd elio` enters that directory. `cargo install --path .` builds the project in the current directory and installs the `elio` command to `~/.cargo/bin`.

Open a new terminal if the shell does not find `elio`, then start the file manager:

```bash
elio
```

## Usage

The second pane uses the preview column. It opens only while that column is available.

### Default key bindings

| Action | Key |
|---|---|
| Open or close the second file pane | `\` |
| Move focus between the two file panes | `Shift+\` |

`\` opens the second pane and moves focus to it. Press `\` again to close the pane and restore the preview. `Shift+\` switches keyboard focus between the left and right panes. A click on a pane also moves focus to it.

### Changing the key bindings

Key bindings are set in the Elio configuration file:

| Platform | Configuration file |
|---|---|
| Linux and BSD | `~/.config/elio/config.toml` or `$XDG_CONFIG_HOME/elio/config.toml` |
| macOS | `~/.config/elio/config.toml` or `~/Library/Application Support/elio/config.toml` |
| Windows | `%APPDATA%\elio\config.toml` |

Create the file if it does not exist, and add a `[keys]` section. Only the bindings you set here change; every other binding keeps its default.

```toml
[keys]
secondary_browser = "\\"
focus_other_file_pane = "|"
```

`secondary_browser` toggles the second file pane. `focus_other_file_pane` switches focus between the panes. The value is the character the key produces. On a standard US layout, `Shift+\` produces `|`, so that binding is written as `"|"`.

Replace the quoted values to assign different keys. An annotated copy of the full configuration, including these entries, is in [`examples/config.toml`](examples/config.toml).

## License

[MIT](LICENSE-MIT)
