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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NullCastLink {
    pub source: ObjectRef,
    pub exiled: ObjectRef,
    pub ability_origin: Option<u64>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ChoiceState {
    pub hand: Option<Box<super::hand_choices::HandInteraction>>,
    pub hand_library_search: Option<Box<super::hand_library_search::HandLibrarySearch>>,
    pub zone_search: Option<Box<super::zone_search::ZoneSearch>>,
    pub library_cast: Option<Box<super::library_cast_look::LibraryCastLook>>,
    pub deployment: Option<Box<super::library_hand_exchange::HandDeployment>>,
    piles: Option<Piles>,
    keep_types: Option<KeepTypes>,
    // Some(None) means tracking this instruction but no successful own sacrifice.
    sacrifice: Option<Option<ObjectRef>>,
    pub links: Vec<LinkedExileView>,
    pub null_cast_links: Vec<NullCastLink>,
    pub hidden: Vec<(ObjectRef, ObjectRef, u8)>,
    pub hideaway_controllers: Vec<(ObjectRef, u8)>,
}
impl ChoiceState {
    // Suspended interactions may own packets; the remaining bookkeeping owns only references.
    #[cfg(test)]
    pub(super) fn has_opaque_interaction(&self) -> bool {
        self.hand.is_some()
            || self.hand_library_search.is_some()
            || self.zone_search.is_some()
            || self.library_cast.is_some()
            || self.deployment.is_some()
    }
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.hand_library_search.as_ref().map_or(0, |_| {
                std::mem::size_of::<super::hand_library_search::HandLibrarySearch>()
            })
            + self
                .zone_search
                .as_ref()
                .map_or(0, |_| std::mem::size_of::<super::zone_search::ZoneSearch>())
            + self.hand.as_ref().map_or(0, |h| h.bytes())
            + self.library_cast.as_ref().map_or(0, |h| h.bytes())
            + self.deployment.as_ref().map_or(0, |h| h.bytes())
            + self.hideaway_controllers.len() * std::mem::size_of::<(ObjectRef, u8)>()
            + self.hidden.len() * std::mem::size_of::<(ObjectRef, ObjectRef, u8)>()
            + self.links.len() * std::mem::size_of::<LinkedExileView>()
            + self.null_cast_links.len() * std::mem::size_of::<NullCastLink>()
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
            data.hand = None;
            data.zone_search = None;
            data.hand_library_search = None;
            data.library_cast = None;
            data.deployment = None;
            data.piles = None;
            data.keep_types = None;
            data.sacrifice = None;
            if data.links.is_empty()
                && data.null_cast_links.is_empty()
                && data.hideaway_controllers.is_empty()
                && data.hidden.is_empty()
            {
                self.state.linked_choices = None;
            }
        }
    }
    pub(super) fn start_library_piles(&mut self, player: Player, maximum: u16) {
        let remaining = self.state.players[player.index()]
            .library
            .iter()
            .rev()
            .filter(|&&id| self.state.objects[id as usize].is_card())
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
        if !self.shares_information(viewer, p.controller) && !p.choosing {
            return None;
        }
        let views =
            |objects: &[ObjectRef]| objects.iter().map(|o| self.object_view(o.card)).collect();
        Some(LibraryPilesView {
            controller: p.controller,
            choosing: p.choosing,
            remaining: if self.shares_information(viewer, p.controller) {
                views(&p.remaining)
            } else {
                Vec::new()
            },
            face_down: (self.shares_information(viewer, p.controller)).then(|| views(&p.down)),
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
                    && o.is_card()
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
    pub(super) fn reveal_hideaway_object(&mut self, object: ObjectRef) {
        if let Some(data) = self.state.linked_choices.as_mut() {
            data.hidden.retain(|&(_, exiled, _)| exiled != object);
        }
    }
    pub(super) fn can_view_hideaway(&self, viewer: Player, object: ObjectRef) -> bool {
        self.state.linked_choices.as_ref().is_none_or(|data| {
            data.hidden.iter().all(|&(_, exiled, viewers)| {
                exiled != object || viewers & self.information_mask(viewer) != 0
            })
        })
    }
    pub(super) fn remember_hideaway_controller(&mut self, source: ObjectRef, player: Player) {
        let has_hideaway = self.state.objects[source.card as usize].card.triggers().iter().any(|ability| ability.program.iter().any(|instruction| matches!(instruction, Instruction::SelectLibraryTo(effect) if effect.destination == crate::cards::LibraryPickDestination::Hideaway)));
        if !has_hideaway {
            return;
        }
        let data = self.state.linked_choices.get_or_insert_default();
        if let Some((_, viewers)) = data
            .hideaway_controllers
            .iter_mut()
            .find(|(r, _)| *r == source)
        {
            *viewers |= 1 << player.index();
        } else {
            data.hideaway_controllers
                .push((source, 1 << player.index()));
        }
        for (linked, _, viewers) in &mut data.hidden {
            if *linked == source {
                *viewers |= 1 << player.index();
            }
        }
    }
    pub(super) fn play_hideaway_if_coven(&mut self, item: &SpellView) {
        let Some(source) = item.source else {
            return;
        };
        let mut powers: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(id, o)| {
                o.zone == Zone::Battlefield
                    && o.controller == item.controller
                    && self.is_creature(*id as CardId)
            })
            .map(|(id, _)| self.stats(id as CardId).power)
            .collect();
        powers.sort_unstable();
        powers.dedup();
        if powers.len() < 3 {
            return;
        }
        let remaining: Vec<_> = self
            .state
            .linked_choices
            .as_ref()
            .map_or_else(Vec::new, |data| {
                data.links
                    .iter()
                    .filter(|link| {
                        link.source == source.object
                            && self.object_ref(link.exiled.card) == link.exiled
                            && self.state.objects[link.exiled.card as usize].zone == Zone::Exile
                    })
                    .map(|link| link.exiled)
                    .collect()
            });
        if remaining.is_empty() {
            return;
        }
        self.state.resolution_free_casts = Some(Box::new(ResolutionFreeCasts {
            siege_transformed: false,
            paid: None,
            casts_remaining: None,
            pending_cast: None,
            remaining_mana_value: None,
            alternative_cost: None,
            maximum_mana_value: None,
            origin: Zone::Exile,
            exile_on_graveyard: false,
            player: item.controller,
            remaining,
            allow_lands: true,
        }));
        self.state.pending = Pending::ResolutionCasts {
            player: item.controller,
        };
    }
    pub(super) fn remember_linked_exile(&mut self, selected: &[CardId]) {
        let Some(source) = self
            .state
            .resolving
            .as_ref()
            .and_then(|r| r.item.source)
            .map(|s| s.object)
        else {
            return;
        };
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
    pub(super) fn random_linked_exile_to_battlefield(&mut self, source: ObjectRef, player: Player) {
        let candidates: Vec<_> = self
            .state
            .linked_choices
            .as_ref()
            .map_or_else(Vec::new, |data| {
                data.links
                    .iter()
                    .filter(|link| {
                        link.source == source
                            && self.object_ref(link.exiled.card) == link.exiled
                            && self.state.objects[link.exiled.card as usize].zone == Zone::Exile
                    })
                    .map(|link| link.exiled.card)
                    .collect()
            });
        if candidates.is_empty() {
            return;
        }
        let mut shuffled = candidates;
        self.state.rng.shuffle(&mut shuffled);
        let mut movement = self.entry_move(shuffled[0], Zone::Battlefield, true);
        movement.controller = player;
        self.stage_entries(vec![movement], None);
    }
    pub(super) fn linked_exile_candidates(&self, source: ObjectRef) -> Vec<ObjectRef> {
        self.state
            .linked_choices
            .as_ref()
            .map_or_else(Vec::new, |data| {
                data.links
                    .iter()
                    .filter(|link| {
                        link.source == source
                            && self.object_ref(link.exiled.card) == link.exiled
                            && self.state.objects[link.exiled.card as usize].zone == Zone::Exile
                    })
                    .map(|link| link.exiled)
                    .collect()
            })
    }
    pub(super) fn choose_linked_exile(&mut self, object: ObjectRef) {
        let Some(source) = self
            .state
            .resolving
            .as_ref()
            .and_then(|r| r.item.source)
            .map(|s| s.object)
        else {
            return;
        };
        assert!(self.linked_exile_candidates(source).contains(&object));
        let movement = self.entry_move(object.card, Zone::Hand, false);
        self.stage_entries(vec![movement], None);
        self.run_program();
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
        let waiting = |source| {
            self.state
                .stack
                .iter()
                .chain(&self.state.pending_triggers)
                .chain(&self.state.ordering_triggers)
                .any(|item| {
                    item.source
                        .is_some_and(|identity| identity.object == source)
                })
                || self.state.resolving.as_ref().is_some_and(|r| {
                    r.item
                        .source
                        .is_some_and(|identity| identity.object == source)
                })
                || self.state.announcing.as_ref().is_some_and(|a| {
                    a.item
                        .source
                        .is_some_and(|identity| identity.object == source)
                })
        };
        data.hideaway_controllers.retain(|&(source, _)| {
            let o = &objects[source.card as usize];
            (o.zone == Zone::Battlefield && o.generation == source.generation) || waiting(source)
        });
        data.hidden.retain(|&(_, exiled, _)| {
            let o = &objects[exiled.card as usize];
            o.zone == Zone::Exile && o.generation == exiled.generation
        });
        data.null_cast_links.retain(|link| {
            let o = &objects[link.exiled.card as usize];
            matches!(o.zone, Zone::Exile | Zone::Stack) && o.generation == link.exiled.generation
        });
        data.links.retain(|link| {
            let o = &objects[link.exiled.card as usize];
            o.zone == Zone::Exile && o.generation == link.exiled.generation
        });
    }
}
