#![forbid(unsafe_code)]

pub type CardId = i64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deck {
    pub name: String,
    pub new_count: u32,
    pub learn_count: u32,
    pub review_count: u32,
}

// This is like a method for the struct
impl Deck {
    pub fn due_count(&self) -> u32 {
        self.new_count + self.learn_count + self.review_count
    }
}

/// A single card ready to be shown for review. `question_html`/`answer_html`
/// are Anki's fully rendered HTML (cloze deletions already resolved).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub id: CardId,
    pub deck_name: String,
    pub question_html: String,
    pub answer_html: String,
}

/// The grade given to a card after review, matching Anki's own 1-4 scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ease {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

impl Ease {
    pub fn from_key(n: u8) -> Option<Self> {
        match n {
            1 => Some(Ease::Again),
            2 => Some(Ease::Hard),
            3 => Some(Ease::Good),
            4 => Some(Ease::Easy),
            _ => None,
        }
    }

    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn label(self) -> &'static str {
        match self {
            Ease::Again => "Again",
            Ease::Hard => "Hard",
            Ease::Good => "Good",
            Ease::Easy => "Easy",
        }
    }
}

#[cfg(test)]
// Creates a module
mod tests {
    // imports everything from outside, so it doesn't need super::deck
    use super::*;

    // tells you its a test
    #[test]
    fn ease_round_trips_through_keys() {
        for n in 1u8..=4 {
            assert_eq!(Ease::from_key(n).unwrap().as_u8(), n);
        }
        assert!(Ease::from_key(0).is_none());
        assert!(Ease::from_key(5).is_none());
    }

    #[test]
    fn deck_due_count_sums_all_queues() {
        let deck = Deck {
            name: "Spanish".into(),
            new_count: 5,
            learn_count: 2,
            review_count: 10,
        };
        assert_eq!(deck.due_count(), 17);
    }
}
