# zellij-gleambar

A one-line tab bar for [Zellij](https://zellij.dev).

```
 1 nvim:zellij  2 claude:api  3 ~                                         TMUX
```

- Tabs on the left; the active tab is highlighted and bold.
- Mode badge on the far right (`TMUX`, `SCROLL`, `SEARCH`, `RENAME`, …). It has a
  fixed slot and is hidden in normal mode, so tabs never shift.
- Names longer than 20 columns end in `…`. Tabs that don't fit hide behind
  `← +N` / `+N →` markers, keeping the active tab in view.
- No session name and no "Zellij" label.
- Flags: `󰊓` fullscreen pane, `󰉈` floating panes visible, `!` bell.
- Click a tab to switch to it; scroll over the bar to cycle tabs.
- Colours come from your Zellij theme, so there is nothing to configure.

Built for Zellij 0.45.

## Install

No Rust needed. Zellij downloads the plugin from the release page.

Add this to `~/.config/zellij/config.kdl`:

```kdl
plugins {
    compact-bar location="https://github.com/snehilshah/zellij-gleambar/releases/download/v0.1.0/gleambar.wasm"
}
default_layout "compact"
```

This replaces the built-in compact bar, so the built-in `compact` layout (and
its swap layouts) now uses gleambar.

### First start

Zellij asks once for permission to read and switch tabs:

```
This plugin asks permission to: ReadApplicationState, ChangeApplicationState. Allow? (y/n)
```

Move focus down to the bar (`Alt j` or `Alt ↓` with default keys) and press `y`.
The bar then stops taking focus. You'll be asked again after changing the
version in the URL.

### Using your own layout

If you use a custom layout instead of `compact`, add the bar to its tab template:

```kdl
layout {
    default_tab_template {
        children
        pane size=1 borderless=true {
            plugin location="https://github.com/snehilshah/zellij-gleambar/releases/download/v0.1.0/gleambar.wasm"
        }
    }
}
```

### Upgrading

Change the version in the URL. Zellij caches plugins by URL, so a
`releases/latest/...` URL would never pick up a new release.

## Theming

gleambar reads these parts of the active theme:

| Theme component     | Used for                                                   |
| ------------------- | ---------------------------------------------------------- |
| `text_unselected`   | `background` = bar; `emphasis_0..3` = mode badges          |
| `ribbon_selected`   | Active tab (`emphasis_0` = tab number)                     |
| `ribbon_unselected` | Other tabs (`emphasis_0` = tab with a bell)                |

Badge colours: `emphasis_0` tmux, `emphasis_1` scroll/search, `emphasis_2`
rename, `emphasis_3` any other mode.

## Tab names

gleambar shows each tab's name as it is. To name tabs after what's running
(`nvim:project`), rename them from your shell. For example, this zsh hook:
[snehilshah/zsh `zellij.zsh`](https://github.com/snehilshah/zsh/blob/main/zellij.zsh).

## Build from source

```sh
rustup target add wasm32-wasip1
make install   # builds and copies to ~/.local/share/zellij/plugins/gleambar.wasm
```

Then point Zellij at the local file:

```kdl
plugins {
    compact-bar location="file:~/.local/share/zellij/plugins/gleambar.wasm"
}
```

To target another Zellij version, set `zellij-tile` in `Cargo.toml` to that version.

## Releasing

Bump `version` in `Cargo.toml`, commit, then push a matching tag:

```sh
git tag v0.2.0 && git push origin v0.2.0
```

The release workflow builds `gleambar.wasm` and attaches it, with a SHA-256
checksum, to a GitHub release.

## License

MIT
