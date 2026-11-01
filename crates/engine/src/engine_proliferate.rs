//! Owned sequential proliferate choices and their simultaneous counter event.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProliferateGroup {
    pub count: u128,
    pub used: Vec<(ObjectRef, u16)>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProliferateSequence {
    pub player: Player,
    pub remaining: u128,
    /// Compact equivalent siblings retain exactly the replacement effects already applied.
    pub pending: std::collections::VecDeque<ProliferateGroup>,
    pub ready: bool,
    pub permanents: Vec<ObjectRef>,
    pub players: u8,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
#[derive(Clone, Debug, Serialize)]
pub struct ProliferateView {
    pub player: Player,
    pub remaining: u128,
    pub selected_permanents: Vec<ObjectRef>,
    pub selected_players: Vec<Player>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ProliferateSequence {
    pub(super) fn view(&self) -> ProliferateView {
        ProliferateView {
            player: self.player,
            remaining: self.pending.iter().fold(
                self.remaining
                    .checked_add(u128::from(self.ready))
                    .capacity("proliferate quantity exhausted"),
                |total, group| {
                    total
                        .checked_add(group.count)
                        .capacity("proliferate quantity exhausted")
                },
            ),
            selected_permanents: self.permanents.clone(),
            selected_players: [Player::P0, Player::P1]
                .into_iter()
                .filter(|p| self.players & (1 << p.index()) != 0)
                .collect(),
        }
    }
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.permanents.len() * std::mem::size_of::<ObjectRef>()
            + self
                .pending
                .iter()
                .map(|group| {
                    std::mem::size_of::<ProliferateGroup>()
                        + group.used.len() * std::mem::size_of::<(ObjectRef, u16)>()
                })
                .sum::<usize>()
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn proliferated_event(&mut self, player: Player) {
        let ids: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, object)| (object.zone == Zone::Battlefield).then_some(id as CardId))
            .collect();
        let sources = self.indexed_trigger_sources(&ids);
        for (source, grant) in sources {
            let definition = grant.as_ref().map_or(source.card, |g| g.granting_card);
            for (index, ability) in definition.triggers().iter().enumerate() {
                if let TriggerCondition::Proliferated(relation) = ability.condition.event() {
                    let matches = match relation {
                        CastRelation::You => source.controller == player,
                        CastRelation::Opponent => source.controller != player,
                        CastRelation::Any => true,
                    };
                    if matches {
                        self.queue_trigger_from(&source, index, grant.map(Box::new));
                    }
                }
            }
        }
    }
    pub(super) fn start_proliferate(&mut self, player: Player, remaining: u128) {
        if remaining != 0 {
            self.state.resolving.as_mut().unwrap().proliferate =
                Some(Box::new(ProliferateSequence {
                    player,
                    remaining,
                    pending: std::collections::VecDeque::new(),
                    ready: false,
                    permanents: Vec::new(),
                    players: 0,
                }));
        }
    }
    pub(super) fn advance_proliferate(&mut self) -> bool {
        loop {
            let Some(sequence) = self.state.resolving.as_mut().unwrap().proliferate.as_mut() else {
                return false;
            };
            if sequence.ready {
                self.state.pending = Pending::Proliferate {
                    player: sequence.player,
                };
                return true;
            }
            let player = sequence.player;
            let used = if let Some(group) = sequence.pending.front_mut() {
                let used = group.used.clone();
                group.count -= 1;
                if group.count == 0 {
                    sequence.pending.pop_front();
                }
                used
            } else if sequence.remaining > 0 {
                sequence.remaining -= 1;
                Vec::new()
            } else {
                self.state.resolving.as_mut().unwrap().proliferate = None;
                return false;
            };
            // Each derived event retains its own applied-effect identities. Resolve
            // this event and its choice before inspecting its next sibling.
            self.stage_proliferate_replacements(player, used);
            if self.advance_replacements(false) {
                return true;
            }
        }
    }
    pub(super) fn finish_proliferate(&mut self) {
        let mut sequence = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .proliferate
            .take()
            .unwrap();
        let mut counters = [[0; 5]; 2];
        for player in [Player::P0, Player::P1] {
            if sequence.players & (1 << player.index()) != 0 {
                for kind in crate::cards::PlayerCounterKind::ALL {
                    if self.state.players[player.index()].player_counter(kind) > 0 {
                        counters[player.index()][kind.index()] = 1;
                    }
                }
            }
        }
        let changes = sequence
            .permanents
            .drain(..)
            .filter_map(|object| {
                if !self.on_battlefield(object.card) || self.object_ref(object.card) != object {
                    return None;
                }
                let present = self.counter_amounts(object.card);
                let mut added = Counters::default();
                for kind in CounterKind::ALL {
                    if present.get(kind) > 0 {
                        added.add(kind, 1);
                    }
                }
                added.has_any().then_some((object, added))
            })
            .collect();
        sequence.ready = false;
        sequence.players = 0;
        let player = sequence.player;
        if sequence.remaining != 0 || !sequence.pending.is_empty() {
            self.state.resolving.as_mut().unwrap().proliferate = Some(sequence);
        }
        self.state.pending = Pending::Priority;
        self.proliferate_counters(changes, counters, player);
    }
}
