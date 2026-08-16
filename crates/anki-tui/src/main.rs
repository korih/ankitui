mod app;
mod event;
mod html;
mod ui;

use std::io;
use std::time::Duration;

use anki_connect::AnkiConnectClient;
use app::App;
use crossterm::event::{Event, KeyEventKind};

fn main() -> io::Result<()> {
    let client = AnkiConnectClient::from_env();
    let mut app = App::new(client);

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut app);
    ratatui::restore();
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, app))?;
        if crossterm::event::poll(Duration::from_millis(200))?
            && let Event::Key(key) = crossterm::event::read()?
            && key.kind == KeyEventKind::Press
        {
            event::handle_key(app, key);
        }
    }
    Ok(())
}
