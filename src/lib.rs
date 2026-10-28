pub mod app;
pub mod config;
pub mod error;
pub mod providers;
pub mod rpc;
pub mod ui;

use anyhow::Result;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::ExecutableCommand;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

pub async fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = app::App::new()?;
    loop {
        app.poll_background().await;
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if crossterm::event::poll(std::time::Duration::from_millis(200))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                let result = app.handle_key(key).await;

                match result {
                    Ok(true) => break,
                    Ok(false) => {}
                    Err(error) => {
                        app.state.error = Some(error.to_string());
                        app.state.loading = false;
                        app.state.searching = false;
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}
