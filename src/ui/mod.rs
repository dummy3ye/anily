pub mod screens;

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

    if app.exit_prompt {
        use ratatui::{
            layout::{Alignment, Rect},
            style::{Color, Style},
            widgets::{Block, Clear, Paragraph},
        };

        let area = f.area();

        let popup_width: u16 = 34;
        let popup_height: u16 = 5;
        let x = area.width.saturating_sub(popup_width) / 2;
        let y = area.height.saturating_sub(popup_height) / 2;
        let popup = Rect::new(
            x,
            y,
            popup_width.min(area.width),
            popup_height.min(area.height),
        );

        f.render_widget(Clear, popup);

        let block = Block::default()
            .title(" Exit? ")
            .borders(ratatui::widgets::Borders::ALL)
            .style(Style::default().bg(Color::Black).fg(Color::White));
        let paragraph = Paragraph::new("Do you want to leave? (y/n)")
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, popup);
    }
}
