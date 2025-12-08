use crate::data_model::card::CardRef;

///
/// Generic name for a deck, group, box, etc.
///
struct CardGroup {
    group_type: CardGroupVariety,
    name: String,
    cards: Vec<CardRef>,
    exclusive: bool,
}

// TODO: have Wil add more varieties based on what's actually used
enum CardGroupVariety {
    Deck,
}
