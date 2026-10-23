//! Supplied private outside-game cards are physical identities, separate from ceased tokens.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct OutsideGameState {
    cards: [SharedVec<ObjectRef>; 2],
}
impl Game {
    /// Construct a game with explicitly supplied private sideboards. Constructed
    /// and frozen Standard enforce fifteen-card sideboards and combined copy limits.
    /// Freeform accepts up to 65535 outside cards per player for deterministic scenarios.
    pub fn new_with_sideboards(config: Config, sideboards: [Vec<Card>; 2]) -> Result<Self, Error> {
        for (index, sideboard) in sideboards.iter().enumerate() {
            let limit = if config.deck_rules == DeckRules::Freeform {
                65535
            } else {
                15
            };
            if sideboard.len() > limit {
                return Err(Error::InvalidDeck(format!(
                    "sideboard exceeds {limit} cards"
                )));
            }
            let mut counts = vec![0usize; Card::catalogue_len()];
            for &card in config.decks[index].iter().chain(sideboard) {
                if config.deck_rules != DeckRules::Freeform {
                    counts[card.index()] += 1;
                    if card
                        .info()
                        .deck_limit
                        .is_some_and(|limit| counts[card.index()] > usize::from(limit))
                    {
                        return Err(Error::InvalidDeck(format!(
                            "combined deck and sideboard copy limit exceeded: {}",
                            card.name()
                        )));
                    }
                }
            }
            for &card in sideboard {
                if card.is_token()
                    || card.is_emblem()
                    || card.inset_parent().is_some()
                    || card.front_face() != card
                {
                    return Err(Error::InvalidDeck(
                        "sideboard requires physical front-face cards".into(),
                    ));
                }
                if !card.is_supported() {
                    return Err(Error::InvalidDeck(format!(
                        "rules not implemented: {}",
                        card.name()
                    )));
                }
                if config.deck_rules == DeckRules::Standard && !card.is_standard_legal() {
                    return Err(Error::InvalidDeck(format!(
                        "not Standard legal in this catalogue: {}",
                        card.name()
                    )));
                }
            }
        }
        let mut game = Self::new(config)?;
        let mut cards: [SharedVec<ObjectRef>; 2] = std::array::from_fn(|_| SharedVec::default());
        for (player, sideboard) in [Player::P0, Player::P1].into_iter().zip(sideboards) {
            for card in sideboard {
                let id =
                    CardId::try_from(game.state.objects.len()).capacity("object slots exhausted");
                game.state.objects.push(Object {
                    card,
                    copy_kind: CopyKind::None,
                    owner: player,
                    controller: player,
                    zone: Zone::Outside,
                    generation: 0,
                    tapped: false,
                    summoning_sick: false,
                    flashback: false,
                    damage: 0,
                    damaged_this_turn: false,
                    dealt_damage_this_turn: false,
                    deathtouch_since_sba: false,
                    growth: Stats::default(),
                    gained_keywords: Keywords::NONE,
                    counters: InitialCounters::default(),
                    used_abilities: 0,
                    attached_to: None,
                });
                cards[player.index()].push(game.object_ref(id));
            }
        }
        if cards.iter().any(|rows| !rows.is_empty()) {
            game.state.outside_game = Some(Box::new(OutsideGameState { cards }));
        }
        Ok(game)
    }
    /// Exact owned private outside-card identities. Ceased tokens are never members.
    pub fn outside_game_cards(&self, player: Player) -> impl Iterator<Item = ObjectRef> + '_ {
        self.state
            .outside_game
            .as_ref()
            .into_iter()
            .flat_map(move |pool| pool.cards[player.index()].iter().copied())
            .filter(move |&object| self.outside_game_card_matches(player, object))
    }
    pub(super) fn outside_game_card_matches(&self, player: Player, object: ObjectRef) -> bool {
        self.state
            .objects
            .get(object.card as usize)
            .is_some_and(|card| {
                card.owner == player
                    && card.zone == Zone::Outside
                    && card.generation == object.generation
                    && !card.is_token()
                    && !card.is_spell_copy()
            })
            && self
                .state
                .outside_game
                .as_ref()
                .is_some_and(|pool| pool.cards[player.index()].contains(&object))
    }
    pub(super) fn forget_outside_game_card(&mut self, object: ObjectRef) {
        if let Some(pool) = self.state.outside_game.as_mut() {
            for cards in &mut pool.cards {
                cards.retain(|&candidate| candidate != object);
            }
        }
    }
    pub(super) fn outside_game_bytes(&self) -> usize {
        self.state.outside_game.as_ref().map_or(0, |pool| {
            std::mem::size_of::<OutsideGameState>()
                + pool
                    .cards
                    .iter()
                    .map(|cards| cards.len() * std::mem::size_of::<ObjectRef>())
                    .sum::<usize>()
        })
    }
}
