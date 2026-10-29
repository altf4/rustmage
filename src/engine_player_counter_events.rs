//! Owned player-counter event identities for replacement-aware simultaneous batches.
//! A redirect or split never merges with another pending event, even if the
//! resulting player and counter kind coincide.
use super::*;
use crate::cards::PlayerCounterKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct EffectIdentity {
    pub origin_stack_id: u64,
    pub index: u16,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PlayerCounterEvent {
    pub id: u64,
    pub player: Player,
    pub kind: PlayerCounterKind,
    pub amount: u64,
    pub effect_origin: u64,
    pub applied_self: Vec<EffectIdentity>,
    pub applied_external: Vec<(ObjectRef, u16)>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CounterDestination {
    pub player: Player,
    pub kind: PlayerCounterKind,
    pub amount: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct PlayerCounterEvents {
    next_id: u64,
    events: Vec<PlayerCounterEvent>,
}
impl PlayerCounterEvents {
    pub fn push(&mut self, player: Player, kind: PlayerCounterKind, amount: u64, effect_origin: u64) -> u64 {
        let id = self.allocate_id();
        self.events.push(PlayerCounterEvent {
            id, player, kind, amount, effect_origin, applied_self: Vec::new(), applied_external: Vec::new(),
        });
        id
    }
    fn allocate_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.checked_add(1).capacity("player counter event identities exhausted");
        id
    }
    pub fn events(&self) -> &[PlayerCounterEvent] { &self.events }
    pub fn event_mut(&mut self, index: usize) -> &mut PlayerCounterEvent { &mut self.events[index] }
    pub fn totals(&self) -> [[u64; 5]; 2] {
        let mut totals = [[0u64; 5]; 2];
        for event in &self.events {
            let value = &mut totals[event.player.index()][event.kind.index()];
            *value = value.checked_add(event.amount).capacity("player counter event total exhausted");
        }
        totals
    }
    /// Replace one event with zero or more distinct descendants. Applied effects
    /// are inherited by every descendant, and exact origin identity is retained.
    pub fn apply_self(&mut self, event_id: u64, effect: EffectIdentity, outcomes: &[CounterDestination]) -> bool {
        let Some(index) = self.events.iter().position(|event| event.id == event_id) else { return false; };
        let event = &self.events[index];
        if event.effect_origin != effect.origin_stack_id || event.applied_self.contains(&effect) { return false; }
        let mut applied = event.applied_self.clone();
        applied.push(effect);
        let inherited_external = event.applied_external.clone();
        let origin = event.effect_origin;
        let mut descendants = Vec::with_capacity(outcomes.len());
        for outcome in outcomes {
            let id = self.allocate_id();
            descendants.push(PlayerCounterEvent {
                id, player: outcome.player, kind: outcome.kind, amount: outcome.amount,
                effect_origin: origin, applied_self: applied.clone(), applied_external: inherited_external.clone(),
            });
        }
        self.events.splice(index..=index, descendants);
        true
    }
    pub fn mark_external(&mut self, event_id: u64, effect: (ObjectRef, u16)) -> bool {
        let Some(event) = self.events.iter_mut().find(|event| event.id == event_id) else { return false; };
        if event.applied_external.contains(&effect) { return false; }
        event.applied_external.push(effect);
        true
    }
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.events.len() * std::mem::size_of::<PlayerCounterEvent>()
            + self.events.iter().map(|event| event.applied_self.len() * std::mem::size_of::<EffectIdentity>()
                + event.applied_external.len() * std::mem::size_of::<(ObjectRef, u16)>()).sum::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn redirected_and_split_events_keep_separate_replacement_history() {
        for player in [Player::P0, Player::P1] {
            let mut queue = PlayerCounterEvents::default();
            let initial = queue.push(player, PlayerCounterKind::Poison, 2, 41);
            let other = queue.push(player.other(), PlayerCounterKind::Energy, 1, 42);
            let source = ObjectRef { card: 3, generation: 1 };
            assert!(queue.mark_external(initial, (source, 7)));
            let applied = EffectIdentity { origin_stack_id: 41, index: 0 };
            let destination = CounterDestination { player: player.other(), kind: PlayerCounterKind::Energy, amount: 3 };
            assert!(queue.apply_self(initial, applied, &[destination, destination]));
            assert_eq!(queue.events().len(), 3);
            assert_ne!(queue.events()[0].id, queue.events()[1].id);
            assert_ne!(queue.events()[1].id, other);
            for event in &queue.events()[..2] {
                assert_eq!(event.effect_origin, 41);
                assert_eq!(event.applied_self, vec![applied]);
                assert_eq!(event.applied_external, vec![(source, 7)]);
            }
            assert!(queue.events()[2].applied_external.is_empty());
            let saved = queue.clone();
            let first = queue.events()[0].id;
            assert!(!queue.apply_self(first, applied, &[destination]));
            assert!(!queue.apply_self(first, EffectIdentity { origin_stack_id: 42, index: 1 }, &[destination]));
            assert!(queue.mark_external(first, (source, 8)));
            assert!(!queue.mark_external(first, (source, 8)));
            assert_eq!(queue.events()[1].applied_external, vec![(source, 7)]);
            queue = saved.clone();
            assert_eq!(queue, saved);
            assert!(queue.apply_self(first, EffectIdentity { origin_stack_id: 41, index: 1 }, &[]));
            assert_eq!(queue.events().len(), 2);
        }
    }
}
