//! Independent owned chronology for rule-layer hand-size setters.
use super::*;
use crate::cards::StaticAbilityFragment as F;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct HandSizeSourceView {
    pub source: ObjectRef,
    pub timestamp: u64,
    /// The value of this setter before considering availability or other setters.
    /// None denotes an unlimited maximum.
    pub maximum: Option<u16>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HandSizeTimelineView {
    pub sources: Vec<HandSizeSourceView>,
    pub lasting: [Option<u64>; 2],
    pub maximum: [Option<usize>; 2],
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HandSizeSetting {
    Unlimited,
    Fixed(u16),
    OpponentGraveyardTypes { base: u16, minimum: u16 },
}
impl HandSizeSetting {
    fn from_fragment(fragment: &F) -> Option<Self> {
        match *fragment {
            F::NoMaximumHandSize => Some(Self::Unlimited),
            F::MaximumHandSize(n) => Some(Self::Fixed(n)),
            F::OpponentMaximumHandSizeByGraveyardTypes { base, minimum } => {
                Some(Self::OpponentGraveyardTypes { base, minimum })
            }
            _ => None,
        }
    }
    fn finite(self) -> bool {
        self != Self::Unlimited
    }
    fn affected_player(self, controller: Player) -> Player {
        match self {
            Self::OpponentGraveyardTypes { .. } => controller.other(),
            Self::Unlimited | Self::Fixed(_) => controller,
        }
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct HandSizeSource {
    pub source: ObjectRef,
    pub timestamp: u64,
    setting: HandSizeSetting,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct HandSizeState {
    clock: u64,
    pub deferred: bool,
    pub sources: Vec<HandSizeSource>,
    pub lasting: [Option<u64>; 2],
    finite_participated: bool,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
        if let Some(setting) = card.hand_size_setting() {
            let timestamp = self.next();
            self.insert(source, timestamp, setting);
        }
    }
    pub(super) fn register_at(&mut self, source: ObjectRef, card: Card, timestamp: u64) {
        if let Some(setting) = card.hand_size_setting() {
            self.insert(source, timestamp, setting);
        }
    }
    fn insert(&mut self, source: ObjectRef, timestamp: u64, setting: HandSizeSetting) {
        self.finite_participated |= setting.finite();
        self.sources.push(HandSizeSource {
            source,
            timestamp,
            setting,
        });
        // Physical entry order and chosen timestamp order are independent.
        self.sources.sort_by_key(|row| row.source);
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
        std::mem::size_of::<Self>() + self.sources.len() * std::mem::size_of::<HandSizeSource>()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Card {
    pub(super) fn hand_size_setting(self) -> Option<HandSizeSetting> {
        self.static_ability_groups()
            .iter()
            .flat_map(|g| g.fragments)
            .find_map(HandSizeSetting::from_fragment)
    }
    pub(super) fn has_finite_hand_size_setting(self) -> bool {
        self.hand_size_setting()
            .is_some_and(HandSizeSetting::finite)
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn new_copy_hand_timestamp(&mut self, card: Card) -> Option<u64> {
        let setting = card.hand_size_setting()?;
        let state = self.state.hand_size.get_or_insert_default();
        state.finite_participated |= setting.finite();
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
            let id = row.source.card;
            let controller = self.state.objects[id as usize].controller;
            if row.timestamp < result.0
                || row.setting.affected_player(controller) != player
                || !self.on_battlefield(id)
                || self.object_ref(id) != row.source
                || !self.continuous_fragment_available(id, |fragment| {
                    HandSizeSetting::from_fragment(fragment) == Some(row.setting)
                })
            {
                continue;
            }
            let (maximum, active) = self.hand_size_assignment(row, controller);
            if active {
                result = (row.timestamp, maximum);
            }
        }
        result.1.map(usize::from)
    }
    fn hand_size_assignment(
        &self,
        row: &HandSizeSource,
        controller: Player,
    ) -> (Option<u16>, bool) {
        match row.setting {
            HandSizeSetting::Unlimited => (None, true),
            HandSizeSetting::Fixed(maximum) => (Some(maximum), true),
            HandSizeSetting::OpponentGraveyardTypes { base, minimum } => {
                let count = self.stat_value(
                    StatValue::CONTROLLER_GRAVEYARD_CARD_TYPES,
                    row.source,
                    controller,
                );
                let count = u16::try_from(count).capacity("card type count exhausted");
                (Some(base.saturating_sub(count)), count >= minimum)
            }
        }
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
            sources: s
                .sources
                .iter()
                .map(|row| {
                    let controller = self.state.objects[row.source.card as usize].controller;
                    let (maximum, _) = self.hand_size_assignment(row, controller);
                    HandSizeSourceView {
                        source: row.source,
                        timestamp: row.timestamp,
                        maximum,
                    }
                })
                .collect(),
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
            .any(|m| Self::hand_entry(m) && m.before.card.has_finite_hand_size_setting());
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
