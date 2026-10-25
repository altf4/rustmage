//! Activated abilities inherited from exact linked exile incarnations.
use super::ability_world::CharacteristicWorld;
use super::*;

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
    /// u16::MAX denotes the exiled card's intrinsic mana ability.
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
impl BorrowedAbilityView {
    pub(super) fn rules(self) -> &'static ActivatedAbility {
        self.definition.activation(self.ability)
    }
}
pub(super) fn abilities<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
) -> Vec<BorrowedAbilityView> {
    inventory(world, id, true)
}
fn inventory<W: CharacteristicWorld + ?Sized>(
    world: &W,
    id: CardId,
    check_available: bool,
) -> Vec<BorrowedAbilityView> {
    let source = world.object(id);
    if source.zone != Zone::Battlefield
        || !source.card.borrows_exiled_activations()
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
        if link.source != world.reference(id) || link.origin != world.ability_origin(id) {
            continue;
        }
        let card = world.object(link.exiled.card);
        if card.zone != Zone::Exile || world.reference(link.exiled.card) != link.exiled {
            continue;
        }
        for (index, _) in card.card.activations().iter().enumerate() {
            if card.card.grants_activation(index as u16)
                || card.card.self_granted_activation_template(index as u16)
            {
                continue;
            }
            let slot = link.first_slot + index as u64;
            result.push(BorrowedAbilityView {
                source: link.source,
                exiled: link.exiled,
                definition: card.card,
                ability: index as u16,
                slot,
                used: world.borrowed_activation_used(link.source, slot),
            });
        }
        if card.card.mana_color().is_some() {
            let slot = link.first_slot + card.card.activations().len() as u64;
            result.push(BorrowedAbilityView {
                source: link.source,
                exiled: link.exiled,
                definition: card.card,
                ability: u16::MAX,
                slot,
                used: world.borrowed_activation_used(link.source, slot),
            });
        }
    }
    result
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
                let definition = self.state.objects[event.after.card as usize].card;
                let width = u64::try_from(
                    definition.activations().len() + usize::from(definition.mana_color().is_some()),
                )
                .capacity("borrowed activation capacity exhausted");
                if width == 0 {
                    continue;
                }
                let first_slot = self
                    .state
                    .borrowed_links
                    .as_ref()
                    .into_iter()
                    .flat_map(|rows| rows.iter())
                    .filter(|link| link.source == item.object)
                    .map(|link| link.first_slot + link.width)
                    .max()
                    .unwrap_or(0);
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
    ) -> BorrowedAbilityView {
        abilities(self, card)
            .into_iter()
            .find(|a| a.exiled == exiled && a.ability == index)
            .expect("legal borrowed activation")
    }
    pub(super) fn borrowed_activation_item(
        &mut self,
        actor: Player,
        card: CardId,
        exiled: ObjectRef,
        index: u16,
    ) -> SpellView {
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
        self.state
            .borrowed_links
            .as_ref()
            .map_or_else(Vec::new, |links| {
                let mut sources = Vec::new();
                for link in links.iter() {
                    if !sources.contains(&link.source.card) {
                        sources.push(link.source.card);
                    }
                }
                sources
                    .into_iter()
                    .flat_map(|id| abilities(self, id))
                    .collect()
            })
    }
    pub(super) fn visit_borrowed_activations(
        &self,
        card: CardId,
        actor: Player,
        sorcery: bool,
        emit: &mut impl FnMut(Action) -> bool,
    ) -> bool {
        if !self.state.objects[card as usize]
            .card
            .borrows_exiled_activations()
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
            if !self.activation_guard_available(ability.guard, card, actor)
                || (ability.mana_production().is_none() && self.named_activation_prohibited(card))
                || !Self::object_activation_available_with_kinds(
                    &ready_object,
                    self.counter_amounts(card),
                    self.current_kinds(card),
                    || CharacteristicWorld::activation_keywords(self, card),
                    self.state.players[actor.index()].life,
                    ability,
                    0,
                    actor,
                    sorcery,
                )
            {
                continue;
            }
            if activations::counter_payments(
                ability.cost,
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
                            emit(Action::AnnounceBorrowedMana {
                                card,
                                exiled: descriptor.exiled,
                                ability: descriptor.ability,
                                counter_payment: counter_payment.clone(),
                                production,
                            })
                        })
                    } else {
                        emit(Action::AnnounceBorrowedAbility {
                            card,
                            exiled: descriptor.exiled,
                            ability: descriptor.ability,
                            counter_payment,
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
