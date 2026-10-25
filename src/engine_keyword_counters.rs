//! Sparse keyword counters use the ordinary layer-six chronology (CR122.1b/613.7c).
use super::*;

impl CounterKind {
    pub(crate) const KEYWORDS: [Self; 11] = [
        Self::Haste,
        Self::Flying,
        Self::FirstStrike,
        Self::DoubleStrike,
        Self::Deathtouch,
        Self::Indestructible,
        Self::Lifelink,
        Self::Menace,
        Self::Reach,
        Self::Trample,
        Self::Vigilance,
    ];
    pub(crate) fn keyword(self) -> Option<Keywords> {
        Some(match self {
            Self::Haste => Keywords::HASTE,
            Self::Flying => Keywords::FLYING,
            Self::FirstStrike => Keywords::FIRST_STRIKE,
            Self::DoubleStrike => Keywords::DOUBLE_STRIKE,
            Self::Deathtouch => Keywords::DEATHTOUCH,
            Self::Indestructible => Keywords::INDESTRUCTIBLE,
            Self::Lifelink => Keywords::LIFELINK,
            Self::Menace => Keywords::MENACE,
            Self::Reach => Keywords::REACH,
            Self::Trample => Keywords::TRAMPLE,
            Self::Vigilance => Keywords::VIGILANCE,
            _ => return None,
        })
    }
}
impl CounterAmounts<'_> {
    pub(super) fn keyword_amounts(self) -> [u64; 11] {
        if self.named.is_none() {
            return [0; 11];
        }
        CounterKind::KEYWORDS.map(|kind| self.get(kind))
    }
}
/// One row represents all interchangeable counters of one kind on one object.
/// Increasing their amount refreshes the entire kind; removing some does not.
pub(super) fn sync_effects(
    state: &mut roles::RoleState,
    object: ObjectRef,
    before: [u64; 11],
    after: [u64; 11],
    timestamp: Option<u64>,
) {
    for (i, kind) in CounterKind::KEYWORDS.into_iter().enumerate() {
        if after[i] == before[i] || (after[i] > 0 && after[i] < before[i]) {
            continue;
        }
        state.keyword_effects.retain(|row| {
            row.recipient != object || row.origin != KeywordEffectOrigin::Counter { kind }
        });
        if after[i] > 0 {
            let time = timestamp.unwrap_or_else(|| state.next_timestamp());
            state.keyword_effects.push(KeywordEffectView {
                recipient: object,
                origin: KeywordEffectOrigin::Counter { kind },
                keywords: kind.keyword().unwrap(),
                rules: Keywords::NONE,
                timestamp: time,
                expires_turn: None,
            });
        }
    }
}
/// An overlay may have removed counters without altering the parent's timeline.
pub(super) fn applies<W: ability_world::CharacteristicWorld + ?Sized>(
    world: &W,
    row: &KeywordEffectView,
) -> bool {
    match row.origin {
        KeywordEffectOrigin::Counter { kind } => world.counter_count(row.recipient.card, kind) > 0,
        _ => true,
    }
}
impl Game {
    pub(super) fn keyword_effect_views(&self, viewer: Player) -> Vec<KeywordEffectView> {
        self.state.roles.as_ref().map_or_else(Vec::new, |state| {
            state
                .keyword_effects
                .iter()
                .filter(|row| {
                    if !matches!(row.origin, KeywordEffectOrigin::Counter { .. }) {
                        return true;
                    }
                    let object = &self.state.objects[row.recipient.card as usize];
                    self.object_ref(row.recipient.card) == row.recipient
                        && self.can_view_hideaway(viewer, row.recipient)
                        && !matches!(object.zone, Zone::Outside | Zone::Library)
                        && (object.zone != Zone::Hand
                            || self.shares_information(viewer, object.owner)
                            || self.state.known_hand.contains(&row.recipient))
                })
                .copied()
                .collect()
        })
    }
}
