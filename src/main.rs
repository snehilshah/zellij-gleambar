//! gleambar: a one-line zellij tab bar.
//!
//! Tabs on the left, mode badge in a fixed slot on the right (empty in normal
//! mode, so tabs never shift). Tabs that don't fit are hidden behind
//! "← +N" / "+N →" markers, centred on the active tab. Colours come from the
//! active zellij theme.

use std::collections::BTreeMap;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use zellij_tile::prelude::*;

/// Longest tab name shown, in columns. Longer names end in "…".
const MAX_NAME: usize = 20;
/// Columns always reserved on the right for the mode badge (" SCROLL ").
const BADGE_SLOT: usize = 8;

#[derive(Default)]
struct State {
    tabs: Vec<TabInfo>,
    mode: InputMode,
    styling: Styling,
    /// (start_col, end_col_exclusive, tab position) for mouse clicks.
    click_targets: Vec<(usize, usize, usize)>,
}

register_plugin!(State);

impl ZellijPlugin for State {
    fn load(&mut self, _config: BTreeMap<String, String>) {
        set_selectable(false);
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
        subscribe(&[EventType::ModeUpdate, EventType::TabUpdate, EventType::Mouse]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::ModeUpdate(info) => {
                let changed = info.mode != self.mode || info.style.colors != self.styling;
                self.mode = info.mode;
                self.styling = info.style.colors;
                changed
            }
            Event::TabUpdate(tabs) => {
                let changed = tabs != self.tabs;
                self.tabs = tabs;
                changed
            }
            Event::Mouse(Mouse::LeftClick(_, col)) => {
                if let Some(&(_, _, pos)) =
                    self.click_targets.iter().find(|(s, e, _)| col >= *s && col < *e)
                {
                    switch_tab_to(pos as u32 + 1);
                }
                false
            }
            Event::Mouse(Mouse::ScrollUp(_)) => {
                go_to_previous_tab();
                false
            }
            Event::Mouse(Mouse::ScrollDown(_)) => {
                go_to_next_tab();
                false
            }
            _ => false,
        }
    }

    fn render(&mut self, _rows: usize, cols: usize) {
        let s = &self.styling;
        let bar = bg(s.text_unselected.background);
        let marker = fg(s.ribbon_unselected.base);
        let tabs_cols = cols.saturating_sub(BADGE_SLOT);
        let mut out = String::new();
        let mut used = 0;
        self.click_targets.clear();

        let names: Vec<String> = self.tabs.iter().map(|t| tab_name(t)).collect();
        let widths: Vec<usize> = self
            .tabs
            .iter()
            .zip(&names)
            .map(|(t, n)| tab_width(t, n))
            .collect();
        let active = self.tabs.iter().position(|t| t.active).unwrap_or(0);

        if !self.tabs.is_empty() {
            let (lo, hi) = visible_range(&widths, active, tabs_cols);

            if lo > 0 {
                let m = format!(" ← +{lo} ");
                self.click_targets.push((used, used + m.width(), lo - 1));
                used += m.width();
                out += &format!("{bar}{marker}{m}");
            }

            for i in lo..=hi {
                let tab = &self.tabs[i];
                let (text, num, body) = if tab.active {
                    let r = &s.ribbon_selected;
                    (fg(r.base), fg(r.emphasis_0), bg(r.background))
                } else {
                    let r = &s.ribbon_unselected;
                    let text = if tab.has_bell_notification { r.emphasis_0 } else { r.base };
                    (fg(text), fg(text), bg(r.background))
                };
                let bold = if tab.active { "\x1b[1m" } else { "" };
                let label = format!(" {} ", tab.position + 1);
                let room = tabs_cols.saturating_sub(used + label.width() + 1);
                let name = truncate(&names[i], room);
                let w = label.width() + name.width() + 1;
                self.click_targets.push((used, used + w, tab.position));
                used += w;
                out += &format!("{body}{bold}{num}{label}{text}{name} \x1b[22m");
            }

            let hidden_right = self.tabs.len() - 1 - hi;
            if hidden_right > 0 {
                let m = format!(" +{hidden_right} → ");
                if used + m.width() <= tabs_cols {
                    self.click_targets.push((used, used + m.width(), hi + 1));
                    used += m.width();
                    out += &format!("{bar}{marker}{m}");
                }
            }
        }

        // Right-aligned mode badge; its slot is always reserved.
        let badge = mode_badge(self.mode, &s.text_unselected).map(|(label, color)| {
            let text = format!(" {label} ");
            (format!("{}{}\x1b[1m{text}\x1b[22m", fg(s.text_unselected.background), bg(color)), text.width())
        });
        let badge_w = badge.as_ref().map_or(0, |b| b.1);
        out += &format!("{bar}{}", " ".repeat(cols.saturating_sub(used + badge_w)));
        if let Some((b, _)) = badge {
            out += &b;
        }
        print!("{out}\x1b[0m");
    }
}

fn mode_badge(mode: InputMode, t: &StyleDeclaration) -> Option<(&'static str, PaletteColor)> {
    Some(match mode {
        InputMode::Normal | InputMode::Locked => return None,
        InputMode::Tmux => ("TMUX", t.emphasis_0),
        InputMode::Scroll => ("SCROLL", t.emphasis_1),
        InputMode::EnterSearch | InputMode::Search => ("SEARCH", t.emphasis_1),
        InputMode::RenameTab | InputMode::RenamePane => ("RENAME", t.emphasis_2),
        _ => ("MODE", t.emphasis_3),
    })
}

/// Tab name plus status flags, capped at MAX_NAME columns.
fn tab_name(tab: &TabInfo) -> String {
    let mut name = if tab.name.is_empty() { "…".to_string() } else { truncate(&tab.name, MAX_NAME) };
    if tab.is_fullscreen_active {
        name += " 󰊓";
    }
    if tab.are_floating_panes_visible {
        name += " 󰉈";
    }
    if tab.has_bell_notification && !tab.active {
        name += " !";
    }
    name
}

/// Width of " N name ".
fn tab_width(tab: &TabInfo, name: &str) -> usize {
    format!(" {} ", tab.position + 1).width() + name.width() + 1
}

/// Widest window of tabs around `active` that fits in `avail` columns,
/// leaving room for the overflow markers.
fn visible_range(widths: &[usize], active: usize, avail: usize) -> (usize, usize) {
    const MARKER: usize = 8; // " ← +NN " plus slack
    let n = widths.len();
    let fits = |lo: usize, hi: usize| {
        let tabs: usize = widths[lo..=hi].iter().sum();
        let markers = MARKER * ((lo > 0) as usize + (hi < n - 1) as usize);
        tabs + markers <= avail
    };
    let (mut lo, mut hi) = (active, active);
    loop {
        let mut grew = false;
        if hi + 1 < n && fits(lo, hi + 1) {
            hi += 1;
            grew = true;
        }
        if lo > 0 && fits(lo - 1, hi) {
            lo -= 1;
            grew = true;
        }
        if !grew {
            return (lo, hi);
        }
    }
}

/// Cut `s` to at most `max` columns, ending with "…" when shortened.
fn truncate(s: &str, max: usize) -> String {
    if s.width() <= max {
        return s.to_string();
    }
    let mut out = String::new();
    let mut w = 0;
    for c in s.chars() {
        let cw = c.width().unwrap_or(0);
        if w + cw + 1 > max {
            break;
        }
        out.push(c);
        w += cw;
    }
    if max > 0 {
        out.push('…');
    }
    out
}

fn fg(c: PaletteColor) -> String {
    match c {
        PaletteColor::Rgb((r, g, b)) => format!("\x1b[38;2;{r};{g};{b}m"),
        PaletteColor::EightBit(n) => format!("\x1b[38;5;{n}m"),
    }
}

fn bg(c: PaletteColor) -> String {
    match c {
        PaletteColor::Rgb((r, g, b)) => format!("\x1b[48;2;{r};{g};{b}m"),
        PaletteColor::EightBit(n) => format!("\x1b[48;5;{n}m"),
    }
}
