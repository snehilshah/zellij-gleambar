# zellij-gleambar

One-line Zellij tab bar: a mode badge (hidden in normal mode) and the tabs, with
`← +N` / `+N →` markers when they don't fit. Colours come from the active Zellij
theme (active tab = `ribbon_selected`, others = `ribbon_unselected`, bar and
badges = `text_unselected`). Click a tab to switch, scroll to cycle.

```sh
make install   # builds and copies to ~/.local/share/zellij/plugins/gleambar.wasm
```

Rebuild after upgrading Zellij: bump `zellij-tile` in `Cargo.toml` to the new version, then `make install`.
