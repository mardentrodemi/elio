<h1 align="left"><img src="assets/logo.png" width="64" alt="elio logo" align="absmiddle" />&nbsp;elio</h1>

A dual-pane fork of [Elio](https://github.com/elio-fm/elio), the terminal file manager.

## Description

This repository is a fork of Elio 1.14.0 that adds a second file pane and folder pins in Places. The second pane opens in the preview column, so two directories can stay visible and usable at the same time. Closing it restores the preview.

Navigation, file operations, previews, themes, and the rest of Elio behave as in the upstream project. Upstream documentation remains the reference for those features: [elio-fm.github.io/docs](https://elio-fm.github.io/docs/).

### Fork features

| Action | Default key |
|---|---|
| Open or close the second file pane | `\` |
| Move focus between the two file panes | `Shift+\` |
| Swap the two file panes | `Ctrl+\` |
| Pin or unpin the current folder in Places | `Shift+B` |
| Cycle sort and remember it for this folder | `s` |

`\` opens the second pane and moves focus to it. Press `\` again to close the pane and restore the preview.

`Shift+\` switches keyboard focus between the left and right panes. A click on a pane also moves focus to it.

`Ctrl+\` swaps the panes: the left directory moves to the right, the right directory moves to the left, and focus stays on the same directory.

`Shift+B` pins the active pane's folder at the end of Places, before Devices. Press `Shift+B` again in that folder to remove the pin. Pins are saved in `place_tabs.toml` next to the Elio config and stay there until you unpin them, including after elio quits. A folder that is already in Places is not added a second time.

`s` still changes the sort, and the order you choose is now remembered separately for each folder. When you open it again, the files are as you left them.

Selecting several files with the mouse has been added. Select the first file with `Space` and click the last one: every file from the first through the last is selected, and you can conveniently drag them from one pane to the other.

### Changing the keys

Add a `[keys]` section to the Elio config. Only the keys you set here change; every other key keeps its default.

| Platform | Configuration file |
|---|---|
| Linux and BSD | `~/.config/elio/config.toml` or `$XDG_CONFIG_HOME/elio/config.toml` |
| macOS | `~/.config/elio/config.toml` or `~/Library/Application Support/elio/config.toml` |
| Windows | `%APPDATA%\elio\config.toml` |

```toml
[keys]
secondary_browser = "\\"
focus_other_file_pane = "|"
swap_file_panes = "ctrl+\\"
toggle_place_tab = "B"
```

| Config key | What it does |
|---|---|
| `secondary_browser` | Open or close the second file pane |
| `focus_other_file_pane` | Switch focus between the two file panes |
| `swap_file_panes` | Swap the two file panes |
| `toggle_place_tab` | Pin or unpin the current folder in Places |

`Shift+\` produces `|` on a standard US layout, so that binding is written as `"|"`. `Shift+B` produces `B`. In a double-quoted TOML string a backslash is written twice, so `Ctrl+\` is `"ctrl+\\"`.

Replace the quoted values to assign different keys. An annotated copy of the full configuration, including these entries, is in [`examples/config.toml`](examples/config.toml).

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

The second pane uses the preview column. It opens only while that column is available. The keys and the config entries are listed in [Fork features](#fork-features) above.

## License

[MIT](LICENSE-MIT)
