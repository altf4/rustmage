//! Exact post-mill incarnations, public destinations and non-targeted choices.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct MilledSelection {
    player: Player,
    spec: MilledSelectionSpec,
    selected: Vec<ObjectRef>,
}
impl MilledSelection {
    pub fn bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.selected.len() * std::mem::size_of::<ObjectRef>()
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct MilledSelectionView {
    pub player: Player,
    pub filter: MilledFilter,
    pub destination: MilledDestination,
    pub minimum: u16,
    pub maximum: u16,
    pub milled: Vec<ObjectRef>,
    pub selected: Vec<ObjectRef>,
    pub candidates: Vec<ObjectRef>,
}
impl Game {
    fn public_milled_object(&self, object: ObjectRef) -> bool {
        self.state
            .objects
            .get(object.card as usize)
            .is_some_and(|card| {
                card.generation == object.generation
                    && matches!(
                        card.zone,
                        Zone::Battlefield
                            | Zone::Graveyard
                            | Zone::Exile
                            | Zone::Stack
                            | Zone::Command
                    )
                    && !card.is_token()
                    && !card.card.is_emblem()
            })
    }
    fn milled_matches(&self, player: Player, filter: MilledFilter, object: ObjectRef) -> bool {
        if !self.public_milled_object(object) {
            return false;
        }
        let card = &self.state.objects[object.card as usize];
        if card.owner != player {
            return false;
        }
        let kinds = self.current_kinds(object.card);
        match filter {
            MilledFilter::Any => true,
            MilledFilter::Kinds(mask) => kinds & u16::from(mask) != 0,
            MilledFilter::KindsOrSubtype {
                kinds: mask,
                subtype,
            } => kinds & u16::from(mask) != 0 || self.has_subtype(object.card, subtype),
            MilledFilter::Adventure => card.card.adventure_spell().is_some(),
        }
    }
    fn milled_selection(&self) -> Option<&MilledSelection> {
        self.state.resolving.as_ref()?.milled_selection.as_deref()
    }
    pub(super) fn is_milled_selection_candidate(&self, object: ObjectRef) -> bool {
        let Some(choice) = self.milled_selection() else {
            return false;
        };
        let MilledCount::Range { maximum, .. } = choice.spec.count else {
            return false;
        };
        choice.selected.len() < usize::from(maximum)
            && !choice.selected.contains(&object)
            && self
                .state
                .resolving
                .as_ref()
                .unwrap()
                .milled
                .contains(&object)
            && self.milled_matches(choice.player, choice.spec.filter, object)
    }
    pub(super) fn can_finish_milled_selection(&self) -> bool {
        let Some(choice) = self.milled_selection() else {
            return false;
        };
        let MilledCount::Range { minimum, .. } = choice.spec.count else {
            return false;
        };
        choice.selected.len() >= usize::from(minimum)
            || !self
                .state
                .resolving
                .as_ref()
                .unwrap()
                .milled
                .iter()
                .any(|&object| self.is_milled_selection_candidate(object))
    }
    pub(super) fn milled_selection_view(&self, viewer: Player) -> Option<MilledSelectionView> {
        let choice = self.milled_selection()?;
        if choice.player != viewer {
            return None;
        }
        let MilledCount::Range { minimum, maximum } = choice.spec.count else {
            return None;
        };
        Some(MilledSelectionView {
            player: choice.player,
            filter: choice.spec.filter,
            destination: choice.spec.destination,
            minimum,
            maximum,
            milled: self
                .state
                .resolving
                .as_ref()
                .unwrap()
                .milled
                .iter()
                .copied()
                .filter(|&object| self.public_milled_object(object))
                .collect(),
            selected: choice.selected.clone(),
            candidates: self
                .state
                .resolving
                .as_ref()
                .unwrap()
                .milled
                .iter()
                .copied()
                .filter(|&object| self.is_milled_selection_candidate(object))
                .collect(),
        })
    }
    pub(super) fn start_milled_selection(&mut self, spec: MilledSelectionSpec) -> bool {
        let resolution = self.state.resolving.as_ref().unwrap();
        let player = resolution.item.controller;
        let eligible: Vec<_> = resolution
            .milled
            .iter()
            .copied()
            .filter(|&object| self.milled_matches(player, spec.filter, object))
            .collect();
        if eligible.is_empty() {
            return false;
        }
        if spec.count == MilledCount::All {
            self.move_milled_selection(player, &eligible, spec.destination);
            return self.state.entry_batch.is_some();
        }
        self.state.resolving.as_mut().unwrap().milled_selection = Some(Box::new(MilledSelection {
            player,
            spec,
            selected: Vec::new(),
        }));
        self.state.pending = Pending::MilledSelection { player };
        true
    }
    pub(super) fn select_milled(&mut self, object: ObjectRef) {
        assert!(self.is_milled_selection_candidate(object));
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .milled_selection
            .as_mut()
            .unwrap();
        choice.selected.push(object);
        let MilledCount::Range { maximum, .. } = choice.spec.count else {
            unreachable!()
        };
        if choice.selected.len() == usize::from(maximum) {
            self.finish_milled_selection();
        }
    }
    pub(super) fn finish_milled_selection(&mut self) {
        assert!(self.can_finish_milled_selection());
        let choice = self
            .state
            .resolving
            .as_mut()
            .unwrap()
            .milled_selection
            .take()
            .unwrap();
        let selected: Vec<_> = choice
            .selected
            .into_iter()
            .filter(|&object| self.milled_matches(choice.player, choice.spec.filter, object))
            .collect();
        self.move_milled_selection(choice.player, &selected, choice.spec.destination);
        self.run_program();
    }
    fn move_milled_selection(
        &mut self,
        player: Player,
        selected: &[ObjectRef],
        destination: MilledDestination,
    ) {
        if let MilledDestination::PlayPermission { next_turn } = destination {
            for &object in selected {
                self.grant_exile_play_permission(player, object, next_turn);
            }
            return;
        }
        // A public replacement destination can be the stack. Removing its spell
        // packet is movement, not a counter event. Hidden destinations never enter
        // this continuation and no old-generation object can be recovered.
        self.state
            .stack
            .retain(|item| !selected.contains(&item.object));
        let destination = match destination {
            MilledDestination::PlayPermission { .. } => {
                unreachable!("handled permission without movement")
            }
            MilledDestination::Hand => Zone::Hand,
            MilledDestination::Battlefield => Zone::Battlefield,
        };
        let ids: Vec<_> = selected
            .iter()
            .filter(|object| self.state.objects[object.card as usize].zone != destination)
            .map(|object| object.card)
            .collect();
        if destination == Zone::Battlefield {
            let moves = ids
                .into_iter()
                .map(|id| {
                    let mut incoming = self.entry_move(id, destination, false);
                    incoming.controller = player;
                    incoming
                })
                .collect();
            // Existing entry continuation handles mandatory Aura attachment and
            // retains an impossible Aura in its previous public zone, except
            // an Aura on the stack goes to its owner's graveyard (CR303.4g).
            self.stage_entries(moves, None);
        } else {
            let events = self.move_objects_raw(&ids, destination);
            self.finish_zone_events(&events, &[]);
            for event in events {
                if event.moved
                    && event.destination == Zone::Hand
                    && self.state.objects[event.after.card as usize].zone == Zone::Hand
                    && self.object_ref(event.after.card) == event.after
                    && !self.state.known_hand.contains(&event.after)
                {
                    self.state.known_hand.push(event.after);
                }
            }
        }
    }
}
