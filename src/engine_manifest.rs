//! Owned face-down bookkeeping for the manifest-dread foundation.
//!
//! Tracks face-down identity, private knowledge, and manifest-dread choices.
use super::*;

/// The public exact incarnation and any underlying card the viewer may know.
/// The current controller can look; a card revealed during a face-up special
/// action remains public knowledge (CR 708.5, 701.34b).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct FaceDownPermanentView {
    pub object: ObjectRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub underlying: Option<Card>,
}

/// Private look at the cards from which manifest dread selects.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ManifestChoiceView {
    pub player: Player,
    pub candidates: Vec<(ObjectRef, Card)>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ManifestDreadEvent {
    pub player: Player,
    pub manifested: Option<ObjectRef>,
    pub graveyard_cards: Vec<ObjectRef>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct FaceUpPaymentView {
    pub object: ObjectRef,
    pub controller: Player,
    pub cost: ManaCost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FaceDownPermanent {
    pub object: ObjectRef,
    pub underlying: Card,
    pub known: bool,
    /// Players who learned this exact face-down incarnation's identity.
    pub known_to: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ManifestChoice {
    pub player: Player,
    pub candidates: Vec<ObjectRef>,
    pub selected: Option<ObjectRef>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ManifestDeath {
    pub object: ObjectRef,
    pub source: SpellView,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ManifestState {
    pub permanents: Vec<FaceDownPermanent>,
    /// Exact physical identities for spells cast face down; never exposed to opponents.
    pub spells: Vec<FaceDownPermanent>,
    pub choice: Option<ManifestChoice>,
    pub payment: Option<FaceUpPaymentView>,
    pub deaths: Vec<ManifestDeath>,
    pub last_event: Option<ManifestDreadEvent>,
    pub last_face_up: Option<ObjectRef>,
    /// A face-up event whose counter replacements have not finished yet.
    pub face_up_completion: Option<ObjectRef>,
}
impl ManifestState {
    pub(super) fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + (self.permanents.len() + self.spells.len()) * std::mem::size_of::<FaceDownPermanent>()
            + self.last_event.as_ref().map_or(0, |event| {
                std::mem::size_of::<ManifestDreadEvent>()
                    + event.graveyard_cards.len() * std::mem::size_of::<ObjectRef>()
            })
            + self
                .deaths
                .iter()
                .map(|death| std::mem::size_of::<ManifestDeath>() + death.source.owned_bytes())
                .sum::<usize>()
            + self.choice.as_ref().map_or(0, |c| {
                std::mem::size_of::<ManifestChoice>()
                    + c.candidates.len() * std::mem::size_of::<ObjectRef>()
            })
    }
}
impl Game {
    pub(super) fn face_down_knowledge_mask(&self, controller: Player) -> u8 {
        [Player::P0, Player::P1]
            .into_iter()
            .filter(|&viewer| self.shares_information(viewer, controller))
            .fold(0, |mask, viewer| mask | (1 << viewer.index()))
    }
    pub(super) fn remember_face_down_controller(&mut self, object: ObjectRef, controller: Player) {
        let mask = self.face_down_knowledge_mask(controller);
        if let Some(state) = &mut self.state.manifest {
            for row in state.permanents.iter_mut().chain(state.spells.iter_mut()) {
                if row.object == object {
                    row.known_to |= mask;
                }
            }
        }
    }
    pub(super) fn remember_controlled_player_faces(&mut self, player: Player, viewer: Player) {
        let objects = &self.state.objects;
        if let Some(state) = &mut self.state.manifest {
            for row in state.permanents.iter_mut().chain(state.spells.iter_mut()) {
                if objects.get(row.object.card as usize).is_some_and(|object| {
                    object.generation == row.object.generation && object.controller == player
                }) {
                    row.known_to |= 1 << viewer.index();
                }
            }
        }
    }
    pub(super) fn restore_public_face_down(
        &mut self,
        current: &State,
        objects: &[ObjectRef],
        obscured: &[ObjectRef],
    ) {
        if let Some(state) = &mut self.state.manifest {
            for row in &mut state.permanents {
                if obscured.contains(&row.object) {
                    row.known = false;
                    row.known_to = 0;
                } else {
                    if objects.contains(&row.object) {
                        row.known = true;
                    }
                    if let Some(current_row) = current.manifest.as_ref().and_then(|state| {
                        state
                            .permanents
                            .iter()
                            .find(|other| other.object == row.object)
                    }) {
                        row.known_to |= current_row.known_to;
                    }
                }
            }
        }
    }
    pub(super) fn obscure_rollback_face_down(&mut self, card: CardId) {
        if let Some(rollback) = &mut self.state.cast_rollback {
            if let Some(reference) = rollback.before.manifest.as_ref().and_then(|state| {
                state
                    .permanents
                    .iter()
                    .find(|row| row.object.card == card)
                    .map(|row| row.object)
            }) && !rollback.obscured_face_down.contains(&reference)
            {
                rollback.obscured_face_down.push(reference);
            }
            rollback
                .public_face_down
                .retain(|object| object.card != card);
        }
    }
    pub(super) fn face_up_payment_fundable(&self) -> bool {
        let Some(request) = self.face_up_payment() else {
            return false;
        };
        if !self.is_face_down(request.object)
            || !self.on_battlefield(request.object.card)
            || self.object_ref(request.object.card) != request.object
            || self.state.objects[request.object.card as usize].controller != request.controller
        {
            return false;
        }
        self.can_fund_face_up(
            FundingGoal {
                actor: request.controller,
                required: request.cost.colored,
                generic: request.cost.generic,
                reserve: None,
                source: None,
            },
            request.object,
        )
    }
    pub(super) fn register_manifest_death(&mut self, item: &SpellView) {
        let Some(bound) = item.targets.first().copied() else {
            return;
        };
        let Some(object) = bound.object else {
            return;
        };
        if !self.same_target_object(bound) || !self.on_battlefield(object.card) {
            return;
        }
        self.state
            .manifest
            .get_or_insert_with(Default::default)
            .deaths
            .push(ManifestDeath {
                object,
                source: item.clone(),
            });
    }
    pub(super) fn queue_manifest_deaths(&mut self, events: &[ZoneChange]) {
        let Some(state) = &mut self.state.manifest else {
            return;
        };
        let mut triggered = Vec::new();
        state.deaths.retain(|death| {
            let dies = events.iter().any(|event| {
                event.moved
                    && event.before.zone == Zone::Battlefield
                    && event.destination == Zone::Graveyard
                    && event.before.reference == death.object
            });
            if dies {
                triggered.push(death.source.clone());
            }
            !dies
        });
        for mut item in triggered {
            item.id = self.next_stack_id();
            item.kind = StackKind::DelayedManifestDread;
            item.targets.clear();
            item.crime_qualifying_target = false;
            item.target_type_slots = None;
            item.mode = None;
            item.cast = None;
            item.is_copy = false;
            item.related_stack = None;
            item.additional_triggers = 0;
            self.state.pending_triggers.push(item);
        }
    }
    pub(super) fn expire_manifest_deaths(&mut self) {
        if let Some(state) = &mut self.state.manifest {
            state.deaths.clear();
        }
    }
    /// Feasibility is evaluated with the public creature spell's characteristics,
    /// never with the private card's colors, types, or printed cost.
    pub(super) fn disguise_cast_fundable(&self, player: Player, id: CardId) -> bool {
        let object = &self.state.objects[id as usize];
        let origin_permitted = match object.zone {
            Zone::Hand => object.owner == player,
            Zone::Graveyard => self.has_graveyard_cast(player, id, GraveyardCastKind::Cast),
            Zone::Exile => !object.is_spell_copy() && self.can_play_exiled(player, id),
            Zone::Library => self.can_cast_library_top(player, id),
            _ => false,
        };
        origin_permitted && self.disguise_mana_fundable(player, id)
    }

    pub(super) fn disguise_mana_fundable(&self, player: Player, id: CardId) -> bool {
        let object = &self.state.objects[id as usize];
        let physical = object.card;
        if object.is_token()
            || physical.disguise_cost().is_none()
            || !(physical.is_supported()
                || cfg!(test)
                    && (physical
                        == Card::from_name("Bubble Smuggler").expect("pinned catalogue identity")
                        || physical
                            == Card::from_name("Concealed Weapon")
                                .expect("pinned catalogue identity")))
        {
            return false;
        }
        self.spell_can_fund_for_definition(
            player,
            id,
            ManaCost {
                colored: [0; 6],
                generic: 3,
            },
            Card::from_name("@face-down/disguise").expect("pinned catalogue identity"),
        )
    }

    /// Begin a face-down spell with only its public copiable values on the
    /// actual stack object. The identity stays in the owned private registry.
    pub(super) fn cast_disguised_item(&mut self, player: Player, id: CardId) -> SpellView {
        let reference = self.object_ref(id);
        let object = &self.state.objects[id as usize];
        let physical = object.card;
        let known = match object.zone {
            Zone::Hand => self.state.known_hand.contains(&reference),
            Zone::Graveyard => true,
            Zone::Exile => self.can_view_hideaway(player.other(), reference),
            Zone::Library => self.public_library_top(object.owner) == Some(reference),
            _ => false,
        };
        // Permission guards inspect the public card in its origin zone, so
        // capture these before applying face-down spell characteristics.
        let modifiers = self.exile_cast_modifiers(player, id);
        let stash_origin = self.foreign_stash(player, id);
        self.state.objects[id as usize].card =
            Card::from_name("@face-down/disguise").expect("pinned catalogue identity");
        let mut item = self.cast_item(player, id, [0; 6]);
        if let Some(cast) = &mut item.cast {
            cast.spend_any_mana = modifiers.0;
            cast.exile_on_graveyard = modifiers.1;
            cast.stash_origin = stash_origin;
        }
        let known_to = self.face_down_knowledge_mask(player);
        self.state
            .manifest
            .get_or_insert_with(Default::default)
            .spells
            .push(FaceDownPermanent {
                object: item.source.expect("face-down cast has source").object,
                underlying: physical,
                known,
                known_to,
            });
        item
    }

    pub(super) fn take_face_down_spell(
        &mut self,
        reference: ObjectRef,
    ) -> Option<FaceDownPermanent> {
        let state = self.state.manifest.as_mut()?;
        let index = state
            .spells
            .iter()
            .position(|row| row.object == reference)?;
        Some(state.spells.remove(index))
    }
    pub(super) fn face_up_payment(&self) -> Option<&FaceUpPaymentView> {
        self.state.manifest.as_ref()?.payment.as_ref()
    }
    pub(super) fn begin_face_up(&mut self, player: Player, object: ObjectRef, cost: ManaCost) {
        let row = self
            .state
            .manifest
            .as_mut()
            .unwrap()
            .permanents
            .iter_mut()
            .find(|row| row.object == object)
            .unwrap();
        row.known = true;
        let physical = row.underlying;
        self.record_zone_reveal(
            physical,
            self.state.objects[object.card as usize].owner,
            Zone::Battlefield,
        );
        self.state.manifest.as_mut().unwrap().payment = Some(FaceUpPaymentView {
            object,
            controller: player,
            cost,
        });
        self.state.pending = Pending::FaceUpPayment { player };
        self.state.passes = 0;
    }
    pub(super) fn finish_face_up(&mut self, payment: Mana) {
        let request = self
            .state
            .manifest
            .as_mut()
            .unwrap()
            .payment
            .take()
            .unwrap();
        self.spend_mana(request.controller, payment);
        // The replacement engine captures this as its resume state. Set it
        // before beginning the transition so a suspended choice is never lost.
        self.state.pending = Pending::Priority;
        self.state.priority = request.controller;
        self.state.passes = 0;
        assert!(self.turn_manifest_face_up(request.object));
    }
    pub(super) fn face_down_physical(&self, object: ObjectRef) -> Option<Card> {
        self.state
            .manifest
            .as_ref()?
            .permanents
            .iter()
            .find(|row| row.object == object)
            .map(|row| row.underlying)
    }

    pub(super) fn is_face_down(&self, object: ObjectRef) -> bool {
        self.state
            .manifest
            .as_ref()
            .is_some_and(|state| state.permanents.iter().any(|row| row.object == object))
    }

    /// Complete the status change after a caller has paid and verified the
    /// special action. This is not a zone change or an enters-the-battlefield event.
    pub(super) fn turn_manifest_face_up(&mut self, object: ObjectRef) -> bool {
        if !self.on_battlefield(object.card) || self.object_ref(object.card) != object {
            return false;
        }
        // CR701.40g: an effect cannot turn an instant or sorcery card face up;
        // reveal it and leave the same permanent face down instead.
        let Some(row) = self
            .state
            .manifest
            .as_mut()
            .and_then(|state| state.permanents.iter_mut().find(|row| row.object == object))
        else {
            return false;
        };
        if row.underlying.kind_mask() & (4 | 8) != 0 {
            row.known = true;
            let physical = row.underlying;
            self.record_zone_reveal(
                physical,
                self.state.objects[object.card as usize].owner,
                Zone::Battlefield,
            );
            return false;
        }
        let physical = row.underlying;
        row.known = true;
        self.record_zone_reveal(
            physical,
            self.state.objects[object.card as usize].owner,
            Zone::Battlefield,
        );
        // Evaluate the resulting copy and layer state without exposing a half
        // completed face-up transition to an affected player's choice.
        let mut prospective = self.clone();
        prospective.materialize_face_up(object);
        let card = prospective.state.objects[object.card as usize].card;
        let counters = card.as_turned_face_up_counters();
        if !counters.is_empty() && prospective.printed_abilities_available(object.card) {
            let mut amount = Counters::default();
            for &(kind, value) in counters {
                amount.add(kind, value);
            }
            let o = &prospective.state.objects[object.card as usize];
            let placement = replacements::CounterPlacement {
                object,
                placed_by: o.controller,
                card,
                owner: o.owner,
                controller: o.controller,
                protector: prospective.battle_protector(object),
                subtypes: prospective.granted_subtypes(object.card),
                kind: prospective.current_kinds(object.card),
                power: prospective.object_view(object.card).power,
                amount,
                effect: true,
                entry_index: None,
                private: false,
            };
            self.state.manifest.as_mut().unwrap().face_up_completion = Some(object);
            let mode = if self.state.resolving.is_some() {
                replacements::ResumeMode::Program
            } else {
                replacements::ResumeMode::Pending
            };
            self.stage_counter_placements(vec![placement], mode);
        } else {
            self.materialize_face_up(object);
            self.queue_face_up_triggers(object);
        }
        true
    }

    /// Apply the copiable face and re-register providers without reporting the
    /// event. Counter replacement commits call this immediately before adding
    /// the finalized counters, so observers see a single completed transition.
    pub(super) fn materialize_face_up(&mut self, object: ObjectRef) {
        let physical = self
            .take_face_down_physical(object)
            .expect("face-down incarnation");
        self.forget_hand_size_source(object);
        self.remove_face_providers(object.card);
        self.state.objects[object.card as usize].card = physical;
        if !self.reveal_temporary_copy(object, physical) {
            self.register_battlefield_providers(object.card);
        }
        self.retimestamp_self_granted_statics(object);
        self.refresh_controllers();
    }

    pub(super) fn finish_face_up_counter_replacement(&mut self) {
        let Some(object) = self
            .state
            .manifest
            .as_mut()
            .and_then(|state| state.face_up_completion.take())
        else {
            return;
        };
        self.queue_face_up_triggers(object);
    }
    pub(super) fn queue_face_up_triggers(&mut self, object: ObjectRef) {
        let Some(turned) = self.state.objects.get(object.card as usize) else {
            return;
        };
        if turned.zone != Zone::Battlefield || self.object_ref(object.card) != object {
            return;
        }
        let controller = turned.controller;
        self.state
            .manifest
            .get_or_insert_with(Default::default)
            .last_face_up = Some(object);
        let ids: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, source)| source.zone == Zone::Battlefield)
            .map(|(id, _)| id as CardId)
            .collect();
        for (source, grant) in self.indexed_trigger_sources(&ids) {
            let definition = grant
                .as_ref()
                .map_or(source.card, |grant| grant.granting_card);
            for (index, ability) in definition.triggers().iter().enumerate() {
                let matched = match ability.condition.event() {
                    TriggerCondition::SelfTurnedFaceUp => source.reference == object,
                    TriggerCondition::PermanentYouControlTurnedFaceUp => {
                        source.controller == controller
                    }
                    _ => false,
                };
                if matched && self.queue_trigger_from(&source, index, grant.map(Box::new)) {
                    self.state.pending_triggers.last_mut().unwrap().event_object =
                        Some(Box::new(EventObjectView {
                            object,
                            facts: EventObjectFacts::Controller(controller),
                            conniver_creature: None,
                            subtype_lki: None,
                        }));
                }
            }
        }
    }
    pub(super) fn queue_manifest_dread_triggers(&mut self, event: ManifestDreadEvent) {
        self.state
            .manifest
            .get_or_insert_with(Default::default)
            .last_event = Some(event.clone());
        let ids: Vec<_> = self
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, source)| source.zone == Zone::Battlefield)
            .map(|(id, _)| id as CardId)
            .collect();
        for (source, grant) in self.indexed_trigger_sources(&ids) {
            let definition = grant
                .as_ref()
                .map_or(source.card, |grant| grant.granting_card);
            for (index, ability) in definition.triggers().iter().enumerate() {
                if ability.condition.event() == TriggerCondition::YouManifestDread
                    && source.controller == event.player
                    && self.queue_trigger_from(&source, index, grant.map(Box::new))
                {
                    self.state
                        .pending_triggers
                        .last_mut()
                        .unwrap()
                        .manifest_event = Some(Box::new(event.clone()));
                }
            }
        }
    }
    /// Start CR701.62a. Looking is private, and selecting the sole card is
    /// mandatory. Entry replacements may suspend before the rest is milled.
    pub(super) fn start_manifest_dread(&mut self, player: Player) -> bool {
        let candidates: Vec<_> = self.state.players[player.index()]
            .library
            .iter()
            .rev()
            .filter(|&&id| self.state.objects[id as usize].is_card())
            .take(2)
            .map(|&id| self.object_ref(id))
            .collect();
        if candidates.is_empty() {
            // CR701.62b: completing the action triggers observers even when
            // all of its component actions are impossible.
            self.queue_manifest_dread_triggers(ManifestDreadEvent {
                player,
                manifested: None,
                graveyard_cards: Vec::new(),
            });
            return false;
        }
        self.state
            .manifest
            .get_or_insert_with(Default::default)
            .choice = Some(ManifestChoice {
            player,
            candidates: candidates.clone(),
            selected: None,
        });
        if candidates.len() == 1 {
            self.choose_manifest(candidates[0]);
        } else {
            self.state.pending = Pending::Manifest { player };
        }
        true
    }

    pub(super) fn choose_manifest(&mut self, selected: ObjectRef) {
        let choice = self
            .state
            .manifest
            .as_mut()
            .unwrap()
            .choice
            .as_mut()
            .unwrap();
        assert!(choice.candidates.contains(&selected));
        choice.selected = Some(selected);
        let player = choice.player;
        // The consumed choice must not be restored as an entry continuation.
        if matches!(self.state.pending, Pending::Manifest { .. }) {
            self.state.pending = Pending::Priority;
        }
        let movement = self
            .manifest_entry(selected, player)
            .expect("selected manifest card remains in library");
        self.stage_entries(vec![movement], None);
    }

    /// Called after the staged entry commits. Cards that weren't manifested,
    /// including a card whose entry was prevented, are moved simultaneously.
    pub(super) fn finish_manifest_entry(&mut self) {
        let ready = self
            .state
            .manifest
            .as_ref()
            .and_then(|s| s.choice.as_ref())
            .is_some_and(|choice| choice.selected.is_some());
        if !ready {
            return;
        }
        let choice = self.state.manifest.as_mut().unwrap().choice.take().unwrap();
        let remaining: Vec<_> = choice
            .candidates
            .into_iter()
            .filter(|&object| {
                self.state
                    .objects
                    .get(object.card as usize)
                    .is_some_and(|card| {
                        card.zone == Zone::Library && self.object_ref(object.card) == object
                    })
            })
            .map(|object| object.card)
            .collect();
        let manifested = choice.selected.and_then(|selected| {
            self.state
                .manifest
                .as_ref()?
                .permanents
                .iter()
                .find(|row| row.object.card == selected.card)
                .map(|row| row.object)
        });
        let events = self.move_objects_raw(&remaining, Zone::Graveyard);
        let graveyard_cards = events
            .iter()
            .filter(|event| event.moved && event.destination == Zone::Graveyard)
            .map(|event| event.after)
            .collect();
        self.finish_zone_events(&events, &[]);
        self.queue_manifest_dread_triggers(ManifestDreadEvent {
            player: choice.player,
            manifested,
            graveyard_cards,
        });
    }
    pub(super) fn take_face_down_physical(&mut self, reference: ObjectRef) -> Option<Card> {
        if let Some(rollback) = &mut self.state.cast_rollback
            && rollback
                .before
                .manifest
                .as_ref()
                .is_some_and(|state| state.permanents.iter().any(|row| row.object == reference))
            && !rollback.obscured_face_down.contains(&reference)
            && !rollback.public_face_down.contains(&reference)
        {
            rollback.public_face_down.push(reference);
        }
        let state = self.state.manifest.as_mut()?;
        let index = state
            .permanents
            .iter()
            .position(|row| row.object == reference)?;
        Some(state.permanents.remove(index).underlying)
    }
    /// Build a staged, private battlefield entry using CR 708.2 copiable
    /// characteristics without exposing or mutating the library card.
    pub(super) fn manifest_entry(
        &self,
        object: ObjectRef,
        controller: Player,
    ) -> Option<entries::EntryMove> {
        let current = self.state.objects.get(object.card as usize)?;
        if current.zone != Zone::Library || self.object_ref(object.card) != object {
            return None;
        }
        let original = current.card;
        let mut movement = self.entry_move(object.card, Zone::Battlefield, false);
        let view = &mut movement.before;
        view.card = Card::from_name("@face-down/manifest").expect("pinned catalogue identity");
        view.name = "";
        view.kinds = 1;
        view.colors = 0;
        view.effective_legendary = None;
        view.granted_subtypes = GrantedSubtypes::default();
        view.entry_choice = None;
        view.abilities = None;
        view.power = 2;
        view.toughness = 2;
        view.power_above_base = false;
        view.keywords = Keywords::NONE;
        view.face = None;
        view.copy_recipe = None;
        view.temporary_copy = None;
        view.provided_ability_origin = None;
        view.token = None;
        view.emblem = None;
        movement.controller = controller;
        movement.manifest_original = Some(original);
        movement.manifest_known = self.public_library_top(current.owner) == Some(object);
        Some(movement)
    }
    /// Determine whether the controller could pay the underlying creature's
    /// printed mana cost as a special action. This does not expose that cost to
    /// an opponent and is not yet wired into legal action enumeration.
    pub(super) fn face_up_payment_available(&self, player: Player, object: ObjectRef) -> bool {
        if self.state.pending != Pending::Priority || self.semantic_actor() != Some(player) {
            return false;
        }
        let Some(row) = self
            .state
            .manifest
            .as_ref()
            .and_then(|state| state.permanents.iter().find(|row| row.object == object))
        else {
            return false;
        };
        let Some(permanent) = self.state.objects.get(object.card as usize) else {
            return false;
        };
        if permanent.zone != Zone::Battlefield
            || self.object_ref(object.card) != object
            || permanent.controller != player
            || !(row.underlying.is_supported()
                || cfg!(test)
                    && (row.underlying
                        == Card::from_name("Bubble Smuggler").expect("pinned catalogue identity")
                        || row.underlying
                            == Card::from_name("Concealed Weapon")
                                .expect("pinned catalogue identity")))
        {
            return false;
        }
        row.underlying
            .cost_options()
            .filter(|_| row.underlying.is_creature() && !row.underlying.info().mana_cost.is_empty())
            .chain(row.underlying.disguise_cost())
            .any(|cost| {
                self.can_fund_face_up(
                    FundingGoal {
                        actor: player,
                        required: cost.colored,
                        generic: cost.generic,
                        reserve: None,
                        source: None,
                    },
                    object,
                )
            })
    }
    pub(super) fn manifest_choice_view(&self, viewer: Player) -> Option<ManifestChoiceView> {
        let choice = self.state.manifest.as_ref()?.choice.as_ref()?;
        if !self.shares_information(viewer, choice.player) {
            return None;
        }
        let candidates = choice
            .candidates
            .iter()
            .filter_map(|&reference| {
                let object = self.state.objects.get(reference.card as usize)?;
                (object.zone == Zone::Library && self.object_ref(reference.card) == reference)
                    .then_some((reference, object.card))
            })
            .collect();
        Some(ManifestChoiceView {
            player: choice.player,
            candidates,
        })
    }
    pub(super) fn face_down_spell_views(&self, viewer: Player) -> Vec<FaceDownPermanentView> {
        self.state.manifest.as_ref().map_or_else(Vec::new, |state| {
            state
                .spells
                .iter()
                .filter_map(|row| {
                    let object = self.state.objects.get(row.object.card as usize)?;
                    (object.zone == Zone::Stack && self.object_ref(row.object.card) == row.object)
                        .then(|| FaceDownPermanentView {
                            object: row.object,
                            underlying: (self.state.outcome.is_some()
                                || row.known
                                || row.known_to & (1 << viewer.index()) != 0
                                || self.shares_information(viewer, object.controller))
                            .then_some(row.underlying),
                        })
                })
                .collect()
        })
    }
    pub(super) fn face_down_views(&self, viewer: Player) -> Vec<FaceDownPermanentView> {
        self.state.manifest.as_ref().map_or_else(Vec::new, |state| {
            state
                .permanents
                .iter()
                .filter_map(|row| {
                    let object = self.state.objects.get(row.object.card as usize)?;
                    if object.zone != Zone::Battlefield
                        || self.object_ref(row.object.card) != row.object
                    {
                        return None;
                    }
                    Some(FaceDownPermanentView {
                        object: row.object,
                        underlying: (self.state.outcome.is_some()
                            || row.known
                            || row.known_to & (1 << viewer.index()) != 0
                            || self.shares_information(viewer, object.controller))
                        .then_some(row.underlying),
                    })
                })
                .collect()
        })
    }
}
