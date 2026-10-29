//! Owned sequential proliferate choices and their simultaneous counter event.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProliferateSequence {
    pub player: Player,
    pub remaining: u128,
    /// Events created by the current replacement process, not re-replaced.
    pub expanded_remaining: u128,
    pub permanents: Vec<ObjectRef>,
    pub players: u8,
}
#[derive(Clone, Debug, Serialize)]
pub struct ProliferateView {
    pub player: Player,
    pub remaining: u128,
    pub selected_permanents: Vec<ObjectRef>,
    pub selected_players: Vec<Player>,
}
impl ProliferateSequence {
    pub(super) fn view(&self) -> ProliferateView {
        ProliferateView {
            player: self.player, remaining: self.remaining.checked_add(self.expanded_remaining).capacity("proliferate quantity exhausted"),
            selected_permanents: self.permanents.clone(),
            selected_players: [Player::P0, Player::P1].into_iter()
                .filter(|p| self.players & (1 << p.index()) != 0).collect(),
        }
    }
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.permanents.len() * std::mem::size_of::<ObjectRef>()
    }
}
impl Game {
    pub(super) fn proliferated_event(&mut self, player: Player) {
        let ids: Vec<_> = self.state.objects.iter().enumerate()
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
                    if matches { self.queue_trigger_from(&source, index, grant.clone().map(Box::new)); }
                }
            }
        }
    }
    pub(super) fn start_proliferate(&mut self, player: Player, remaining: u128) {
        if remaining != 0 {
            self.state.resolving.as_mut().unwrap().proliferate = Some(Box::new(ProliferateSequence {
                player, remaining, expanded_remaining: 0, permanents: Vec::new(), players: 0,
            }));
        }
    }
    pub(super) fn advance_proliferate(&mut self) -> bool {
        loop {
            let Some(sequence) = self.state.resolving.as_ref().unwrap().proliferate.as_ref() else {
                return false;
            };
            if sequence.expanded_remaining > 0 {
                self.state.pending = Pending::Proliferate { player: sequence.player };
                return true;
            }
            if sequence.remaining == 0 {
                self.state.resolving.as_mut().unwrap().proliferate = None;
                return false;
            }
            let player = sequence.player;
            self.state.resolving.as_mut().unwrap().proliferate.as_mut().unwrap().remaining -= 1;
            // One original event is replaced once. Resulting copies bypass this
            // replacement loop, but the next original event gets its own choices.
            self.stage_proliferate_replacements(player);
            if self.advance_replacements(false) { return true; }
        }
    }
    pub(super) fn finish_proliferate(&mut self) {
        let mut sequence = self.state.resolving.as_mut().unwrap().proliferate.take().unwrap();
        let mut poison = [0; 2];
        for player in [Player::P0, Player::P1] {
            if sequence.players & (1 << player.index()) != 0
                && self.state.players[player.index()].poison_counters > 0 {
                poison[player.index()] = 1;
            }
        }
        let changes = sequence.permanents.drain(..).filter_map(|object| {
            if !self.on_battlefield(object.card) || self.object_ref(object.card) != object { return None; }
            let present = self.counter_amounts(object.card);
            let mut added = Counters::default();
            for kind in CounterKind::ALL {
                if present.get(kind) > 0 { added.add(kind, 1); }
            }
            added.has_any().then_some((object, added))
        }).collect();
        sequence.expanded_remaining -= 1;
        sequence.players = 0;
        let player = sequence.player;
        if sequence.remaining != 0 || sequence.expanded_remaining != 0 {
            self.state.resolving.as_mut().unwrap().proliferate = Some(sequence);
        }
        self.state.pending = Pending::Priority;
        self.proliferate_counters(changes, poison, player);
    }
}
