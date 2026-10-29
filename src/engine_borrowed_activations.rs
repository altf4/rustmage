//! Activated abilities inherited from exact linked exile incarnations.
use super::ability_world::CharacteristicWorld;
use super::*;
/// Acquisition identifiers are separate from public ability slots. Both are
/// allocated monotonically within an exact source incarnation.
pub(super) const DONOR_SLOT_WIDTH: u64 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct SlotRow {
    source: ObjectRef,
    acquisition: u64,
    graveyard: bool,
    pub(super) definition: Option<Card>,
    ability: u16,
    slot: u64,
}
pub(super) type SlotRows = Option<SharedVec<SlotRow>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Link {
    pub source: ObjectRef,
    pub origin: Option<u64>,
    pub exiled: ObjectRef,
    pub first_slot: u64,
    pub width: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct BorrowedAbilityView {
    pub source: ObjectRef,
    pub exiled: ObjectRef,
    pub definition: Card,
    /// The final five indices identify the exiled object's intrinsic mana abilities.
    pub ability: u16,
    pub slot: u64,
    pub used: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Use {
    pub source: ObjectRef,
    pub slot: u64,
    pub turn: u32,
    pub exhaust: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct GraveyardBorrowedAbilityView {
    pub source: ObjectRef,
    pub graveyard: ObjectRef,
    pub definition: Card,
    pub ability: u16,
    pub slot: u64,
    pub used: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Descriptor {
    pub source: ObjectRef,
    pub donor: ObjectRef,
    pub zone: Zone,
    pub definition: Card,
    pub ability: u16,
    pub slot: u64,
    pub used: bool,
    pub(super) acquisition: u64,
}
impl Descriptor {
    pub(super) fn rules(self) -> &'static ActivatedAbility {
        self.definition.activation(self.ability)
    }
}
pub(super) fn abilities<W: CharacteristicWorld + ?Sized>(world: &W, id: CardId) -> Vec<Descriptor> {
    inventory(world, id, true)
}
fn inventory<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
    check_available: bool,
) -> Vec<Descriptor> {
    let source = world.object(id);
    if source.zone != Zone::Battlefield
        || !source.card.borrows_activations()
        || check_available && !world.printed_abilities_available(id)
    {
        return Vec::new();
    }
    let mut result = Vec::new();
    for link in world
        .game()
        .state
        .borrowed_links
        .as_ref()
        .map_or(&[][..], |rows| rows.as_slice())
    {
        if !source.card.borrows_exiled_activations()
            || link.source != world.reference(id)
            || link.origin != world.ability_origin(id)
        {
            continue;
        }
        let card = world.object(link.exiled.card);
        if card.zone != Zone::Exile || world.reference(link.exiled.card) != link.exiled {
            continue;
        }
        for (index, _) in card.card.activations().iter().enumerate() {
            if card.card.legacy_subtype_mana_activation(index as u16) {
                continue;
            }
            if card.card.grants_activation(index as u16)
                || card.card.self_granted_activation_template(index as u16)
            {
                continue;
            }
            let slot = 0;
            result.push(Descriptor {
                source: link.source,
                donor: link.exiled,
                zone: Zone::Exile,
                definition: card.card,
                ability: index as u16,
                slot,
                used: false,
                acquisition: link.first_slot,
            });
        }
        if world.kinds(link.exiled.card) & 2 != 0 {
            let subtypes = world.subtypes(link.exiled.card);
            for (color, subtype) in ["Plains", "Island", "Swamp", "Mountain", "Forest"]
                .iter()
                .enumerate()
            {
                if !subtypes.has(card.card, subtype) {
                    continue;
                }
                let slot = 0;
                result.push(Descriptor {
                    source: link.source,
                    donor: link.exiled,
                    zone: Zone::Exile,
                    definition: card.card,
                    ability: u16::MAX - 1 - color as u16,
                    slot,
                    used: false,
                    acquisition: link.first_slot,
                });
            }
        }
        if !card.card.is_land() && card.card.mana_color().is_some() {
            let slot = 0;
            result.push(Descriptor {
                source: link.source,
                donor: link.exiled,
                zone: Zone::Exile,
                definition: card.card,
                ability: u16::MAX,
                slot,
                used: false,
                acquisition: link.first_slot,
            });
        }
    }
    if source.card.graveyard_activation_subtype().is_some() {
        result.extend(graveyard_borrowing::inventory(world, id));
    }
    assign_slots(world, &mut result);
    result
}

// Existing rows are immutable. New candidate rows receive provisional compact
// slots in inventory order, the same order used when persisting the registry.
fn assign_slots<W: CharacteristicWorld + ?Sized>(world: &W, abilities: &mut [Descriptor]) {
    let mut rows = world.borrowed_slots().to_vec();
    for ability in abilities {
        let intrinsic = ability.ability >= u16::MAX - 5 && ability.ability < u16::MAX;
        let definition = (!intrinsic).then_some(ability.definition);
        let graveyard = ability.zone == Zone::Graveyard;
        let matching = |row: &&SlotRow| {
            row.source == ability.source
                && row.acquisition == ability.acquisition
                && row.graveyard == graveyard
                && row.definition == definition
                && row.ability == ability.ability
        };
        let slot = if let Some(row) = rows.iter().find(matching) {
            row.slot
        } else {
            let slot = rows
                .iter()
                .filter(|row| row.source == ability.source)
                .map(|row| row.slot)
                .max()
                .map_or(0, |slot| {
                    slot.checked_add(1)
                        .capacity("borrowed ability slots exhausted")
                });
            rows.push(SlotRow {
                source: ability.source,
                acquisition: ability.acquisition,
                graveyard,
                definition,
                ability: ability.ability,
                slot,
            });
            slot
        };
        ability.slot = slot;
        ability.used = world.borrowed_activation_used(ability.source, slot);
    }
}

pub(super) fn refreshed_slots<W: CharacteristicWorld + ?Sized>(world: &W) -> SlotRows {
    if world.borrowed_slots().is_empty()
        && world.graveyard_borrowing().is_empty()
        && world.game().state.borrowed_links.is_none()
    {
        return None;
    }
    let mut rows: Vec<_> = world
        .borrowed_slots()
        .iter()
        .copied()
        .filter(|row| {
            world.object(row.source.card).zone == Zone::Battlefield
                && world.reference(row.source.card) == row.source
        })
        .collect();
    let mut sources = world.static_sources().to_vec();
    for link in world
        .game()
        .state
        .borrowed_links
        .as_ref()
        .into_iter()
        .flat_map(|rows| rows.iter())
    {
        if !sources.contains(&link.source.card) {
            sources.push(link.source.card);
        }
    }
    for id in sources {
        for ability in inventory(world, id, false) {
            let intrinsic = ability.ability >= u16::MAX - 5 && ability.ability < u16::MAX;
            let row = SlotRow {
                source: ability.source,
                acquisition: ability.acquisition,
                graveyard: ability.zone == Zone::Graveyard,
                definition: (!intrinsic).then_some(ability.definition),
                ability: ability.ability,
                slot: ability.slot,
            };
            if !rows.iter().any(|old| {
                old.source == row.source
                    && old.acquisition == row.acquisition
                    && old.graveyard == row.graveyard
                    && old.definition == row.definition
                    && old.ability == row.ability
            }) {
                rows.push(row);
            }
        }
    }
    if rows == world.borrowed_slots() {
        return world.borrowed_slots_state().clone();
    }
    (!rows.is_empty()).then(|| rows.into())
}
impl Game {
    pub(super) fn exile_and_borrow_activated_abilities(&mut self, item: &SpellView) {
        let ids: Vec<_> = item
            .targets
            .iter()
            .filter(|bound| self.same_target_object(**bound))
            .filter_map(|bound| match bound.target {
                Target::Card(id) => Some(id),
                _ => None,
            })
            .collect();
        let events = self.move_objects_raw(&ids, Zone::Exile);
        if self.on_battlefield(item.object.card) && self.object_ref(item.object.card) == item.object
        {
            for event in &events {
                if !event.moved || event.destination != Zone::Exile {
                    continue;
                }
                let width = DONOR_SLOT_WIDTH;
                let first_slot = graveyard_borrowing::next_slot(self, item.object);
                (first_slot.checked_add(width)).capacity("borrowed activation capacity exhausted");
                self.state
                    .borrowed_links
                    .get_or_insert_default()
                    .push(Link {
                        source: item.object,
                        origin: item.source_ability_origin,
                        exiled: event.after,
                        first_slot,
                        width,
                    });
            }
        }
        self.finish_zone_events(&events, &[]);
    }
    pub(super) fn borrowed_descriptor(
        &self,
        card: CardId,
        exiled: ObjectRef,
        index: u16,
    ) -> Descriptor {
        abilities(self, card)
            .into_iter()
            .find(|a| a.donor == exiled && a.ability == index)
            .expect("legal borrowed activation")
    }
    pub(super) fn borrowed_activation_item(
        &mut self,
        actor: Player,
        card: CardId,
        exiled: ObjectRef,
        index: u16,
    ) -> SpellView {
        self.refresh_graveyard_borrowing();
        let descriptor = self.borrowed_descriptor(card, exiled, index);
        let mut item = self.activation_item(actor, card, index);
        item.source_card = descriptor.definition;
        item.source_counter_lki =
            card_quantities::counter_quantity_facts(descriptor.rules().program, |kind| {
                self.source_counter_quantity_live(item.object, kind)
                    .expect("live inherited source")
            });
        item.source_copy_recipe = Some(Box::new(self.copiable_recipe(item.object)));
        item.borrowed_slot = Some(descriptor.slot);
        item
    }
    pub(super) fn record_borrowed_use(&mut self, source: ObjectRef, slot: u64, exhaust: bool) {
        if !self.state.borrowed_uses.as_ref().is_some_and(|rows| {
            rows.iter().any(|row| {
                row.source == source
                    && row.slot == slot
                    && (row.exhaust || row.turn == self.state.turn)
            })
        }) {
            self.state.borrowed_uses.get_or_insert_default().push(Use {
                source,
                slot,
                turn: self.state.turn,
                exhaust,
            });
        }
    }
    pub(super) fn forget_borrowed_source(&mut self, source: ObjectRef) {
        graveyard_borrowing::forget(&mut self.state.graveyard_borrowing, source);
        if let Some(rows) = &mut self.state.borrowed_slots {
            rows.retain(|row| row.source != source);
            if rows.is_empty() {
                self.state.borrowed_slots = None;
            }
        }
        if let Some(rows) = &mut self.state.borrowed_links {
            rows.retain(|row| row.source != source);
            if rows.is_empty() {
                self.state.borrowed_links = None;
            }
        }
        if let Some(rows) = &mut self.state.borrowed_uses {
            rows.retain(|row| row.source != source);
            if rows.is_empty() {
                self.state.borrowed_uses = None;
            }
        }
    }
    pub(super) fn borrowed_ability_views(&self) -> Vec<BorrowedAbilityView> {
        let mut sources = Vec::new();
        for link in self
            .state
            .borrowed_links
            .as_ref()
            .into_iter()
            .flat_map(|links| links.iter())
        {
            if !sources.contains(&link.source.card) {
                sources.push(link.source.card);
            }
        }
        sources
            .into_iter()
            .flat_map(|id| abilities(self, id))
            .filter(|a| a.zone == Zone::Exile)
            .map(|a| BorrowedAbilityView {
                source: a.source,
                exiled: a.donor,
                definition: a.definition,
                ability: a.ability,
                slot: a.slot,
                used: a.used,
            })
            .collect()
    }
    pub(super) fn graveyard_borrowed_ability_views(
        &self,
    ) -> Option<Vec<GraveyardBorrowedAbilityView>> {
        let rows: Vec<_> = self
            .state
            .static_sources
            .iter()
            .copied()
            .filter(|&id| {
                self.state.objects[id as usize]
                    .card
                    .graveyard_activation_subtype()
                    .is_some()
            })
            .flat_map(|id| abilities(self, id))
            .filter(|a| a.zone == Zone::Graveyard)
            .map(|a| GraveyardBorrowedAbilityView {
                source: a.source,
                graveyard: a.donor,
                definition: a.definition,
                ability: a.ability,
                slot: a.slot,
                used: a.used,
            })
            .collect();
        (!rows.is_empty()).then_some(rows)
    }
    pub(super) fn visit_borrowed_activations(
        &self,
        card: CardId,
        actor: Player,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        if !self.state.objects[card as usize].card.borrows_activations()
            || CharacteristicWorld::activation_prohibited(self, actor, card)
        {
            return false;
        }
        let object = &self.state.objects[card as usize];
        let descriptors = abilities(self, card);
        let loyalty_used = descriptors
            .iter()
            .any(|a| a.rules().cost.loyalty.is_some() && a.used);
        let mut ready_object = object.clone();
        ready_object.used_abilities = 0;
        for descriptor in &descriptors {
            let ability = descriptor.rules();
            let readiness = self.activation_readiness(actor, ability);
            if ability.cost.loyalty.is_some() && loyalty_used {
                continue;
            }
            if !self.power_up_available(
                descriptor.source,
                descriptor.definition,
                descriptor.ability,
                Some(descriptor.slot),
            ) || descriptor.used && (ability.once_per_turn || ability.exhaust)
            {
                continue;
            }
            if ability
                .crew_minimum()
                .is_some_and(|minimum| !self.crew_fundable(descriptor.source, actor, minimum))
            {
                continue;
            }
            if ability
                .saddle_minimum()
                .is_some_and(|minimum| !self.saddle_fundable(descriptor.source, actor, minimum))
            {
                continue;
            }
            if (ability.cost.sacrifice && CharacteristicWorld::sacrifice_prohibited(self, card))
                || !self.activation_guard_available(ability.guard, card, actor)
                || (ability.mana_production().is_none()
                    && (self.nonmana_actions_prohibited()
                        || self.named_activation_prohibited(card)))
                || !Self::object_activation_available_with_kinds(
                    &ready_object,
                    self.counter_amounts(card),
                    self.current_kinds(card),
                    || CharacteristicWorld::activation_keywords(self, card),
                    self.state.players[actor.index()].life,
                    &readiness,
                    0,
                    actor,
                    sorcery,
                )
            {
                continue;
            }
            if self.visit_activation_counter_payments(
                actor,
                ability,
                self.counter_amounts(card),
                |counter_payment| {
                    let fundable = self
                        .activation_mana_costs(card, descriptor.definition, descriptor.ability, 0)
                        .any(|cost| {
                            self.initial_activation_target_fundable(
                                FundingGoal {
                                    actor,
                                    required: cost.colored,
                                    generic: cost.generic,
                                    reserve: None,
                                    source: Some((
                                        descriptor.source,
                                        descriptor.ability,
                                        counter_payment.clone(),
                                        descriptor.definition,
                                        Some(descriptor.slot),
                                    )),
                                },
                                ability.targets,
                            )
                        });
                    let modes = match ability.program.first() {
                        Some(Instruction::Modal(modes)) => Some(*modes),
                        _ => None,
                    };
                    let targets_available = modes.map_or_else(
                        || self.targets_available(ability.targets, actor, descriptor.source),
                        |modes| {
                            modes.iter().any(|mode| {
                                self.targets_available(mode.targets, actor, descriptor.source)
                            })
                        },
                    );
                    if !fundable || !targets_available {
                        return false;
                    }
                    if let Some(production) = ability.mana_production() {
                        production.in_world(self, card).visit(|production| {
                            emit(if descriptor.zone == Zone::Graveyard {
                                Action::AnnounceGraveyardBorrowedMana {
                                    card,
                                    graveyard: descriptor.donor,
                                    ability: descriptor.ability,
                                    counter_payment: counter_payment.clone(),
                                    production,
                                }
                            } else {
                                Action::AnnounceBorrowedMana {
                                    card,
                                    exiled: descriptor.donor,
                                    ability: descriptor.ability,
                                    counter_payment: counter_payment.clone(),
                                    production,
                                }
                            })
                        })
                    } else {
                        emit(if descriptor.zone == Zone::Graveyard {
                            Action::AnnounceGraveyardBorrowedAbility {
                                card,
                                graveyard: descriptor.donor,
                                ability: descriptor.ability,
                                counter_payment,
                            }
                        } else {
                            Action::AnnounceBorrowedAbility {
                                card,
                                exiled: descriptor.donor,
                                ability: descriptor.ability,
                                counter_payment,
                            }
                        })
                    }
                },
            ) {
                return true;
            }
        }
        false
    }
}
