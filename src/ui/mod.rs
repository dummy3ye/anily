pub mod screens;

// src/ui/mod.rs
// ui module, it dispatches drawing based on the current screen.

use crate::app::state::Screen;
use crate::app::App;
use crate::ui::screens::home;
use ratatui::Frame;

pub fn draw(f: &mut Frame<'_>, app: &mut App) {
    match app.state.screen {
        Screen::Home => home::draw(f, app),
        Screen::Details => home::draw_details(f, app),
        Screen::Episodes => home::draw_episodes(f, &app.state),
    }
}
