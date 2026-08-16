use std::collections::VecDeque;

use anki_connect::AnkiConnectClient;
use anki_model::{Card, CardId, Deck, Ease};

pub enum Screen {
    DeckList,
    Reviewing,
}

pub enum Connection {
    Connecting,
    Connected,
    Failed(String),
}

pub struct App {
    pub client: AnkiConnectClient,
    pub connection: Connection,
    pub screen: Screen,
    pub decks: Vec<Deck>,
    pub selected: usize,
    pub pending_g: bool,
    pub queue: VecDeque<CardId>,
    pub current: Option<Card>,
    pub revealed: bool,
    pub reviewed_count: usize,
    pub session_total: usize,
    pub status: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new(client: AnkiConnectClient) -> Self {
        let mut app = Self {
            client,
            connection: Connection::Connecting,
            screen: Screen::DeckList,
            decks: Vec::new(),
            selected: 0,
            pending_g: false,
            queue: VecDeque::new(),
            current: None,
            revealed: false,
            reviewed_count: 0,
            session_total: 0,
            status: None,
            should_quit: false,
        };
        app.refresh_decks();
        app
    }

    /// Checks connectivity and (re)loads the deck list with due counts.
    pub fn refresh_decks(&mut self) {
        if let Err(err) = self.client.version() {
            self.connection = Connection::Failed(err.to_string());
            return;
        }
        let names = match self.client.deck_names() {
            Ok(names) => names,
            Err(err) => {
                self.connection = Connection::Failed(err.to_string());
                return;
            }
        };
        match self.client.deck_stats(&names) {
            Ok(mut decks) => {
                decks.sort_by(|a, b| a.name.cmp(&b.name));
                self.decks = decks;
                self.selected = self.selected.min(self.decks.len().saturating_sub(1));
                self.connection = Connection::Connected;
            }
            Err(err) => self.connection = Connection::Failed(err.to_string()),
        }
    }

    pub fn select_next(&mut self) {
        if !self.decks.is_empty() {
            self.selected = (self.selected + 1).min(self.decks.len() - 1);
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn select_first(&mut self) {
        self.selected = 0;
    }

    pub fn select_last(&mut self) {
        self.selected = self.decks.len().saturating_sub(1);
    }

    /// Builds a review queue for the selected deck: due cards first, then
    /// new cards, each capped at the deck's own due counts. This doesn't
    /// reproduce Anki's exact new/review interleave order, just the right
    /// set of cards within the day's configured limits.
    pub fn start_review(&mut self) {
        let Some(deck) = self.decks.get(self.selected) else {
            return;
        };
        let name = deck.name.clone();
        let due_limit = (deck.learn_count + deck.review_count) as usize;
        let new_limit = deck.new_count as usize;

        let due_query = format!("deck:\"{name}\" is:due");
        let new_query = format!("deck:\"{name}\" is:new");

        let due_ids = match self.client.find_cards(&due_query) {
            Ok(mut ids) => {
                ids.sort_unstable();
                ids.truncate(due_limit);
                ids
            }
            Err(err) => {
                self.status = Some(format!("failed to load due cards: {err}"));
                return;
            }
        };
        let new_ids = match self.client.find_cards(&new_query) {
            Ok(mut ids) => {
                ids.sort_unstable();
                ids.truncate(new_limit);
                ids
            }
            Err(err) => {
                self.status = Some(format!("failed to load new cards: {err}"));
                return;
            }
        };

        let mut queue = VecDeque::with_capacity(due_ids.len() + new_ids.len());
        queue.extend(due_ids);
        queue.extend(new_ids);

        if queue.is_empty() {
            self.status = Some(format!("no due or new cards in \"{name}\""));
            return;
        }

        self.session_total = queue.len();
        self.reviewed_count = 0;
        self.queue = queue;
        self.screen = Screen::Reviewing;
        self.status = None;
        self.advance();
    }

    /// Loads the next card in the queue, or ends the session if empty.
    fn advance(&mut self) {
        self.revealed = false;
        let Some(id) = self.queue.pop_front() else {
            self.current = None;
            self.end_session(Some(format!(
                "session complete: reviewed {} card(s)",
                self.reviewed_count
            )));
            return;
        };
        match self.client.cards_info(&[id]) {
            Ok(mut cards) if !cards.is_empty() => self.current = Some(cards.remove(0)),
            Ok(_) => self.advance(),
            Err(err) => {
                self.current = None;
                self.end_session(Some(format!("failed to load card: {err}")));
            }
        }
    }

    pub fn flip(&mut self) {
        if self.current.is_some() {
            self.revealed = true;
        }
    }

    pub fn grade(&mut self, ease: Ease) {
        let (Some(card), true) = (&self.current, self.revealed) else {
            return;
        };
        if let Err(err) = self.client.answer_cards(&[(card.id, ease)]) {
            self.status = Some(format!("failed to submit grade: {err}"));
            return;
        }
        self.reviewed_count += 1;
        self.advance();
    }

    pub fn end_session(&mut self, status: Option<String>) {
        self.queue.clear();
        self.current = None;
        self.revealed = false;
        self.screen = Screen::DeckList;
        self.status = status;
        self.refresh_decks();
    }
}
