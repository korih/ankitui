use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

use crate::app::{App, Connection, Screen};
use crate::html::html_to_text;

pub fn draw(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::DeckList => draw_deck_list(frame, app),
        Screen::Reviewing => draw_review(frame, app),
    }
}

fn draw_deck_list(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    frame.render_widget(title_line("AnkiTUI — decks"), chunks[0]);

    match &app.connection {
        Connection::Connecting => {
            frame.render_widget(Paragraph::new("Connecting to AnkiConnect..."), chunks[1]);
        }
        Connection::Failed(err) => {
            let text = format!("Could not reach AnkiConnect:\n{err}\n\npress r to retry, q to quit");
            frame.render_widget(
                Paragraph::new(text).style(Style::default().fg(Color::Red)),
                chunks[1],
            );
        }
        Connection::Connected => draw_deck_items(frame, app, chunks[1]),
    }

    let help = match app.connection {
        Connection::Failed(_) => "r: retry  q: quit",
        _ => "j/k: move  gg/G: top/bottom  enter/l: review  r: refresh  q: quit",
    };
    let mut footer = help.to_string();
    if let Some(status) = &app.status {
        footer = format!("{status}   ({help})");
    }
    frame.render_widget(Paragraph::new(footer).style(Style::default().fg(Color::DarkGray)), chunks[2]);
}

fn draw_deck_items(frame: &mut Frame, app: &App, area: Rect) {
    if app.decks.is_empty() {
        frame.render_widget(Paragraph::new("No decks found."), area);
        return;
    }

    let items: Vec<ListItem> = app
        .decks
        .iter()
        .map(|deck| {
            ListItem::new(format!(
                "{:<30} new:{:<4} learn:{:<4} review:{:<4}",
                deck.name, deck.new_count, deck.learn_count, deck.review_count
            ))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    let mut state = ListState::default().with_selected(Some(app.selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_review(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let Some(card) = &app.current else {
        frame.render_widget(Paragraph::new("Loading card..."), chunks[1]);
        return;
    };

    let progress = format!(
        "{} — card {} of {}",
        card.deck_name,
        app.reviewed_count + 1,
        app.session_total
    );
    frame.render_widget(title_line(&progress), chunks[0]);

    let width = chunks[1].width.saturating_sub(2);
    let mut lines = vec![Line::from(Span::styled(
        "Q:",
        Style::default().add_modifier(Modifier::BOLD),
    ))];
    lines.extend(
        html_to_text(&card.question_html, width)
            .lines()
            .map(|l| Line::from(l.to_string())),
    );

    if app.revealed {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "A:",
            Style::default().add_modifier(Modifier::BOLD),
        )));
        lines.extend(
            html_to_text(&card.answer_html, width)
                .lines()
                .map(|l| Line::from(l.to_string())),
        );
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, chunks[1]);

    let help = if app.revealed {
        "1: again  2: hard  3: good  4: easy  q: end session"
    } else {
        "space/enter: show answer  q: end session"
    };
    let footer = match &app.status {
        Some(status) => format!("{status}   ({help})"),
        None => help.to_string(),
    };
    frame.render_widget(Paragraph::new(footer).style(Style::default().fg(Color::DarkGray)), chunks[2]);
}

fn title_line(text: &str) -> Paragraph<'_> {
    Paragraph::new(text).style(Style::default().add_modifier(Modifier::BOLD))
}
