//! Owned pile/type choices and exact links shared by the Foundations choice batch.
use super::*;
use crate::PermanentType;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LibraryPile {
    FaceDown,
    FaceUp,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Piles {
    controller: Player,
    remaining: Vec<ObjectRef>,
    down: Vec<ObjectRef>,
    up: Vec<ObjectRef>,
    choosing: bool,
}
#[derive(Clone, Debug, Serialize)]
pub struct LibraryPilesView {
    pub controller: Player,
    pub choosing: bool,
    pub remaining: Vec<ObjectView>,
    pub face_down: Option<Vec<ObjectView>>,
    pub face_down_count: usize,
    pub face_up: Vec<ObjectView>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct KeepTypes {
    player: Player,
    next_type: usize,
    selected: Vec<(PermanentType, ObjectRef)>,
}
#[derive(Clone, Debug, Serialize)]
pub struct KeepTypesView {
    pub player: Player,
    pub permanent_type: PermanentType,
    pub selected: Vec<(PermanentType, ObjectRef)>,
    pub candidates: Vec<ObjectRef>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct LinkedExileView {
    pub source: ObjectRef,
    pub exiled: ObjectRef,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ChoiceState {
    piles: Option<Piles>,
    keep_types: Option<KeepTypes>,
    // Some(None) means tracking this instruction but no successful own sacrifice.
    sacrifice: Option<Option<ObjectRef>>,
    pub links: Vec<LinkedExileView>,
}
impl ChoiceState {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.links.len() * std::mem::size_of::<LinkedExileView>()
            + self.piles.as_ref().map_or(0, |p| {
                (p.remaining.len() + p.up.len() + p.down.len()) * std::mem::size_of::<ObjectRef>()
            })
            + self.keep_types.as_ref().map_or(0, |p| {
                p.selected.len() * std::mem::size_of::<(PermanentType, ObjectRef)>()
            })
    }
}
impl Game {
    pub(super) fn abandon_linked_choices(&mut self) {
        if let Some(data) = self.state.linked_choices.as_mut() {
            data.piles = None;
            data.keep_types = None;
            data.sacrifice = None;
            if data.links.is_empty() {
                self.state.linked_choices = None;
            }
        }
    }
    pub(super) fn start_library_piles(&mut self, player: Player, maximum: u16) {
        let remaining = self.state.players[player.index()]
            .library
            .iter()
            .rev()
            .take(usize::from(maximum))
            .map(|&id| self.object_ref(id))
            .collect();
        self.state.linked_choices.get_or_insert_default().piles = Some(Piles {
            controller: player,
            remaining,
            down: Vec::new(),
            up: Vec::new(),
            choosing: false,
        });
        self.advance_library_piles();
    }
    fn advance_library_piles(&mut self) {
        let piles = self
            .state
            .linked_choices
            .as_mut()
            .unwrap()
            .piles
            .as_mut()
            .unwrap();
        if piles.remaining.is_empty() {
            piles.choosing = true;
        }
        self.state.pending = Pending::LinkedChoice {
            player: if piles.choosing {
                piles.controller.other()
            } else {
                piles.controller
            },
        };
    }
    pub(super) fn assign_library_pile(&mut self, object: ObjectRef, pile: LibraryPile) {
        let piles = self
            .state
            .linked_choices
            .as_mut()
            .unwrap()
            .piles
            .as_mut()
            .unwrap();
        let at = piles.remaining.iter().position(|&o| o == object).unwrap();
        piles.remaining.remove(at);
        match pile {
            LibraryPile::FaceDown => piles.down.push(object),
            LibraryPile::FaceUp => piles.up.push(object),
        }
        self.advance_library_piles();
    }
    pub(super) fn choose_library_pile(&mut self, pile: LibraryPile) {
        let piles = self
            .state
            .linked_choices
            .as_mut()
            .unwrap()
            .piles
            .take()
            .unwrap();
        // Do not disclose an unchosen hidden pile by preserving the former known top.
        self.forget_library_top(piles.controller);
        let moves = [
            (LibraryPile::FaceDown, piles.down),
            (LibraryPile::FaceUp, piles.up),
        ]
        .into_iter()
        .flat_map(|(side, objects)| objects.into_iter().map(move |object| (side, object)))
        .map(|(side, object)| {
            let destination = if side == pile {
                Zone::Hand
            } else {
                Zone::Graveyard
            };
            let mut movement = self.entry_move(object.card, destination, false);
            movement.reveal_hand = side == LibraryPile::FaceUp && destination == Zone::Hand;
            movement
        })
        .collect();
        self.stage_entries(moves, None);
        self.run_program();
    }
    pub(super) fn library_piles_view(&self, viewer: Player) -> Option<LibraryPilesView> {
        let p = self.state.linked_choices.as_ref()?.piles.as_ref()?;
        // The split itself is private until both piles have been made.
        if viewer != p.controller && !p.choosing {
            return None;
        }
        let views =
            |objects: &[ObjectRef]| objects.iter().map(|o| self.object_view(o.card)).collect();
        Some(LibraryPilesView {
            controller: p.controller,
            choosing: p.choosing,
            remaining: if viewer == p.controller {
                views(&p.remaining)
            } else {
                Vec::new()
            },
            face_down: (viewer == p.controller).then(|| views(&p.down)),
            face_down_count: p.down.len(),
            face_up: views(&p.up),
        })
    }
    pub(super) fn start_keep_types(&mut self, player: Player) -> bool {
        self.state.linked_choices.get_or_insert_default().keep_types = Some(KeepTypes {
            player,
            next_type: 0,
            selected: Vec::new(),
        });
        self.advance_keep_types()
    }
    fn keep_type_candidates(&self, choice: &KeepTypes) -> Vec<ObjectRef> {
        let kind = PermanentType::ALL[choice.next_type];
        self.state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == choice.player
                    && self.current_kinds(*id as CardId) & kind.mask() != 0
            })
            .map(|(id, _)| self.object_ref(id as CardId))
            .collect()
    }
    fn advance_keep_types(&mut self) -> bool {
        loop {
            let choice = self
                .state
                .linked_choices
                .as_ref()
                .unwrap()
                .keep_types
                .as_ref()
                .unwrap();
            if choice.next_type == PermanentType::ALL.len() {
                break;
            }
            if !self.keep_type_candidates(choice).is_empty() {
                self.state.pending = Pending::LinkedChoice {
                    player: choice.player,
                };
                return true;
            }
            self.state
                .linked_choices
                .as_mut()
                .unwrap()
                .keep_types
                .as_mut()
                .unwrap()
                .next_type += 1;
        }
        let choice = self
            .state
            .linked_choices
            .as_mut()
            .unwrap()
            .keep_types
            .take()
            .unwrap();
        let sacrifices: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == choice.player
                    && !choice
                        .selected
                        .iter()
                        .any(|(_, saved)| *saved == self.object_ref(*id as CardId))
            })
            .map(|(id, _)| (choice.player, id as CardId))
            .collect();
        if !sacrifices.is_empty() {
            self.sacrifice_batch(&sacrifices);
        }
        false
    }
    pub(super) fn choose_kept_type(&mut self, object: ObjectRef) {
        let choice = self
            .state
            .linked_choices
            .as_mut()
            .unwrap()
            .keep_types
            .as_mut()
            .unwrap();
        choice
            .selected
            .push((PermanentType::ALL[choice.next_type], object));
        choice.next_type += 1;
        if !self.advance_keep_types() {
            self.run_program();
        }
    }
    pub(super) fn keep_types_view(&self) -> Option<KeepTypesView> {
        let choice = self.state.linked_choices.as_ref()?.keep_types.as_ref()?;
        Some(KeepTypesView {
            player: choice.player,
            permanent_type: PermanentType::ALL[choice.next_type],
            selected: choice.selected.clone(),
            candidates: self.keep_type_candidates(choice),
        })
    }
    pub(super) fn visit_linked_choices(&self, mut emit: impl FnMut(Action) -> bool) -> bool {
        let data = self.state.linked_choices.as_ref().unwrap();
        if let Some(piles) = &data.piles {
            if piles.choosing {
                return [LibraryPile::FaceDown, LibraryPile::FaceUp]
                    .into_iter()
                    .any(|pile| emit(Action::ChooseLibraryPile { pile }));
            }
            return piles.remaining.iter().any(|&object| {
                [LibraryPile::FaceDown, LibraryPile::FaceUp]
                    .into_iter()
                    .any(|pile| emit(Action::AssignLibraryPile { object, pile }))
            });
        }
        let choice = data.keep_types.as_ref().unwrap();
        self.keep_type_candidates(choice)
            .into_iter()
            .any(|object| emit(Action::KeepPermanentType { object }))
    }
    pub(super) fn remember_own_sacrifice(&mut self) {
        self.state.linked_choices.get_or_insert_default().sacrifice = Some(None);
    }
    pub(super) fn record_own_sacrifices(&mut self, events: &[ZoneChange]) {
        let Some(controller) = self.state.resolving.as_ref().map(|r| r.item.controller) else {
            return;
        };
        let Some(record) = self
            .state
            .linked_choices
            .as_mut()
            .and_then(|c| c.sacrifice.as_mut())
        else {
            return;
        };
        for e in events {
            if e.moved && e.sacrificed_by == Some(controller) {
                *record = Some(e.after);
            }
        }
    }
    pub(super) fn recover_another_permanent(&mut self, player: Player) -> bool {
        let Some(Some(excluded)) = self
            .state
            .linked_choices
            .as_mut()
            .and_then(|s| s.sacrifice.take())
        else {
            return false;
        };
        let candidates: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, o)| {
                o.zone == Zone::Graveyard
                    && o.owner == player
                    && !o.is_token()
                    && crate::RecoveryFilter::Permanent.matches(o.card)
                    && self.object_ref(*id as CardId) != excluded
            })
            .map(|(id, _)| self.object_ref(id as CardId))
            .collect();
        if candidates.is_empty() {
            return false;
        }
        self.state.resolving.as_mut().unwrap().milled = candidates;
        self.state.pending = Pending::RecoverMilled {
            player,
            filter: RecoveryFilter::Permanent,
        };
        true
    }
    pub(super) fn remember_linked_exile(&mut self, selected: &[CardId]) {
        let source = self.state.resolving.as_ref().unwrap().item.object;
        for &id in selected {
            if self.state.objects[id as usize].zone == Zone::Exile {
                let exiled = self.object_ref(id);
                self.state
                    .linked_choices
                    .get_or_insert_default()
                    .links
                    .push(LinkedExileView { source, exiled });
            }
        }
    }
    pub(super) fn return_linked_exile(&mut self, source: ObjectRef) {
        let Some(data) = self.state.linked_choices.as_ref() else {
            return;
        };
        let ids: Vec<_> = data
            .links
            .iter()
            .filter(|link| {
                link.source == source
                    && self.object_ref(link.exiled.card) == link.exiled
                    && self.state.objects[link.exiled.card as usize].zone == Zone::Exile
            })
            .map(|link| link.exiled.card)
            .collect();
        let moves = ids
            .iter()
            .map(|&id| self.entry_move(id, Zone::Hand, false))
            .collect();
        self.stage_entries(moves, None);
    }
    pub(super) fn prune_linked_exiles(&mut self) {
        let Some(data) = self.state.linked_choices.as_mut() else {
            return;
        };
        let objects = &self.state.objects;
        data.links.retain(|link| {
            let o = &objects[link.exiled.card as usize];
            o.zone == Zone::Exile && o.generation == link.exiled.generation
        });
    }
}
