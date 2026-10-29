//! Owned, private outside-game selection; the outside pool is not a game zone.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Selection {
    Pool(u16),
    Basic(Card),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Choice {
    id: u64,
    owner: Player,
    first: Option<Selection>,
    pair: Option<[Selection; 2]>,
}
/// Frozen pre-event card facts. This has no `ObjectRef` or in-game origin:
/// the physical outside entry acquires an object identity only on admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OutsideEntryDestination {
    Hand,
    #[allow(
        dead_code,
        reason = "Synthetic replacement foundation is not publicly enabled"
    )]
    Exile,
    #[allow(
        dead_code,
        reason = "Synthetic replacement foundation is not publicly enabled"
    )]
    Prevent,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct OutsideEntryReplacement {
    pub id: u64,
    pub destination: OutsideEntryDestination,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct OutsideHandEvent {
    owner: Player,
    choice_id: u64,
    selected: Selection,
    pair: [Selection; 2],
    revealed: [Card; 2],
    card: Card,
    pool_version: u64,
    destination: OutsideEntryDestination,
    choices: Vec<OutsideEntryReplacement>,
    applied: Vec<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct OutsideChoiceView {
    pub owner: Player,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pool: Option<Vec<Card>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<Selection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revealed: Option<[Card; 2]>,
}
// Count names rather than testing every pair. Large casual collections may
// contain many duplicates, and action enumeration must not become quadratic.
struct NameIndex {
    total: usize,
    single: std::collections::HashMap<&'static str, usize>,
    pairs: std::collections::HashMap<(&'static str, &'static str), usize>,
}
impl NameIndex {
    fn keys(card: Card) -> Vec<&'static str> {
        let mut names: Vec<_> = outside_names::names(card)
            .expect("validated outside card")
            .iter()
            .map(|name| crate::card_names::comparison_key(name))
            .collect();
        names.sort_unstable();
        names.dedup();
        names
    }
    fn new(state: &OutsideGameState, owner: Player, candidates: &[Selection]) -> Self {
        let mut result = Self {
            total: candidates.len(),
            single: Default::default(),
            pairs: Default::default(),
        };
        for &candidate in candidates {
            let names = Self::keys(state.card(owner, candidate));
            for &name in &names {
                *result.single.entry(name).or_default() += 1;
            }
            if names.len() == 2 {
                *result.pairs.entry((names[0], names[1])).or_default() += 1;
            }
        }
        result
    }
    fn has_distinct(&self, card: Card) -> bool {
        let names = Self::keys(card);
        let mut overlapping = names
            .iter()
            .map(|n| self.single.get(n).copied().unwrap_or(0))
            .sum::<usize>();
        if names.len() == 2 {
            overlapping -= self.pairs.get(&(names[0], names[1])).copied().unwrap_or(0);
        }
        overlapping < self.total
    }
}
impl OutsideGameState {
    fn card(&self, owner: Player, selection: Selection) -> Card {
        match selection {
            Selection::Pool(index) => self.pools[owner.index()][usize::from(index)],
            Selection::Basic(card) => card,
        }
    }
    fn checked_card(&self, owner: Player, selection: Selection) -> Option<Card> {
        match selection {
            Selection::Pool(index) => self.pools[owner.index()].get(usize::from(index)).copied(),
            Selection::Basic(card) if self.mode == OutsideGameMode::LimitedSideboard => Some(card),
            Selection::Basic(_) => None,
        }
    }
    fn candidates(&self, owner: Player) -> Vec<Selection> {
        let mut result: Vec<_> = (0..self.pools[owner.index()].len())
            .map(|i| Selection::Pool(u16::try_from(i).expect("pool bound")))
            .collect();
        if self.mode == OutsideGameMode::LimitedSideboard {
            result.extend(
                [
                    Card::Plains,
                    Card::Island,
                    Card::Swamp,
                    Card::Mountain,
                    Card::Forest,
                ]
                .into_iter()
                .map(Selection::Basic),
            );
        }
        result
    }
    fn distinct(&self, owner: Player, a: Selection, b: Selection) -> bool {
        a != b && outside_names::different(self.card(owner, a), self.card(owner, b)) == Some(true)
    }
}
impl Game {
    pub(super) fn start_outside_choice(&mut self, owner: Player) -> bool {
        // Always offer a private decline, even with no valid pair: skipping
        // this decision would disclose information about the hidden pool.
        let state = self.state.outside_game.get_or_insert_with(|| {
            Box::new(OutsideGameState {
                mode: OutsideGameMode::CasualCollection,
                pools: [Vec::new(), Vec::new()],
                choice: None,
                pending_entry: None,
                pool_versions: [0; 2],
                next_choice_id: 0,
                known: [Vec::new(), Vec::new()],
            })
        });
        let next_choice_id = state
            .next_choice_id
            .checked_add(1)
            .capacity("outside choice identity exhausted");
        let id = state.next_choice_id;
        state.next_choice_id = next_choice_id;
        // Replacing the choice invalidates its old uncommitted plan.
        state.pending_entry = None;
        state.choice = Some(Choice {
            id,
            owner,
            first: None,
            pair: None,
        });
        self.state.pending = Pending::OutsideGameChoice { player: owner };
        true
    }
    pub(super) fn visit_outside_choices(&self, emit: &mut impl FnMut(Action) -> bool) -> bool {
        let state = self.state.outside_game.as_ref().unwrap();
        let choice = state.choice.as_ref().unwrap();
        if choice.pair.is_some() {
            return emit(Action::ChooseOutsideRevealed { index: 0 })
                || emit(Action::ChooseOutsideRevealed { index: 1 });
        }
        if emit(Action::DeclineOutsideCards) {
            return true;
        }
        // CR 723.4 hides outside cards from a player's controller. Wizards'
        // Mindslaver release note says that controller cannot have the player
        // choose cards outside the game; do not expose candidate IDs or names.
        if self.decision_controller(choice.owner) != choice.owner {
            return false;
        }
        if choice.first.is_some() && emit(Action::CancelOutsideCard) {
            return true;
        }
        let candidates = state.candidates(choice.owner);
        let names = NameIndex::new(state, choice.owner, &candidates);
        for &candidate in &candidates {
            let legal = if let Some(first) = choice.first {
                state.distinct(choice.owner, first, candidate)
            } else {
                names.has_distinct(state.card(choice.owner, candidate))
            };
            let action = match candidate {
                Selection::Pool(index) => Action::ChooseOutsideCard { index },
                Selection::Basic(card) => Action::ChooseOutsideBasic { card },
            };
            if legal && emit(action) {
                return true;
            }
        }
        false
    }
    pub(super) fn outside_choice_view(&self, viewer: Player) -> Option<OutsideChoiceView> {
        let state = self.state.outside_game.as_ref()?;
        let choice = state.choice.as_ref()?;
        let private = viewer == choice.owner;
        Some(OutsideChoiceView {
            owner: choice.owner,
            pool: private.then(|| state.pools[choice.owner.index()].clone()),
            first: if private { choice.first } else { None },
            revealed: choice.pair.map(|p| {
                [
                    state.card(choice.owner, p[0]),
                    state.card(choice.owner, p[1]),
                ]
            }),
        })
    }
    pub(super) fn owned_outside_view(&self, viewer: Player) -> Option<[Option<Vec<Card>>; 2]> {
        let state = self.state.outside_game.as_ref()?;
        let visible = std::array::from_fn(|i| {
            let owner = if i == 0 { Player::P0 } else { Player::P1 };
            (viewer == owner && !state.pools[i].is_empty()).then(|| state.pools[i].clone())
        });
        visible.iter().any(Option::is_some).then_some(visible)
    }
    pub(super) fn sideboard_counts_view(&self) -> Option<[usize; 2]> {
        let state = self.state.outside_game.as_ref()?;
        (state.mode != OutsideGameMode::CasualCollection)
            .then(|| std::array::from_fn(|p| state.pools[p].len()))
    }
    pub(super) fn known_outside_view(&self) -> Option<[Vec<Card>; 2]> {
        let state = self.state.outside_game.as_ref()?;
        state.known.iter().any(|v| !v.is_empty()).then(|| {
            std::array::from_fn(|p| {
                state.known[p]
                    .iter()
                    .map(|i| state.pools[p][usize::from(*i)])
                    .collect()
            })
        })
    }
    pub(super) fn choose_outside_card(&mut self, selected: Selection) {
        let state = self.state.outside_game.as_mut().unwrap();
        let choice = state.choice.as_mut().unwrap();
        if let Some(first) = choice.first {
            choice.pair = Some([first, selected]);
            for entry in [first, selected] {
                if let Selection::Pool(index) = entry {
                    let known = &mut state.known[choice.owner.index()];
                    if !known.contains(&index) {
                        known.push(index);
                    }
                }
            }
            self.state.pending = Pending::OutsideGameChoice {
                player: choice.owner.other(),
            };
        } else {
            choice.first = Some(selected);
        }
    }
    pub(super) fn cancel_outside_card(&mut self) {
        self.state
            .outside_game
            .as_mut()
            .unwrap()
            .choice
            .as_mut()
            .unwrap()
            .first = None;
    }
    pub(super) fn decline_outside_cards(&mut self) {
        let state = self.state.outside_game.as_mut().unwrap();
        state.choice = None;
        state.pending_entry = None;
        self.run_program();
    }
    /// Freeze the physical slot and pool version without consuming the card.
    /// The choice stays owned until commitment; this is not a zone-change event.
    pub(super) fn plan_outside_hand(&mut self, index: u8) -> bool {
        let Some(state) = self.state.outside_game.as_mut() else {
            return false;
        };
        let Some(choice) = state.choice.as_ref() else {
            return false;
        };
        let Some(pair) = choice.pair else {
            return false;
        };
        let Some(selected) = pair.get(usize::from(index)).copied() else {
            return false;
        };
        if state.pending_entry.is_some() {
            return false;
        }
        let Some(first) = state.checked_card(choice.owner, pair[0]) else {
            return false;
        };
        let Some(second) = state.checked_card(choice.owner, pair[1]) else {
            return false;
        };
        let card = if index == 0 { first } else { second };
        state.pending_entry = Some(OutsideHandEvent {
            owner: choice.owner,
            choice_id: choice.id,
            selected,
            pair,
            revealed: [first, second],
            card,
            pool_version: state.pool_versions[choice.owner.index()],
            destination: OutsideEntryDestination::Hand,
            choices: Vec::new(),
            applied: Vec::new(),
        });
        true
    }

    /// Memory owned by an uncommitted replacement continuation.
    pub(super) fn outside_entry_bytes(&self) -> usize {
        self.state
            .outside_game
            .as_ref()
            .and_then(|s| s.pending_entry.as_ref())
            .map_or(0, |event| {
                event.choices.capacity() * std::mem::size_of::<OutsideEntryReplacement>()
                    + event.applied.capacity() * std::mem::size_of::<u64>()
            })
    }

    /// Synthetic providers exercise the owned plan without exposing an
    /// incomplete replacement decision in the production public API.
    #[cfg(test)]
    pub(super) fn offer_outside_entry_replacements(
        &mut self,
        choices: Vec<OutsideEntryReplacement>,
    ) -> bool {
        let Some(event) = self
            .state
            .outside_game
            .as_mut()
            .and_then(|state| state.pending_entry.as_mut())
        else {
            return false;
        };
        if !event.choices.is_empty()
            || event.destination == OutsideEntryDestination::Prevent
            || choices.is_empty()
            || choices
                .iter()
                .any(|effect| event.applied.contains(&effect.id))
            || choices
                .iter()
                .enumerate()
                .any(|(i, effect)| choices[..i].iter().any(|prior| prior.id == effect.id))
        {
            return false;
        }
        event.choices = choices;
        true
    }
    #[cfg(test)]
    pub(super) fn choose_outside_entry_replacement(&mut self, id: u64) -> bool {
        let Some(event) = self
            .state
            .outside_game
            .as_mut()
            .and_then(|state| state.pending_entry.as_mut())
        else {
            return false;
        };
        let Some(effect) = event.choices.iter().find(|effect| effect.id == id).copied() else {
            return false;
        };
        event.destination = effect.destination;
        event.applied.push(effect.id);
        event.choices.clear();
        true
    }

    pub(super) fn choose_outside_revealed(&mut self, index: u8) {
        assert!(self.plan_outside_hand(index), "validated outside selection");
        assert!(
            self.commit_outside_hand(),
            "outside selection must remain valid"
        );
        self.run_program();
    }
    /// Commit only if the selected slot and owned choice have not changed.
    /// False leaves the plan, pool, public knowledge and objects unchanged.
    pub(super) fn commit_outside_hand(&mut self) -> bool {
        let Some(state) = self.state.outside_game.as_mut() else {
            return false;
        };
        let Some(event) = state.pending_entry.clone() else {
            return false;
        };
        let Some(choice) = state.choice.as_ref() else {
            return false;
        };
        if !event.choices.is_empty()
            || choice.owner != event.owner
            || choice.id != event.choice_id
            || choice.pair != Some(event.pair)
            || state.pool_versions[event.owner.index()] != event.pool_version
            || state.checked_card(event.owner, event.pair[0]) != Some(event.revealed[0])
            || state.checked_card(event.owner, event.pair[1]) != Some(event.revealed[1])
        {
            return false;
        }
        if event.destination == OutsideEntryDestination::Prevent {
            state.choice = None;
            state.pending_entry = None;
            return true;
        }
        let next_version = match event.selected {
            Selection::Pool(_) => Some(
                state.pool_versions[event.owner.index()]
                    .checked_add(1)
                    .capacity("outside pool version exhausted"),
            ),
            Selection::Basic(_) => None,
        };
        let id = CardId::try_from(self.state.objects.len()).capacity("object slots exhausted");
        state.choice = None;
        state.pending_entry = None;
        let card = match event.selected {
            Selection::Pool(index) => {
                let known = &mut state.known[event.owner.index()];
                known.retain(|&entry| entry != index);
                for entry in known {
                    if *entry > index {
                        *entry -= 1;
                    }
                }
                state.pool_versions[event.owner.index()] = next_version.unwrap();
                state.pools[event.owner.index()].remove(usize::from(index))
            }
            Selection::Basic(card) => card,
        };
        self.state.objects.push(Object {
            card,
            copy_kind: CopyKind::None,
            owner: event.owner,
            controller: event.owner,
            zone: match event.destination {
                OutsideEntryDestination::Hand => Zone::Hand,
                OutsideEntryDestination::Exile => Zone::Exile,
                OutsideEntryDestination::Prevent => {
                    unreachable!("prevented entry did not allocate")
                }
            },
            generation: 0,
            tapped: false,
            summoning_sick: false,
            flashback: false,
            cast_for_warp: false,
            damage: 0,
            damaged_this_turn: false,
            dealt_damage_this_turn: false,
            dealt_combat_damage_this_incarnation: false,
            deathtouch_since_sba: false,
            growth: Stats::default(),
            gained_keywords: Keywords::NONE,
            counters: InitialCounters::default(),
            used_abilities: 0,
            loyalty_activated_this_turn: false,
            class_level: 1,
            attached_to: None,
        });
        if event.destination == OutsideEntryDestination::Hand {
            self.state.known_hand.push(self.object_ref(id));
        }
        true
    }
}
