#![forbid(unsafe_code)]

use std::collections::HashMap;

use anki_model::{Card, CardId, Deck, Ease};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:8765";
const API_VERSION: u8 = 6;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not reach AnkiConnect at {endpoint} (is Anki open with the AnkiConnect addon installed?): {source}")]
    Transport {
        endpoint: String,
        #[source]
        source: ureq::Error,
    },
    #[error("AnkiConnect reported an error: {0}")]
    AnkiConnect(String),
    #[error("AnkiConnect returned no result for a successful call")]
    MissingResult,
}

#[derive(Serialize)]
struct Request<'a> {
    action: &'a str,
    version: u8,
    params: Value,
}

#[derive(Deserialize)]
struct Envelope<T> {
    result: Option<T>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct DeckStatsDto {
    name: String,
    new_count: u32,
    learn_count: u32,
    review_count: u32,
}

impl From<DeckStatsDto> for Deck {
    fn from(dto: DeckStatsDto) -> Self {
        Deck {
            name: dto.name,
            new_count: dto.new_count,
            learn_count: dto.learn_count,
            review_count: dto.review_count,
        }
    }
}

#[derive(Deserialize)]
struct CardInfoDto {
    #[serde(rename = "cardId")]
    card_id: CardId,
    question: String,
    answer: String,
    #[serde(rename = "deckName")]
    deck_name: String,
}

impl From<CardInfoDto> for Card {
    fn from(dto: CardInfoDto) -> Self {
        Card {
            id: dto.card_id,
            deck_name: dto.deck_name,
            question_html: dto.question,
            answer_html: dto.answer,
        }
    }
}

/// A blocking client for the AnkiConnect addon's local HTTP API.
pub struct AnkiConnectClient {
    endpoint: String,
}

impl AnkiConnectClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }

    /// Builds a client from `ANKI_CONNECT_URL`, falling back to the default
    /// local AnkiConnect address.
    pub fn from_env() -> Self {
        let endpoint =
            std::env::var("ANKI_CONNECT_URL").unwrap_or_else(|_| DEFAULT_ENDPOINT.to_string());
        Self::new(endpoint)
    }

    fn invoke<T: for<'de> Deserialize<'de>>(&self, action: &str, params: Value) -> Result<T, Error> {
        let request = Request {
            action,
            version: API_VERSION,
            params,
        };
        let mut response =
            ureq::post(&self.endpoint)
                .send_json(&request)
                .map_err(|source| Error::Transport {
                    endpoint: self.endpoint.clone(),
                    source,
                })?;
        let envelope: Envelope<T> =
            response
                .body_mut()
                .read_json()
                .map_err(|source| Error::Transport {
                    endpoint: self.endpoint.clone(),
                    source,
                })?;
        if let Some(message) = envelope.error {
            return Err(Error::AnkiConnect(message));
        }
        envelope.result.ok_or(Error::MissingResult)
    }

    /// Confirms AnkiConnect is reachable, returning its reported API version.
    pub fn version(&self) -> Result<u8, Error> {
        self.invoke("version", json!({}))
    }

    pub fn deck_names(&self) -> Result<Vec<String>, Error> {
        self.invoke("deckNames", json!({}))
    }

    /// Fetches new/learn/review counts (respecting configured daily limits)
    /// for the given deck names.
    pub fn deck_stats(&self, names: &[String]) -> Result<Vec<Deck>, Error> {
        let raw: HashMap<String, DeckStatsDto> =
            self.invoke("getDeckStats", json!({ "decks": names }))?;
        Ok(raw.into_values().map(Deck::from).collect())
    }

    pub fn find_cards(&self, query: &str) -> Result<Vec<CardId>, Error> {
        self.invoke("findCards", json!({ "query": query }))
    }

    pub fn cards_info(&self, ids: &[CardId]) -> Result<Vec<Card>, Error> {
        let raw: Vec<CardInfoDto> = self.invoke("cardsInfo", json!({ "cards": ids }))?;
        Ok(raw.into_iter().map(Card::from).collect())
    }

    /// Submits grades for reviewed cards directly, without needing Anki's
    /// GUI to be on the review screen.
    pub fn answer_cards(&self, answers: &[(CardId, Ease)]) -> Result<(), Error> {
        let payload: Vec<Value> = answers
            .iter()
            .map(|(id, ease)| json!({ "cardId": id, "ease": ease.as_u8() }))
            .collect();
        let _: Vec<bool> = self.invoke("answerCards", json!({ "answers": payload }))?;
        Ok(())
    }
}
