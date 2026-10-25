//! Independent owned chronology for rule-layer hand-size setters.
use super::*;
use crate::cards::StaticAbilityFragment as F;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct HandSizeSourceView {
    pub source: ObjectRef,
    pub timestamp: u64,
    /// None denotes an unlimited maximum, not a missing effect.
    pub maximum: Option<u16>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HandSizeTimelineView {
    pub sources: Vec<HandSizeSourceView>,
    pub lasting: [Option<u64>; 2],
    pub maximum: [Option<usize>; 2],
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct HandSizeState {
    clock: u64,
    pub deferred: bool,
    pub sources: Vec<HandSizeSourceView>,
    pub lasting: [Option<u64>; 2],
    finite_participated: bool,
}
impl HandSizeState {
    fn next(&mut self) -> u64 {
        if self.deferred {
            return 0;
        }
        self.clock = self
            .clock
            .checked_add(1)
            .capacity("hand-size timestamps exhausted");
        self.clock
    }
    pub(super) fn register(&mut self, source: ObjectRef, card: Card) {
        if self.sources.iter().any(|v| v.source == source) {
            return;
        }
        if let Some(maximum) = card.hand_size_setting() {
            let timestamp = self.next();
            self.finite_participated |= maximum.is_some();
            self.sources.push(HandSizeSourceView {
                source,
                timestamp,
                maximum,
            });
            // Physical entry order and chosen timestamp order are independent.
            self.sources
                .sort_by_key(|row| (row.source.card, row.source.generation));
        }
    }
    pub(super) fn assign(&mut self, source: ObjectRef) {
        if self.sources.iter().any(|v| v.source == source) {
            let timestamp = self.next();
            for row in &mut self.sources {
                if row.source == source {
                    row.timestamp = timestamp;
                }
            }
        }
    }
    pub(super) fn forget(&mut self, source: ObjectRef) {
        self.sources.retain(|v| v.source != source);
    }
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.sources.len() * std::mem::size_of::<HandSizeSourceView>()
    }
}
impl Card {
    /// Outer None means this definition has no printed hand-size setter.
    pub(super) fn hand_size_setting(self) -> Option<Option<u16>> {
        self.static_ability_groups()
            .iter()
            .flat_map(|g| g.fragments)
            .find_map(|f| match f {
                F::NoMaximumHandSize => Some(None),
                F::MaximumHandSize(n) => Some(Some(*n)),
                _ => None,
            })
    }
}
impl Game {
    pub(super) fn new_copy_hand_timestamp(&mut self, card: Card) -> Option<u64> {
        let maximum = card.hand_size_setting()?;
        let state = self.state.hand_size.get_or_insert_default();
        state.finite_participated |= maximum.is_some();
        Some(state.next())
    }
    pub(super) fn register_hand_size_source(&mut self, id: CardId) {
        let card = self.state.objects[id as usize].card;
        if card.hand_size_setting().is_some() {
            let source = self.object_ref(id);
            self.state
                .hand_size
                .get_or_insert_default()
                .register(source, card);
        }
    }
    pub(super) fn forget_hand_size_source(&mut self, source: ObjectRef) {
        if let Some(s) = &mut self.state.hand_size {
            s.forget(source);
        }
    }
    pub(super) fn retimestamp_hand_size_source(&mut self, source: ObjectRef) {
        // Copy-aware object retimestamps update every retained layer together
        // through assign_static_source_timestamp, including this independent clock.
        if self.temporary_copy_row(source).is_some() {
            return;
        }
        if let Some(s) = &mut self.state.hand_size {
            s.assign(source);
        }
    }
    pub(super) fn grant_no_maximum_hand_size(&mut self, player: Player) {
        self.state.players[player.index()].no_maximum_hand_size = true;
        let s = self.state.hand_size.get_or_insert_default();
        let timestamp = s.next();
        s.lasting[player.index()] = Some(timestamp);
    }
    pub(super) fn maximum_hand_size(&self, player: Player) -> Option<usize> {
        let Some(s) = &self.state.hand_size else {
            return Some(7);
        };
        debug_assert_eq!(
            s.lasting[player.index()].is_some(),
            self.state.players[player.index()].no_maximum_hand_size
        );
        let mut result = s.lasting[player.index()].map_or((0, Some(7)), |t| (t, None));
        for row in &s.sources {
            if self.on_battlefield(row.source.card)
                && self.object_ref(row.source.card) == row.source
                && self.state.objects[row.source.card as usize].controller == player
                && row.timestamp >= result.0
                && self.continuous_fragment_available(row.source.card, |f| match (f, row.maximum) {
                    (F::NoMaximumHandSize, None) => true,
                    (F::MaximumHandSize(a), Some(b)) => *a == b,
                    _ => false,
                })
            {
                result = (row.timestamp, row.maximum.map(usize::from));
            }
        }
        result.1
    }
    pub(super) fn above_hand_limit(&self, player: Player, selected: usize) -> bool {
        self.maximum_hand_size(player)
            .is_some_and(|n| self.hand_count(player) - selected > n)
    }
    pub(super) fn hand_size_timeline(&self) -> Option<HandSizeTimelineView> {
        let s = self.state.hand_size.as_ref()?;
        // Without a finite setter all existing effects commute and set infinity.
        // Their chronology adds no public distinction to the existing effective
        // bool. Once finite rules participate, expose every competing owned row,
        // even when a provider is suppressed or a later source hides a grant.
        s.finite_participated.then(|| HandSizeTimelineView {
            sources: s.sources.clone(),
            lasting: s.lasting,
            maximum: [
                self.maximum_hand_size(Player::P0),
                self.maximum_hand_size(Player::P1),
            ],
        })
    }
    pub(super) fn hand_entry_order(batch: &entries::EntryBatch, active: Player) -> Vec<usize> {
        Self::hand_entry_order_for(&batch.moves, &batch.timestamp_order, active)
    }
    pub(super) fn hand_entry_order_for(
        moves: &[entries::EntryMove],
        order: &[usize],
        active: Player,
    ) -> Vec<usize> {
        let has_finite = moves
            .iter()
            .any(|m| Self::hand_entry(m) && m.before.card.hand_size_setting().flatten().is_some());
        if has_finite {
            return order
                .iter()
                .copied()
                .filter(|&i| Self::hand_entry(&moves[i]))
                .collect();
        }
        // Equal infinity setters commute; no additional old-game choice is needed.
        let mut ids: Vec<_> = moves
            .iter()
            .enumerate()
            .filter_map(|(i, m)| Self::hand_entry(m).then_some(i))
            .collect();
        ids.sort_by_key(|&i| {
            (
                moves[i].controller != active,
                order.iter().position(|&j| j == i).unwrap_or(usize::MAX),
                i,
            )
        });
        ids
    }
    pub(super) fn hand_entry(m: &entries::EntryMove) -> bool {
        !m.skipped
            && m.destination == Zone::Battlefield
            && m.before.zone != Zone::Battlefield
            && m.before.card.hand_size_setting().is_some()
    }
}
