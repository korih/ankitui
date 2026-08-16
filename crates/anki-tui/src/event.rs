use anki_model::Ease;
use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{App, Connection, Screen};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match app.screen {
        Screen::DeckList => handle_deck_list_key(app, key),
        Screen::Reviewing => handle_review_key(app, key),
    }
}

fn handle_deck_list_key(app: &mut App, key: KeyEvent) {
    if let Connection::Failed(_) = app.connection {
        match key.code {
            KeyCode::Char('r') => app.refresh_decks(),
            KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
            _ => {}
        }
        return;
    }

    // vim-style "gg" needs to see two 'g' presses in a row.
    if key.code == KeyCode::Char('g') {
        if app.pending_g {
            app.pending_g = false;
            app.select_first();
        } else {
            app.pending_g = true;
        }
        return;
    }
    app.pending_g = false;

    match key.code {
        KeyCode::Char('G') => app.select_last(),
        KeyCode::Char('j') | KeyCode::Down => app.select_next(),
        KeyCode::Char('k') | KeyCode::Up => app.select_prev(),
        KeyCode::Enter | KeyCode::Char('l') => app.start_review(),
        KeyCode::Char('r') => app.refresh_decks(),
        KeyCode::Char('q') => app.should_quit = true,
        _ => {}
    }
}

fn handle_review_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char(' ') | KeyCode::Enter if !app.revealed => app.flip(),
        KeyCode::Char(c @ '1'..='4') if app.revealed => {
            if let Some(ease) = Ease::from_key(c as u8 - b'0') {
                app.grade(ease);
            }
        }
        KeyCode::Char('q') | KeyCode::Esc => app.end_session(None),
        _ => {}
    }
}
