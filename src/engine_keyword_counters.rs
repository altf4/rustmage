//! Keyword counters are independent layer-six grants, ordered against removal.
use super::ability_world::CharacteristicWorld;
use super::*;

pub(super) const KINDS: [CounterKind; 3] = [
    CounterKind::Flying,
    CounterKind::FirstStrike,
    CounterKind::Lifelink,
];
fn keyword(kind: CounterKind) -> Keywords {
    match kind {
        CounterKind::Flying => Keywords::FLYING,
        CounterKind::FirstStrike => Keywords::FIRST_STRIKE,
        CounterKind::Lifelink => Keywords::LIFELINK,
        _ => Keywords::NONE,
    }
}
pub(super) fn keywords(amounts: CounterAmounts<'_>) -> Keywords {
    KINDS.into_iter().fold(Keywords::NONE, |flags, kind| {
        if amounts.get(kind) > 0 {
            flags.union(keyword(kind))
        } else {
            flags
        }
    })
}
pub(super) fn effect_live<W: CharacteristicWorld + ?Sized>(
    world: &W,
    row: &KeywordEffectView,
) -> bool {
    match row.origin {
        KeywordEffectOrigin::Counter { kind } => {
            world.counter_amounts(row.recipient.card).get(kind) > 0
        }
        _ => true,
    }
}
impl Game {
    pub(super) fn sync_keyword_counters(&mut self, id: CardId, before: [u64; 3]) {
        // Private-zone counters grant abilities directly through the borrowed
        // query; no public timeline row can reveal a private card's identity.
        if !matches!(
            self.state.objects[id as usize].zone,
            Zone::Battlefield | Zone::Stack
        ) {
            return;
        }
        let recipient = self.object_ref(id);
        for (kind, old) in KINDS.into_iter().zip(before) {
            let now = self.counter_count(id, kind);
            if now == old {
                continue;
            }
            if (now == 0 || now > old)
                && let Some(state) = self.state.roles.as_mut()
            {
                state.keyword_effects.retain(|row| {
                    row.recipient != recipient
                        || row.origin != (KeywordEffectOrigin::Counter { kind })
                });
            }
            // CR 613.7c: adding another counter retimestamps every counter of
            // that kind. Removing only some leaves the remaining timestamp.
            if now > old {
                let timestamp = self.next_role_timestamp();
                self.state
                    .roles
                    .get_or_insert_with(Default::default)
                    .keyword_effects
                    .push(KeywordEffectView {
                        recipient,
                        origin: KeywordEffectOrigin::Counter { kind },
                        keywords: keyword(kind),
                        rules: Keywords::NONE,
                        timestamp,
                        expires_turn: None,
                        expires_before_turn: None,
                    });
            }
        }
    }
}
