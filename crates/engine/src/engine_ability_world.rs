//! Borrowed earlier-layer facts shared by actual and speculative mana worlds.
//! The overlay decides presence, control, counters and life; no cached ability
//! status from the real game is carried across a virtual sacrifice or payment.
use super::*;
use crate::cards::{
    ActivationGenericReduction, ActivationReductionAmount, LiveStatRecipients, LiveStaticEffect,
};

#[cfg(feature = "test-support")]
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
thread_local! {
    /// Synthetic selective protection; no authored provider uses this fixture.
    pub(super) static TEST_SACRIFICE_PROHIBITED_OBJECT: std::cell::Cell<Option<ObjectRef>> = const { std::cell::Cell::new(None) };
    /// Test-only provider identity; the query still uses live control and ability layers.
    pub(super) static TEST_SACRIFICE_PROHIBITOR: std::cell::Cell<Option<ObjectRef>> = const { std::cell::Cell::new(None) };
    pub(super) static TEST_EXILE_PROHIBITOR: std::cell::Cell<Option<ObjectRef>> = const { std::cell::Cell::new(None) };
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) trait CharacteristicWorld {
    fn departed_source_power(&self, source: ObjectRef) -> Option<i128> {
        self.game()
            .contributor_source_facts(source)
            .map(|facts| facts.power)
    }
    fn power(&self, id: CardId) -> i128 {
        characteristic_query::CharacteristicQuery::in_world(self)
            .stats(id)
            .power
    }
    /// Derived toughness is sampled only by resolving quantities and costs, never
    /// as an earlier-layer StatValue that could recursively define its own stats.
    fn greatest_creature_toughness(&self, player: Player) -> i128 {
        let query = characteristic_query::CharacteristicQuery::in_world(self);
        (0..self.game().state.objects.len())
            .map(|id| id as CardId)
            .filter(|&id| {
                let object = self.object(id);
                object.zone == Zone::Battlefield
                    && object.controller == player
                    && self.kinds(id) & 1 != 0
            })
            .map(|id| query.stats(id).toughness)
            .max()
            .unwrap_or(0)
    }
    fn controls_greatest_creature_power(&self, player: Player) -> bool {
        let query = characteristic_query::CharacteristicQuery::in_world(self);
        let mut greatest: [Option<i128>; 2] = [None; 2];
        for id in 0..self.game().state.objects.len() {
            let id = id as CardId;
            let object = self.object(id);
            if object.zone == Zone::Battlefield && self.kinds(id) & 1 != 0 {
                let power = query.stats(id).power;
                let best = &mut greatest[object.controller.index()];
                *best = Some(best.map_or(power, |previous| previous.max(power)));
            }
        }
        let own = greatest[player.index()];
        own.is_some() && own >= greatest[player.other().index()]
    }
    fn game(&self) -> &Game;
    fn void_active_this_turn(&self) -> bool {
        self.game().state.void_active_this_turn
    }
    fn borrowed_slots_state(&self) -> &borrowed_activations::SlotRows {
        &self.game().state.borrowed_slots
    }
    fn borrowed_slots(&self) -> &[borrowed_activations::SlotRow] {
        self.borrowed_slots_state()
            .as_ref()
            .map_or(&[], |rows| rows.as_slice())
    }
    fn graveyard_borrowing_state(&self) -> &graveyard_borrowing::Rows {
        &self.game().state.graveyard_borrowing
    }
    fn graveyard_borrowing(&self) -> &[graveyard_borrowing::Row] {
        self.graveyard_borrowing_state()
            .as_ref()
            .map_or(&[], |rows| rows.as_slice())
    }
    fn exhaust_activated_this_turn(&self, player: Player) -> bool {
        self.game().state.players[player.index()]
            .turn_facts
            .exhaust_activated_this_turn
    }
    fn has_enduring_story(&self, player: Player) -> bool {
        self.game().state.players[player.index()].enduring_story
    }
    fn static_sources(&self) -> &[CardId] {
        &self.game().state.static_sources
    }
    fn trigger_multiplier_sources(&self) -> &[CardId] {
        trigger_multiplicity::TriggerMultiplicityState::providers(self.game())
    }
    fn attachment_sources(&self) -> &[CardId] {
        &self.game().state.attachment_sources
    }
    fn controls(&self) -> Option<&continuous_control::ControlState> {
        self.game().state.controls.as_deref()
    }
    fn creatures_died(&self) -> u64 {
        self.game().state.creatures_died_this_turn
    }
    fn card_left_graveyard(&self, player: Player) -> bool {
        self.game().state.players[player.index()].card_left_graveyard_this_turn
    }
    fn roles(&self) -> Option<&roles::RoleState> {
        self.game().state.roles.as_deref()
    }
    fn entry_values(&self) -> &[entry_choices::ChoiceRow] {
        self.game()
            .state
            .entry_values
            .as_deref()
            .map_or(&[], |rows| rows.as_slice())
    }
    fn raw_entry_value(&self, object: ObjectRef) -> Option<EntryChoiceValue> {
        self.entry_values()
            .iter()
            .find(|r| r.object == object)
            .map(|r| r.value)
    }
    fn entry_value(&self, object: ObjectRef) -> Option<EntryChoiceValue> {
        // Fresh choices belong to the copied ability’s acquisition, not merely
        // its physical object. Older choices remain dormant under a later copy.
        self.entry_values()
            .iter()
            .find(|row| row.object == object && row.origin == self.ability_origin(object.card))
            .map(|row| row.value)
    }

    /// Costs use the actual source's current name, independent of the card
    /// defining an indexed, borrowed or externally granted ability.
    fn fixed_mana_choice_can_be_late(&self, source: CardId, definition: Card, index: u16) -> bool {
        let mut cost = definition.activation(index).cost;
        cost.tap = false;
        cost != ActivationCost::ZERO
            || definition.activation_put_counter(index).is_some()
            || self.named_activation_generic_increase(source) != 0
    }
    fn controlled_activation_discounts(&self, source: CardId) -> power_up::ActivationDiscounts {
        let subject = self.object(source);
        let mut discounts = power_up::ActivationDiscounts::default();
        if subject.zone != Zone::Battlefield {
            return discounts;
        }
        let mut query = None;
        for &id in self.static_sources() {
            let provider = self.object(id);
            if provider.zone != Zone::Battlefield || provider.controller != subject.controller {
                continue;
            }
            for fragment in self
                .definition(id)
                .static_ability_groups()
                .iter()
                .flat_map(|g| g.fragments)
            {
                let StaticAbilityFragment::ControlledActivationReduction {
                    kinds,
                    amount,
                    leave_one_mana,
                } = *fragment
                else {
                    continue;
                };
                if kinds != 0 && self.kinds(source) & kinds == 0 {
                    continue;
                }
                let query = query.get_or_insert_with(|| {
                    characteristic_query::CharacteristicQuery::in_world(self)
                });
                if !query.continuous_fragment_available(id, |candidate| {
                    std::ptr::eq(candidate, fragment)
                }) {
                    continue;
                }
                let amount = match amount {
                    ActivationReductionAmount::Fixed(amount) => u64::from(amount),
                    ActivationReductionAmount::ProviderPower => {
                        u64::try_from(query.stats(id).power.max(0)).unwrap_or(u64::MAX)
                    }
                };
                let total = if leave_one_mana {
                    &mut discounts.leave_one
                } else {
                    &mut discounts.ordinary
                };
                *total = total.saturating_add(amount);
            }
        }
        discounts
    }

    fn activation_generic_discount(
        &self,
        source: CardId,
        definition: Card,
        ability: u16,
    ) -> power_up::ActivationDiscounts {
        let controller = self.object(source).controller;
        let authored = definition
            .activation_generic_reductions(ability)
            .map(|reduction| {
                let (value, multiplier) = match reduction {
                    ActivationGenericReduction::Stat { value, multiplier } => (
                        self.game().stat_value_in_world(
                            self,
                            value,
                            self.reference(source),
                            controller,
                        ),
                        multiplier,
                    ),
                    ActivationGenericReduction::SourcePower => (self.power(source), 1),
                };
                u32::try_from(value.max(0))
                    .unwrap_or(u32::MAX)
                    .saturating_mul(multiplier)
            })
            .fold(0u32, u32::saturating_add);
        let subtype =
            definition
                .activation_subtype_discount(ability)
                .map_or(0, |(subtype, other)| {
                    (0..self.game().state.objects.len())
                        .filter(|&id| {
                            let id = id as CardId;
                            let object = self.object(id);
                            object.zone == Zone::Battlefield
                                && (!other || id != source)
                                && object.controller == controller
                                && self.subtypes(id).has(object.card, subtype)
                        })
                        .count()
                        .try_into()
                        .unwrap_or(u32::MAX)
                });
        let power_up =
            if definition.power_up_abilities().contains(&ability) && self.kinds(source) & 1 != 0 {
                (0..self.game().state.objects.len())
                    .filter_map(|id| {
                        let id = id as CardId;
                        let object = self.object(id);
                        (id != source
                            && object.zone == Zone::Battlefield
                            && object.controller == controller
                            && self.printed_abilities_available(id))
                        .then_some(object.card.power_up_other_creature_reduction())
                    })
                    .fold(0u32, u32::saturating_add)
            } else {
                0
            };
        let artifact_token = if self.object(source).zone == Zone::Battlefield
            && self.object(source).is_token()
            && self.kinds(source) & 16 != 0
        {
            let query = characteristic_query::CharacteristicQuery::in_world(self);
            self.static_sources().iter().copied().filter(|&id| {
                let provider = self.object(id);
                provider.zone == Zone::Battlefield && provider.controller == controller
            }).fold(0u32, |total, id| {
                self.definition(id).static_ability_groups().iter().flat_map(|group| group.fragments).fold(total, |total, fragment| {
                    if let crate::cards::StaticAbilityFragment::ArtifactTokenActivationReduction(amount) = *fragment
                        && query.continuous_fragment_available(id, |candidate| std::ptr::eq(candidate, fragment))
                    {
                        total.saturating_add(amount)
                    } else {
                        total
                    }
                })
            })
        } else {
            0
        };
        let equip = self.stored_recipe(source).map_or(0, |recipe| {
            let reduction = recipe.exceptions.equip_generic_reduction;
            if reduction != 0
                && self.object(source).zone == Zone::Battlefield
                && definition.activation(ability).is_equip()
                && !self.face_down(source)
                && self.printed_abilities_available(source)
            {
                reduction
            } else {
                0
            }
        });
        let ordinary = exhaust::generic_discount(self, source, definition, ability)
            .saturating_add(authored)
            .saturating_add(equip)
            .saturating_add(power_up)
            .saturating_add(artifact_token)
            .saturating_add(subtype)
            .saturating_add(
                definition
                    .activation_counter_discount(ability)
                    .map_or(0, |kind| {
                        self.counter_count(source, kind)
                            .try_into()
                            .unwrap_or(u32::MAX)
                    }),
            );
        self.controlled_activation_discounts(source)
            .with_ordinary(u64::from(ordinary))
    }
    fn activation_power_up_costs(
        &self,
        object: ObjectRef,
        definition: Card,
        ability: u16,
    ) -> Option<Box<power_up::ActivationCostReductions>> {
        if !definition.power_up_abilities().contains(&ability) {
            return None;
        }
        let source = self.object(object.card);
        let reductions = if source.zone == Zone::Battlefield
            && self.reference(object.card) == object
            && self.entered_this_turn(object)
        {
            self.recipe(object.card).power_up_mana_reductions()
        } else {
            vec![ManaCost {
                colored: [0; 6],
                generic: 0,
            }]
        };
        Some(Box::new(power_up::ActivationCostReductions {
            reductions,
            ..Default::default()
        }))
    }
    fn projected_self_grant_used(&self, _source: ObjectRef, _timestamp: u64) -> bool {
        false
    }
    fn self_grant_activation_used(
        &self,
        source: ObjectRef,
        timestamp: u64,
        definition: Card,
        ability: u16,
        actor: Player,
    ) -> bool {
        let rules = definition.activation(ability);
        let spent = self.projected_self_grant_used(source, timestamp)
            || self
                .game()
                .state
                .granted_exhaust_uses
                .as_deref()
                .into_iter()
                .flatten()
                .any(|row| {
                    row.source == source
                        && row.grantor == source
                        && row.origin == Some(timestamp)
                        && row.definition == definition
                        && row.ability == ability
                        && (rules.exhaust || row.turn == self.game().state.turn)
                });
        spent && ((rules.exhaust && !exhaust::reuse_available(self, actor)) || rules.once_per_turn)
    }
    fn crew_saddle_eligible(&self, source: ObjectRef, actor: Player, object: ObjectRef) -> bool {
        let o = self.object(object.card);
        object != source
            && self.reference(object.card) == object
            && o.zone == Zone::Battlefield
            && o.controller == actor
            && !o.tapped
            && self.kinds(object.card) & 1 != 0
    }
    fn departed_source_subtypes(&self, source: ObjectRef) -> Option<(Card, GrantedSubtypes)> {
        self.game()
            .contributor_source_facts(source)
            .map(|facts| (facts.card, facts.granted_subtypes))
    }
    fn crew_saddle_subject_matches(
        &self,
        source: ObjectRef,
        subtype: &str,
        fallback: bool,
    ) -> bool {
        if self.reference(source.card) == source
            && self.object(source.card).zone == Zone::Battlefield
        {
            self.subtypes(source.card)
                .has(self.definition(source.card), subtype)
        } else {
            self.departed_source_subtypes(source)
                .map_or(fallback, |(card, subtypes)| subtypes.has(card, subtype))
        }
    }
    #[cfg(feature = "test-support")]
    fn crew_saddle_contribution(&self, card: CardId, subject: CardId, subtype: &str) -> i128 {
        self.crew_saddle_contribution_for(
            card,
            self.subtypes(subject)
                .has(self.definition(subject), subtype),
        )
    }
    #[cfg(feature = "test-support")]
    fn crew_saddle_contribution_for(&self, card: CardId, subject_matches: bool) -> i128 {
        let (base, bonus) = self.crew_saddle_contribution_terms(card, subject_matches);
        base.checked_add(bonus)
            .capacity("individual crew/saddle quantity representation exhausted")
    }
    fn crew_saddle_contribution_terms(&self, card: CardId, subject_matches: bool) -> (i128, i128) {
        let query = characteristic_query::CharacteristicQuery::in_world(self);
        let uses_toughness = |fragment: &crate::cards::StaticAbilityFragment| {
            matches!(
                fragment,
                crate::cards::StaticAbilityFragment::CrewSaddleWithToughness
            )
        };
        if self
            .definition(card)
            .static_ability_groups()
            .iter()
            .any(|g| g.fragments.iter().any(uses_toughness))
            && query.continuous_fragment_available(card, uses_toughness)
        {
            return (query.stats(card).toughness, 0);
        }
        let bonus = if subject_matches
            && query
                .keywords(card)
                .contains(Keywords::CREW_SADDLE_PLUS_TWO)
        {
            2
        } else {
            0
        };
        (query.stats(card).power, bonus)
    }
    fn named_activation_generic_increase(&self, id: CardId) -> u32 {
        let query = characteristic_query::CharacteristicQuery::in_world(self);
        self.static_sources()
            .iter()
            .copied()
            .fold(0u32, |total, source| {
                let provider = self.object(source);
                if provider.zone != Zone::Battlefield
                    || !matches!(self.entry_value(self.reference(source)),
                    Some(EntryChoiceValue::CardName(chosen)) if self.rules_name_matches(id, chosen))
                {
                    return total;
                }
                provider
                    .card
                    .static_ability_groups()
                    .iter()
                    .flat_map(|g| g.fragments)
                    .fold(total, |total, fragment| {
                        let crate::cards::StaticAbilityFragment::NamedActivationGenericIncrease(
                            amount,
                        ) = *fragment
                        else {
                            return total;
                        };
                        if query.continuous_fragment_available(source, |candidate| {
                            std::ptr::eq(candidate, fragment)
                        }) {
                            total
                                .checked_add(amount)
                                .capacity("activation cost increase exhausted")
                        } else {
                            total
                        }
                    })
            })
    }
    fn battle_protector(&self, object: ObjectRef) -> Option<Player> {
        self.game().battle_protector(object)
    }
    fn object(&self, id: CardId) -> &Object;
    /// The immutable layer-one definition is materialized in both actual and
    /// speculative objects; physical identity lives separately while copying.
    fn definition(&self, id: CardId) -> Card {
        self.object(id).card
    }
    fn ability_origin(&self, id: CardId) -> Option<u64> {
        (self.object(id).zone == Zone::Battlefield)
            .then(|| self.game().copy_ability_origin(self.reference(id)))
            .flatten()
    }
    fn provided_origin(&self, object: ObjectRef) -> Option<u64> {
        self.game().copied_grant_origin(object)
    }
    fn face_down(&self, id: CardId) -> bool {
        self.game().is_face_down(self.reference(id))
    }
    fn temporary_copy_rows_state(&self) -> &temporary_copy::Rows {
        &self.game().state.temporary_copies
    }
    fn temporary_copy_view(&self, object: ObjectRef) -> Option<TemporaryCopyView> {
        self.temporary_copy_rows_state()
            .as_ref()?
            .iter()
            .find(|row| row.object == object)
            .map(temporary_copy::Row::view)
    }
    fn stored_recipe(&self, id: CardId) -> Option<CopyRecipe> {
        self.game().copy_recipe(self.reference(id))
    }
    fn is_aura(&self, id: CardId) -> bool {
        self.kinds(id) & 32 != 0 && self.subtypes(id).has(self.definition(id), "Aura")
    }
    fn is_equipment(&self, id: CardId) -> bool {
        self.kinds(id) & 16 != 0 && self.subtypes(id).has(self.definition(id), "Equipment")
    }
    fn mana_value(&self, id: CardId) -> u64 {
        let object = self.object(id);
        if object.zone == Zone::Exile && self.game().exiled_face_down(self.reference(id))
            || self.recipe(id).exceptions.embalm
        {
            return 0;
        }
        if object.zone == Zone::Battlefield
            && self.temporary_copy_view(self.reference(id)).is_none()
            && let Some(entry) = self.roles().and_then(|roles| {
                roles
                    .melded_permanents
                    .iter()
                    .find(|row| {
                        row.object == self.reference(id) && row.entry.combined == object.card
                    })
                    .map(|row| row.entry)
            })
        {
            return entry
                .front_cards
                .iter()
                .map(|card| card.fixed_mana_value())
                .sum();
        }
        let Some(front) = object.card.face_parent() else {
            return object.card.fixed_mana_value();
        };
        if object.zone != Zone::Battlefield || !front.transforming_double_face() {
            return object.card.fixed_mana_value();
        }
        if let Some(recipe) = self.stored_recipe(id) {
            return match recipe.face_structure() {
                copies::CopyFaces::Double {
                    front: copied_front,
                    back,
                    back_up: true,
                    ..
                } if copied_front == front && back == object.card => front.fixed_mana_value(),
                _ => 0,
            };
        }
        if self.temporary_copy_view(self.reference(id)).is_some() {
            0
        } else {
            front.fixed_mana_value()
        }
    }
    fn recipe(&self, id: CardId) -> CopyRecipe {
        self.stored_recipe(id)
            .unwrap_or_else(|| CopyRecipe::new(self.definition(id)))
    }
    fn copiable_name(&self, id: CardId) -> &'static str {
        self.recipe(id)
            .overrides
            .and_then(|value| value.name)
            .unwrap_or_else(|| self.definition(id).name())
    }
    fn intrinsic_keywords(&self, id: CardId) -> Keywords {
        self.recipe(id).intrinsic_keywords()
    }
    fn copied_keywords(&self, id: CardId) -> Keywords {
        let recipe = self.recipe(id);
        let keywords = recipe
            .overrides
            .map_or(Keywords::NONE, |value| value.keywords);
        if recipe.exceptions.sacrifice_abilities != 0
            || recipe.exceptions.conditional_sacrifice_abilities != 0
        {
            keywords.union(Keywords::HASTE)
        } else {
            keywords
        }
    }
    fn legendary(&self, id: CardId) -> bool {
        let intrinsic = self
            .recipe(id)
            .overrides
            .and_then(|value| value.legendary)
            .unwrap_or_else(|| self.definition(id).is_legendary());
        intrinsic
            || characteristic_layers::grants_legendary(self, id)
            || self.object(id).zone == Zone::Battlefield
                && self.attachment_sources().iter().any(|&source| {
                    let aura = self.object(source);
                    aura.zone == Zone::Battlefield
                        && self.kinds(source) & 1 == 0
                        && aura.attached_to == Some(self.reference(id))
                        && aura.card.static_ability_groups().iter().any(|group| {
                            group.first_layer == crate::cards::AbilityLayer::Type
                                && group.fragments.iter().any(|fragment| {
                                    matches!(
                                        fragment,
                                        crate::cards::StaticAbilityFragment::AttachedLegendary
                                    )
                                })
                        })
                })
    }
    fn intrinsic_mana_color(&self, id: CardId) -> Option<usize> {
        let card = self.definition(id);
        if self.has_ability_removal() {
            let evaluation = ability_evaluation::evaluate_abilities(self);
            let state = &evaluation.result.objects[id as usize];
            if !card.is_land() && state.printed && card.mana_color().is_some() {
                return card.mana_color();
            }
            if let Some(color) = state
                .mana
                .iter()
                .filter(|grant| grant.subtype_derived)
                .find_map(|grant| match grant.production {
                    ManaProduction::Fixed(mana) => mana.iter().position(|amount| *amount != 0),
                    _ => None,
                })
            {
                return Some(color);
            }
            return if state.printed
                && !(card.is_land() && card.printed_intrinsic_mana_colors().next().is_some())
            {
                card.mana_color()
            } else {
                None
            };
        }
        // The generated shortcut on a nonland (e.g. Llanowar Elves) is a
        // printed ability, independent of any types the object gains later.
        if !card.is_land() && card.mana_color().is_some() {
            return card.mana_color();
        }
        if self.kinds(id) & 2 != 0
            && let Some(color) = ["Plains", "Island", "Swamp", "Mountain", "Forest"]
                .iter()
                .position(|subtype| self.subtypes(id).has(card, subtype))
        {
            return Some(color);
        }
        if card.is_land() && card.printed_intrinsic_mana_colors().next().is_some() {
            None
        } else {
            card.mana_color()
        }
    }
    /// Sparse rows belong to this world, including virtual counter payments.
    fn named_counter_rows(&self) -> &[live_counters::NamedCounterRow] {
        self.game()
            .state
            .named_counters
            .as_ref()
            .map_or(&[], |rows| rows.as_slice())
    }
    fn counter_amounts(&self, id: CardId) -> CounterAmounts<'_> {
        live_counters::amounts(
            &self.game().state.named_counters,
            self.reference(id),
            self.object(id).counters,
        )
    }
    fn counter_count(&self, id: CardId, kind: CounterKind) -> u64 {
        self.counter_amounts(id).get(kind)
    }
    fn life(&self, player: Player) -> i128;
    fn gained_life(&self, player: Player) -> bool;
    fn lost_life(&self, player: Player) -> bool {
        self.game().state.players[player.index()].life_lost_this_turn > 0
    }
    fn attacking(&self, id: CardId) -> bool;
    /// Exact current combat subject in this actual or projected world.
    fn attacking_defender(&self, id: CardId) -> Option<CombatDefender> {
        if !self.attacking(id)
            || self.object(id).zone != Zone::Battlefield
            || self.reference(id) != self.game().object_ref(id)
        {
            return None;
        }
        self.game()
            .state
            .attackers
            .iter()
            .find(|row| row.card == id)?
            .defender
    }

    /// Effective string name in this actual or speculative object world.
    /// Name-changing effects belong here, before later characteristic layers.
    fn name(&self, id: CardId) -> &'static str {
        characteristic_layers::name(self, id)
    }
    fn rules_name_matches(&self, id: CardId, chosen: crate::CardName) -> bool {
        let current = self.name(id);
        if !matches!(self.object(id).zone, Zone::Battlefield | Zone::Stack)
            && current == self.definition(id).name()
            && let Some(names) = outside_names::names(self.definition(id))
        {
            return names.iter().any(|name| chosen.matches(name));
        }
        chosen.matches(current)
    }
    fn colors(&self, id: CardId) -> u8 {
        characteristic_layers::colors(self, id)
    }

    fn borrowed_activation_used(&self, source: ObjectRef, slot: u64) -> bool {
        self.game()
            .state
            .borrowed_uses
            .as_ref()
            .is_some_and(|rows| {
                rows.iter().any(|row| {
                    row.source == source
                        && row.slot == slot
                        && (row.exhaust || row.turn == self.game().state.turn)
                })
            })
    }
    /// Activation-only permission: never grants the haste keyword or attack permission.
    fn activation_keywords(&self, id: CardId) -> Keywords {
        let query = characteristic_query::CharacteristicQuery::in_world(self);
        let keywords = query.keywords(id);
        let object = self.object(id);
        if object.zone != Zone::Battlefield
            || self.kinds(id) & 1 == 0
            || !object.summoning_sick
            || keywords.contains(Keywords::HASTE)
        {
            return keywords;
        }
        let grants = |f: &crate::cards::StaticAbilityFragment| {
            matches!(
                f,
                crate::cards::StaticAbilityFragment::ActivateControlledCreaturesAsHaste
            )
        };
        if self.static_sources().iter().any(|&source| {
            let provider = self.object(source);
            provider.zone == Zone::Battlefield
                && provider.controller == object.controller
                && provider
                    .card
                    .static_ability_groups()
                    .iter()
                    .any(|g| g.fragments.iter().any(grants))
                && query.continuous_fragment_available(source, grants)
        }) {
            keywords.union(Keywords::HASTE)
        } else {
            keywords
        }
    }
    fn zone_move_prevented(&self, object: ObjectRef, requested: Zone) -> bool {
        #[cfg(feature = "test-support")]
        if zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|p| p.get() == Some(object)) {
            return true;
        }
        let target = self.object(object.card);
        if self.reference(object.card) != object
            || target.zone != Zone::Battlefield
            || requested != Zone::Graveyard
        {
            return false;
        }
        let query = characteristic_query::CharacteristicQuery::in_world(self);
        self.static_sources().iter().any(|&source| {
            let provider = self.object(source);
            if provider.zone != Zone::Battlefield || provider.controller != target.controller {
                return false;
            }
            let fragment = |f: &crate::cards::StaticAbilityFragment| {
                matches!(
                    f,
                    crate::cards::StaticAbilityFragment::PreventControlledPermanentGraveyardMove
                )
            };
            let printed = provider
                .card
                .static_ability_groups()
                .iter()
                .any(|group| group.fragments.iter().any(fragment))
                && query.continuous_fragment_available(source, fragment);
            #[cfg(feature = "test-support")]
            let synthetic = zone_replacements::TEST_PREVENT_PROVIDER
                .with(|p| p.get() == Some(self.reference(source)))
                && self.printed_abilities_available(source);
            #[cfg(not(feature = "test-support"))]
            let synthetic = false;
            printed || synthetic
        })
    }
    /// Keep potential providers in speculative worlds even while blanked.
    fn sacrifice_prohibition_source(&self, id: CardId) -> bool {
        let object = self.object(id);
        let printed = object.card.static_ability_groups().iter().any(|group| {
            group.fragments.iter().any(|f| {
                matches!(
                    f,
                    crate::cards::StaticAbilityFragment::ControlledPermanentsCannotBeSacrificed
                ) || matches!(f, crate::cards::StaticAbilityFragment::KeywordGrant(_, grant)
                    if grant.keywords.contains(Keywords::CANT_BE_SACRIFICED))
            })
        });
        #[cfg(feature = "test-support")]
        let synthetic = TEST_SACRIFICE_PROHIBITOR.with(|p| p.get() == Some(self.reference(id)));
        #[cfg(not(feature = "test-support"))]
        let synthetic = false;
        printed || synthetic
    }
    /// Whether a permanent can be sacrificed in this world, including projected
    /// source control and live layer-six suppression.
    fn sacrifice_prohibited(&self, id: CardId) -> bool {
        let target = self.object(id);
        if target.zone != Zone::Battlefield {
            return true;
        }
        #[cfg(feature = "test-support")]
        if TEST_SACRIFICE_PROHIBITED_OBJECT.with(|object| object.get() == Some(self.reference(id)))
        {
            return true;
        }
        let query = characteristic_query::CharacteristicQuery::in_world(self);
        if query.keywords(id).contains(Keywords::CANT_BE_SACRIFICED) {
            return true;
        }
        (0..self.game().state.objects.len()).any(|index| {
            let source = index as CardId;
            let provider = self.object(source);
            if provider.zone != Zone::Battlefield || provider.controller != target.controller {
                return false;
            }
            let fragment = |f: &crate::cards::StaticAbilityFragment| {
                matches!(
                    f,
                    crate::cards::StaticAbilityFragment::ControlledPermanentsCannotBeSacrificed
                )
            };
            let printed = provider
                .card
                .static_ability_groups()
                .iter()
                .any(|group| group.fragments.iter().any(fragment))
                && query.continuous_fragment_available(source, fragment);
            #[cfg(feature = "test-support")]
            let synthetic = TEST_SACRIFICE_PROHIBITOR
                .with(|p| p.get() == Some(self.reference(source)))
                && self.printed_abilities_available(source);
            #[cfg(not(feature = "test-support"))]
            let synthetic = false;
            printed || synthetic
        })
    }
    /// A true prohibition is distinct from a replacement or prevention.
    fn graveyard_exile_prohibited(&self, id: CardId) -> bool {
        if self.object(id).zone != Zone::Graveyard || !self.object(id).is_card() {
            return false;
        }
        let query = characteristic_query::CharacteristicQuery::in_world(self);
        (0..self.game().state.objects.len()).any(|index| {
            let source = index as CardId;
            let provider = self.object(source);
            if provider.zone != Zone::Battlefield {
                return false;
            }
            let fragment = |f: &crate::cards::StaticAbilityFragment| {
                matches!(
                    f,
                    crate::cards::StaticAbilityFragment::GraveyardCardsCannotBeExiled
                )
            };
            let printed = provider
                .card
                .static_ability_groups()
                .iter()
                .any(|group| group.fragments.iter().any(fragment))
                && query.continuous_fragment_available(source, fragment);
            #[cfg(feature = "test-support")]
            let synthetic = TEST_EXILE_PROHIBITOR.with(|p| p.get() == Some(self.reference(source)))
                && self.printed_abilities_available(source);
            #[cfg(not(feature = "test-support"))]
            let synthetic = false;
            printed || synthetic
        })
    }
    fn activation_prohibited(&self, actor: Player, id: CardId) -> bool {
        let target = self.object(id);
        (target.zone == Zone::Battlefield
            && self.static_sources().iter().any(|&source| {
                let provider = self.object(source);
                if provider.zone != Zone::Battlefield {
                    return false;
                }
                provider.card.static_ability_groups().iter().any(|group| {
                    group.fragments.iter().any(|fragment| {
                        let prohibited = match *fragment {
                        crate::cards::StaticAbilityFragment::ActivatedPermanentKindsProhibited(
                            kinds,
                        ) => self.kinds(id) & kinds != 0,
                        crate::cards::StaticAbilityFragment::AttachedActivationsProhibited => {
                            provider.attached_to == Some(self.reference(id))
                                && self.attached(self.reference(source), id)
                        }
                        _ => false,
                    };
                        prohibited
                            && characteristic_query::CharacteristicQuery::in_world(self)
                                .continuous_fragment_available(source, |found| {
                                    std::ptr::eq(found, fragment)
                                })
                    })
                })
            }))
            || (target.zone == Zone::Battlefield
                && self.roles().is_some_and(|roles| {
                    roles.source_linked_effects.iter().any(|row| {
                        row.kind == SourceLinkedEffectKind::PreventActivationsUntilUntapped
                            && row.recipient == self.reference(id)
                            && source_linked_effects::active(self, row)
                    })
                }))
            || target.zone == Zone::Battlefield
                && self.kinds(id) & (1 | 16 | 32) != 0
                && self.static_sources().iter().any(|&source| {
                    let provider = self.object(source);
                    provider.zone == Zone::Battlefield
                        && provider.controller == self.game().state.active
                        && provider.controller != actor
                        && provider.card.static_cast_restriction() & 2 != 0
                        && self.printed_abilities_available(source)
                })
    }
    fn harnessed(&self, id: CardId) -> bool {
        self.object(id).zone == Zone::Battlefield
            && self
                .roles()
                .is_some_and(|state| state.harnessed.contains(&self.reference(id)))
    }
    fn is_token(&self, id: CardId) -> bool {
        self.object(id).is_token()
    }

    fn plus_one_placed_this_turn(&self, object: ObjectRef, player: Player) -> bool {
        counter_placement_history::plus_one_placed(
            &self.game().state.counter_placement_history,
            object,
            player,
        )
    }
    fn placement_history_recipient(
        &self,
        source: ObjectRef,
        filter: CountFilter,
        id: CardId,
    ) -> bool {
        let controller = self.object(source.card).controller;
        self.plus_one_placed_this_turn(self.reference(id), controller)
            && Game::count_object_matches(
                filter,
                source,
                controller,
                id,
                self.object(id),
                &|id| self.subtypes(id),
                &|id, _| self.kinds(id),
            )
    }
    fn placement_history_keywords(&self, id: CardId) -> Keywords {
        let object = self.object(id);
        if object.zone != Zone::Battlefield
            || !self.plus_one_placed_this_turn(self.reference(id), object.controller)
        {
            return Keywords::NONE;
        }
        object
            .card
            .static_ability_groups()
            .iter()
            .flat_map(|g| g.fragments)
            .fold(Keywords::NONE, |keywords, fragment| {
                if let crate::cards::StaticAbilityFragment::SourcePlusOnePlacedKeywords(grant) =
                    *fragment
                {
                    keywords.union(grant)
                } else {
                    keywords
                }
            })
    }

    fn source_conditional_keywords(&self, id: CardId) -> Keywords {
        let object = self.object(id);
        if object.zone != Zone::Battlefield {
            return Keywords::NONE;
        }
        object
            .card
            .static_ability_groups()
            .iter()
            .flat_map(|group| group.fragments)
            .fold(Keywords::NONE, |result, fragment| {
                if let crate::cards::StaticAbilityFragment::SourceConditionalKeywords {
                    condition,
                    keywords,
                } = *fragment
                    && self.value(condition, id) != 0
                {
                    result.union(keywords)
                } else {
                    result
                }
            })
    }

    fn counter_subtype_keywords(&self, id: CardId) -> Keywords {
        let object = self.object(id);
        if object.zone != Zone::Battlefield {
            return Keywords::NONE;
        }
        let creature = self.kinds(id) & 1 != 0;
        let mut result = Keywords::NONE;
        for group in object.card.static_ability_groups() {
            for fragment in group.fragments {
                if let crate::cards::StaticAbilityFragment::SourceCounterKeywords {
                    kind,
                    minimum,
                    keywords,
                }
                | crate::cards::StaticAbilityFragment::SourceCounterSubtype {
                    kind,
                    minimum,
                    keywords,
                    ..
                } = *fragment
                    && creature
                    && self.counter_count(id, kind) >= minimum
                {
                    result = result.union(keywords);
                }
                if let crate::cards::StaticAbilityFragment::SourceCounterCreature {
                    own_turn,
                    kind,
                    minimum,
                    keywords,
                    ..
                } = *fragment
                    && (!own_turn || object.controller == self.game().state.active)
                    && self.counter_count(id, kind) >= minimum
                {
                    result = result.union(keywords);
                }
            }
        }
        result
    }

    fn controlled_creature_trigger_applies(
        &self,
        source: ObjectRef,
        id: CardId,
        other: bool,
        nontoken: bool,
    ) -> bool {
        let target = self.object(id);
        target.zone == Zone::Battlefield
            && target.controller == self.object(source.card).controller
            && self.kinds(id) & 1 != 0
            && (!other || self.reference(id) != source)
            && (!nontoken || !self.is_token(id))
    }
    fn other_tapped_legendary_creature_applies(
        &self,
        source: ObjectRef,
        id: CardId,
        condition: StatValue,
    ) -> bool {
        let target = self.object(id);
        target.zone == Zone::Battlefield
            && target.controller == self.object(source.card).controller
            && self.reference(id) != source
            && target.tapped
            && self.kinds(id) & 1 != 0
            && self.legendary(id)
            && self.value(condition, source.card) != 0
    }
    fn death_baron_applies(&self, source: ObjectRef, id: CardId) -> bool {
        let target = self.object(id);
        target.zone == Zone::Battlefield
            && self.kinds(id) & 1 != 0
            && target.controller == self.object(source.card).controller
            && ((self.subtypes(id).has(target.card, "Skeleton"))
                || (self.reference(id) != source && (self.subtypes(id).has(target.card, "Zombie"))))
    }
    fn has_ability_removal(&self) -> bool {
        self.static_sources().iter().any(|&id| {
            self.object(id).zone == Zone::Battlefield
                && self
                    .object(id)
                    .card
                    .static_ability_groups()
                    .iter()
                    .any(|group| {
                        group.fragments.iter().any(|fragment| {
                            matches!(
                                fragment,
                                crate::cards::StaticAbilityFragment::ChosenBasicLandType
                            )
                        })
                    })
                && matches!(
                    self.entry_value(self.reference(id)),
                    Some(EntryChoiceValue::BasicLandType(_))
                )
        }) || self.roles().is_some_and(|state| {
            state
                .resolved_ability_modifications
                .iter()
                .any(|row| return_ability_modification::active(self, row))
                || state.source_linked_effects.iter().any(|row| {
                    row.kind.removes_abilities() && source_linked_effects::active(self, row)
                })
                || state.ability_removal_sources.iter().any(|&source| {
                    self.object(source.card).zone == Zone::Battlefield
                        && self.reference(source.card) == source
                })
        })
    }
    fn fragment_available(
        &self,
        id: CardId,
        matches: impl Fn(&crate::cards::StaticAbilityFragment) -> bool,
    ) -> bool {
        if self.object(id).zone != Zone::Battlefield {
            return true;
        }
        let evaluation = ability_evaluation::evaluate_if_needed(self);
        if evaluation.is_none()
            && !matches!(
                self.object(id).card.entry_choice(),
                Some(EntryChoiceKind::NamedModes(_))
            )
        {
            return true;
        }
        self.object(id)
            .card
            .static_ability_groups()
            .iter()
            .enumerate()
            .any(|(index, group)| {
                group.fragments.iter().any(&matches)
                    && entry_modes::group_enabled(self, self.reference(id), group)
                    && evaluation.as_ref().is_none_or(|evaluation| {
                        evaluation.group_available(self.reference(id), index)
                    })
            })
    }
    fn printed_abilities_available(&self, id: CardId) -> bool {
        if self.object(id).zone == Zone::Battlefield
            && self
                .object(id)
                .card
                .static_ability_groups()
                .iter()
                .any(|group| {
                    group.fragments.iter().any(|fragment| {
                        matches!(
                            fragment,
                            crate::cards::StaticAbilityFragment::ChosenBasicLandType
                        )
                    })
                })
            && matches!(
                self.entry_value(self.reference(id)),
                Some(EntryChoiceValue::BasicLandType(_))
            )
        {
            return ability_evaluation::evaluate_abilities(self).result.objects[id as usize]
                .printed;
        }
        if self.roles().is_some_and(|state| {
            state.resolved_ability_modifications.iter().any(|row| {
                row.recipient == self.reference(id)
                    && return_ability_modification::active(self, row)
            }) || state.source_linked_effects.iter().any(|row| {
                row.recipient == self.reference(id)
                    && row.kind.removes_abilities()
                    && source_linked_effects::active(self, row)
            })
        }) {
            return ability_evaluation::evaluate_abilities(self).result.objects[id as usize]
                .printed;
        }
        // Only an exact attached removal provider can remove printed abilities.
        // Unrelated sources need no whole-world layer-six evaluation. Keep the
        // full evaluator for candidates: competing effects may blank a provider.
        let recipient = self.reference(id);
        !self.roles().is_some_and(|state| {
            state
                .ability_removal_sources
                .iter()
                .any(|&source| self.attachment_link(source, recipient))
        }) || ability_evaluation::evaluate_abilities(self).result.objects[id as usize].printed
    }

    fn reference(&self, id: CardId) -> ObjectRef {
        ObjectRef {
            card: id,
            generation: self.object(id).generation,
        }
    }
    fn kinds(&self, id: CardId) -> u16 {
        characteristic_layers::kinds(self, id, self.object(id))
    }
    fn subtypes(&self, id: CardId) -> GrantedSubtypes {
        subtypes::in_world(self, id)
    }
    fn value(&self, value: StatValue, source: CardId) -> i128 {
        self.game().stat_value_in_world(
            self,
            value,
            self.reference(source),
            self.object(source).controller,
        )
    }
    fn entered_this_turn(&self, source: ObjectRef) -> bool {
        self.game().entered_this_turn(source)
    }
    /// Basic is a supertype; a basic land type alone is insufficient.
    /// Read this world so virtual departures and control changes take effect.
    fn controls_basic_land(&self, player: Player) -> bool {
        self.game().state.objects.iter().enumerate().any(|(id, _)| {
            let object = self.object(id as CardId);
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.card.is_basic_land()
                && self.kinds(id as CardId) & 2 != 0
        })
    }
    fn own_turn_source_or_controlled_creature_kind(
        &self,
        source: ObjectRef,
        id: CardId,
        kinds: u16,
    ) -> bool {
        let provider = self.object(source.card);
        let target = self.object(id);
        provider.zone == Zone::Battlefield
            && self.reference(source.card) == source
            && provider.controller == self.game().state.active
            && target.zone == Zone::Battlefield
            && (id == source.card
                || (target.controller == provider.controller && {
                    let current = self.kinds(id);
                    current & 1 != 0 && current & kinds != 0
                }))
    }
    fn conditional_ward_applies(
        &self,
        source: ObjectRef,
        condition: StatValue,
        filter: CountFilter,
        id: CardId,
    ) -> bool {
        self.object(source.card).zone == Zone::Battlefield
            && self.reference(source.card) == source
            && self.value(condition, source.card) != 0
            && Game::count_object_matches(
                filter,
                source,
                self.object(source.card).controller,
                id,
                self.object(id),
                &|id| self.subtypes(id),
                &|id, _| self.kinds(id),
            )
    }
    fn static_tap_mana(
        &self,
        source: ObjectRef,
        id: CardId,
        fragment: crate::cards::StaticAbilityFragment,
    ) -> Option<ManaProduction> {
        use crate::cards::{LandManaRecipients, StaticAbilityFragment};
        let production = match fragment {
            StaticAbilityFragment::LandMana { production, .. }
            | StaticAbilityFragment::OtherControlledPermanentTapMana(production)
            | StaticAbilityFragment::ControlledPermanentTapMana { production, .. }
            | StaticAbilityFragment::ControlledSubtypeTapMana { production, .. } => production,
            _ => return None,
        };
        let provider = self.object(source.card);
        let target = self.object(id);
        if provider.zone != Zone::Battlefield
            || self.reference(source.card) != source
            || target.zone != Zone::Battlefield
        {
            return None;
        }
        let own = provider.controller == target.controller;
        let qualifies = match fragment {
            StaticAbilityFragment::LandMana { recipients, .. } => {
                let qualifies = match recipients {
                    LandManaRecipients::YouControl => own,
                    LandManaRecipients::SourceChosenName => {
                        matches!(self.entry_value(source), Some(EntryChoiceValue::CardName(name)) if name.matches(self.name(id)))
                    }
                };
                qualifies && self.kinds(id) & 2 != 0
            }
            StaticAbilityFragment::OtherControlledPermanentTapMana(_) => own && id != source.card,
            StaticAbilityFragment::ControlledPermanentTapMana { kinds, .. } => {
                own && self.kinds(id) & kinds != 0
            }
            StaticAbilityFragment::ControlledSubtypeTapMana { subtype, .. } => {
                own && self.subtypes(id).has(target.card, subtype)
            }
            _ => unreachable!("static mana grant"),
        };
        qualifies.then_some(production)
    }
    /// Availability of an exact immutable activation template granted to its
    /// own source. This is not a printed-ability permission or a flattened tap
    /// mana grant: the activation retains its slot, costs and use identity.
    fn source_activation_template_available(&self, id: CardId, index: u16) -> bool {
        let object = self.object(id);
        let Some(ability) = object.card.activations().get(usize::from(index)) else {
            return false;
        };
        if object.zone != ability.source_zone()
            || !object.card.static_ability_groups().iter().any(|group| {
                group.fragments.iter().any(|fragment| matches!(fragment,
                    crate::cards::StaticAbilityFragment::SourceMaxSpeedActivation(slot) | crate::cards::StaticAbilityFragment::SourceEntryModeActivation { ability: slot, .. } if *slot == index))
            })
        {
            return false;
        }
        let controller = if object.zone == Zone::Battlefield {
            object.controller
        } else {
            object.owner
        };
        let eligible = object.card.static_ability_groups().iter().any(|group| {
            entry_modes::group_enabled(self, self.reference(id), group)
                && group.fragments.iter().any(|fragment| match *fragment {
                    crate::cards::StaticAbilityFragment::SourceMaxSpeedActivation(slot) => {
                        slot == index && self.game().state.players[controller.index()].speed >= 4
                    }
                    crate::cards::StaticAbilityFragment::SourceEntryModeActivation {
                        mode,
                        ability,
                    } => {
                        ability == index
                            && entry_modes::active_mode(self, self.reference(id)) == Some(mode)
                    }
                    _ => false,
                })
        });
        if !eligible {
            return false;
        }
        match object.zone {
            Zone::Battlefield => {
                // The evaluator itself uses only the immutable template and
                // earlier-layer predicates, never this query, so it is acyclic.
                ability_evaluation::evaluate_abilities(self).result.objects[id as usize]
                    .grants
                    .iter()
                    .any(|grant| {
                        grant.source == self.reference(id)
                            && grant.granting_card == object.card
                            && grant.kind == ability_order::GrantedAbilityKind::Template(index)
                    })
            }
            Zone::Hand | Zone::Graveyard => {
                self.game().state.players[controller.index()].speed >= 4
                    && object.card.static_ability_groups().iter().any(|group| {
                        group.fragments.iter().any(|fragment| matches!(fragment,
                            crate::cards::StaticAbilityFragment::SourceMaxSpeedActivation(slot) if *slot == index))
                    })
            }
            _ => false,
        }
    }
    fn granted_mana(&self, id: CardId) -> Vec<ability_order::GrantedManaAbility> {
        // Animation grants only keywords. It cannot remove a provider's printed
        // ability or change a mana grant, so only removal needs layer-six order.
        if self.has_ability_removal()
            || self.roles().is_some_and(|roles| {
                roles.source_linked_effects.iter().any(|row| {
                    matches!(
                        row.kind,
                        SourceLinkedEffectKind::GrantTwoColorlessUntilSourceCast
                            | SourceLinkedEffectKind::LandUntilControllerNextTurn
                    ) && source_linked_effects::active(self, row)
                })
            })
        {
            let evaluation = ability_evaluation::evaluate_abilities(self);
            let mut result = evaluation.result.objects[id as usize].mana.clone();
            if evaluation.result.objects[id as usize].printed
                && self.kinds(id) & 2 != 0
                && !(!self.definition(id).is_land()
                    && evaluation.result.objects[id as usize].printed
                    && self.definition(id).mana_color().is_some())
                && let Some(first) = self.intrinsic_mana_color(id)
            {
                let mut production = [0; 6];
                production[first] = 1;
                if let Some(index) = result.iter().position(|grant| {
                    grant.subtype_derived
                        && grant.source == self.reference(id)
                        && grant.production == ManaProduction::Fixed(production)
                }) {
                    result.remove(index);
                }
            }
            return result;
        }
        let mut result = Vec::new();
        self.append_additional_basic_subtype_mana(id, &mut result);
        for &source in self.attachment_sources() {
            let reference = self.reference(source);
            if self.kinds(source) & 1 != 0 || !self.attachment_link(reference, self.reference(id)) {
                continue;
            }
            for group in self.object(source).card.static_ability_groups() {
                for fragment in group.fragments {
                    if let crate::cards::StaticAbilityFragment::AttachmentMana(production)
                    | crate::cards::StaticAbilityFragment::AttachmentSacrificeMana(
                        production,
                    ) = *fragment
                    {
                        result.push(ability_order::GrantedManaAbility {
                            source: reference,
                            production,
                            subtype_derived: false,
                            sacrifice: matches!(
                                *fragment,
                                crate::cards::StaticAbilityFragment::AttachmentSacrificeMana(_)
                            ),
                        });
                    }
                }
            }
        }
        for &source in self.static_sources() {
            for group in self.object(source).card.static_ability_groups() {
                for fragment in group.fragments {
                    if let Some(production) =
                        self.static_tap_mana(self.reference(source), id, *fragment)
                    {
                        result.push(ability_order::GrantedManaAbility {
                            source: self.reference(source),
                            production,
                            sacrifice: false,
                            subtype_derived: false,
                        });
                    }
                }
            }
        }
        result
    }
    fn append_additional_basic_subtype_mana(
        &self,
        id: CardId,
        result: &mut Vec<ability_order::GrantedManaAbility>,
    ) {
        let card = self.definition(id);
        if self.object(id).zone != Zone::Battlefield || self.kinds(id) & 2 == 0 {
            return;
        }
        let Some(first) = self.intrinsic_mana_color(id) else {
            return;
        };
        for (color, subtype) in ["Plains", "Island", "Swamp", "Mountain", "Forest"]
            .iter()
            .enumerate()
        {
            if (color != first || (!card.is_land() && card.mana_color().is_some()))
                && self.subtypes(id).has(card, subtype)
            {
                let mut production = [0; 6];
                production[color] = 1;
                result.push(ability_order::GrantedManaAbility {
                    source: self.reference(id),
                    production: ManaProduction::Fixed(production),
                    sacrifice: false,
                    subtype_derived: true,
                });
            }
        }
    }
    fn attachment_link(&self, source: ObjectRef, recipient: ObjectRef) -> bool {
        let object = self.object(source.card);
        object.zone == Zone::Battlefield
            && self.reference(source.card) == source
            && self.object(recipient.card).zone == Zone::Battlefield
            && self.reference(recipient.card) == recipient
            && object.attached_to == Some(recipient)
    }
    fn equipment_attached_to(&self, source: CardId, recipient: ObjectRef) -> bool {
        self.attachment_link(self.reference(source), recipient) && self.is_equipment(source)
    }
    fn equipped(&self, id: CardId) -> bool {
        self.object(id).zone == Zone::Battlefield
            && self
                .attachment_sources()
                .iter()
                .any(|&source| self.equipment_attached_to(source, self.reference(id)))
    }
    fn has_modification(&self, id: CardId) -> bool {
        let object = self.object(id);
        object.zone == Zone::Battlefield
            && Game::object_modified_with(
                self.reference(id),
                self.counter_amounts(id).has_any(),
                self.attachment_sources()
                    .iter()
                    .map(|&id| (id, self.object(id))),
                |id, attachment| {
                    self.is_equipment(id)
                        || (attachment.controller == object.controller && self.is_aura(id))
                },
            )
    }
    fn attached(&self, source: ObjectRef, recipient: CardId) -> bool {
        self.attachment_link(source, self.reference(recipient))
            && self.kinds(source.card) & 1 == 0
            && ((self.is_aura(source.card)
                && aura_kind_matches(self, self.definition(source.card), recipient))
                || (self.is_equipment(source.card) && self.kinds(recipient) & 1 != 0))
    }
    fn live_applies(&self, source: ObjectRef, effect: &LiveStaticEffect, id: CardId) -> bool {
        let target = self.object(id);
        let provider = self.object(source.card);
        if target.zone != Zone::Battlefield
            || self.kinds(id) & 1 == 0
            || provider.zone != Zone::Battlefield
            || self.reference(source.card) != source
        {
            return false;
        }
        let member = match effect.recipients {
            LiveStatRecipients::Attached => provider.attached_to == Some(self.reference(id)),
            LiveStatRecipients::OtherColoredCreaturesYouControl(color) => {
                target.controller == provider.controller
                    && self.reference(id) != source
                    && self.colors(id) & (1 << color.index()) != 0
            }
            LiveStatRecipients::Creatures {
                relation,
                other,
                enchanted_by_controller,
            } => {
                (match relation {
                    0 => target.controller == provider.controller,
                    1 => target.controller != provider.controller,
                    _ => true,
                }) && (!other || self.reference(id) != source)
                    && (!enchanted_by_controller
                        || self.attachment_sources().iter().any(|&a| {
                            let aura = self.object(a);
                            aura.zone == Zone::Battlefield
                                && aura.card.aura().is_some()
                                && aura.controller == provider.controller
                                && aura.attached_to == Some(self.reference(id))
                        }))
            }
        };
        member
            && effect.condition.is_none_or(|condition| {
                let matching = target.controller == provider.controller
                    && (self.subtypes(id).has(target.card, condition.subtype));
                matching != condition.negated
            })
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CharacteristicWorld for Game {
    fn game(&self) -> &Game {
        self
    }
    fn object(&self, id: CardId) -> &Object {
        &self.state.objects[id as usize]
    }
    fn life(&self, player: Player) -> i128 {
        self.state.players[player.index()].life
    }
    fn gained_life(&self, player: Player) -> bool {
        self.state.players[player.index()].life_gained_this_turn > 0
    }
    fn attacking(&self, id: CardId) -> bool {
        self.state
            .attackers
            .iter()
            .any(|attacker| attacker.card == id)
    }
}

/// A cast proposal moves one exact object before sampling targets or costs.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) struct CastDepartureWorld<'a> {
    game: &'a Game,
    id: CardId,
    spell: Object,
    graveyard_owner: Option<Player>,
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    pub(super) fn cast_departure(
        &self,
        source: ObjectRef,
        controller: Player,
    ) -> Option<CastDepartureWorld<'_>> {
        let original = self.state.objects.get(source.card as usize)?;
        if original.generation != source.generation
            || !matches!(
                original.zone,
                Zone::Hand | Zone::Graveyard | Zone::Exile | Zone::Library
            )
        {
            return None;
        }
        let mut spell = original.clone();
        spell.zone = Zone::Stack;
        spell.controller = controller;
        Some(CastDepartureWorld {
            game: self,
            id: source.card,
            spell,
            graveyard_owner: (original.zone == Zone::Graveyard && original.is_card())
                .then_some(original.owner),
        })
    }
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CharacteristicWorld for CastDepartureWorld<'_> {
    fn game(&self) -> &Game {
        self.game
    }
    fn object(&self, id: CardId) -> &Object {
        if id == self.id {
            &self.spell
        } else {
            self.game.object(id)
        }
    }
    fn card_left_graveyard(&self, player: Player) -> bool {
        self.graveyard_owner == Some(player) || self.game.card_left_graveyard(player)
    }
    fn life(&self, player: Player) -> i128 {
        self.game.life(player)
    }
    fn gained_life(&self, player: Player) -> bool {
        self.game.gained_life(player)
    }
    fn attacking(&self, id: CardId) -> bool {
        self.game.attacking(id)
    }
}

/// An active provider for this player. Identical instances of protection are redundant.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn player_protection_source<W: CharacteristicWorld + ?Sized>(
    world: &W,
    player: Player,
) -> Option<CardId> {
    let grants_protection = |fragment: &crate::cards::StaticAbilityFragment| {
        matches!(
            fragment,
            crate::cards::StaticAbilityFragment::PlayerProtectionFromOpponents
        )
    };
    world.static_sources().iter().copied().find(|&id| {
        let object = world.object(id);
        object.zone == Zone::Battlefield
            && object.controller == player
            && object
                .card
                .static_ability_groups()
                .iter()
                .any(|group| group.fragments.iter().any(grants_protection))
            && characteristic_query::CharacteristicQuery::in_world(world)
                .continuous_fragment_available(id, grants_protection)
    })
}

/// Evaluate the printed Aura restriction in either live or projected worlds.
/// Shared live enchant restriction for actual attachment/SBA and speculative entry.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn aura_kind_matches<W: CharacteristicWorld + ?Sized>(
    world: &W,
    aura: Card,
    id: CardId,
) -> bool {
    let Some(spec) = aura.aura() else {
        return false;
    };
    let kinds = world.kinds(id);
    let kind_match = if spec.kinds == 0 {
        kinds & (1 | 2 | 16 | 32 | 64 | 256) != 0
    } else {
        kinds & u16::from(spec.kinds) != 0
            || aura
                .aura_kind_or_subtype()
                .is_some_and(|subtype| world.subtypes(id).has(world.definition(id), subtype))
    };
    kind_match
        && (!aura.enchant_basic() || world.definition(id).is_basic_land())
        && aura_artifact_or_non_aura_enchantment(world, aura, id)
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn aura_artifact_or_non_aura_enchantment<W: CharacteristicWorld + ?Sized>(
    world: &W,
    aura: Card,
    id: CardId,
) -> bool {
    if !aura.aura_artifact_or_non_aura_enchantment() {
        return true;
    }
    let kinds = world.kinds(id);
    kinds & 16 != 0 || (kinds & 32 != 0 && !world.subtypes(id).has(world.object(id).card, "Aura"))
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl ManaProduction {
    /// Evaluate against the actual game or a speculative funding overlay.
    pub(super) fn in_world<W: CharacteristicWorld + ?Sized>(
        self,
        world: &W,
        source: CardId,
    ) -> Self {
        self.in_world_for_controller(world, source, world.object(source).controller)
    }
    pub(super) fn in_world_for_controller<W: CharacteristicWorld + ?Sized>(
        self,
        world: &W,
        source: CardId,
        controller: Player,
    ) -> Self {
        match self {
            Self::Tagged { production, .. } => {
                production.in_world_for_controller(world, source, controller)
            }
            Self::TapSourcePowerChoice { mask } => {
                let amount = u32::try_from(power_after_source_tap(world, source).max(0))
                    .capacity("mana representation exhausted");
                if amount == 0 {
                    Self::Fixed([0; 6])
                } else {
                    Self::Choice { mask, amount }
                }
            }
            Self::SourcePower { color } => {
                let mut mana = [0; 6];
                mana[usize::from(color)] = u32::try_from(world.power(source).max(0))
                    .capacity("mana representation exhausted");
                Self::Fixed(mana)
            }
            Self::SourcePowerSplit { first, second } => {
                let power = if world.kinds(source) & 1 == 0 {
                    0
                } else {
                    world.power(source)
                };
                Self::split_for_power(first, second, power)
            }
            Self::AnyColorChangingSource(amount) => Self::any_color(amount),
            Self::ColorsAmongControlledPermanents => {
                let mask = self::controlled_colors(world, controller);
                Self::Fixed(std::array::from_fn(|i| {
                    u32::from(i < 5 && mask & (1 << i) != 0)
                }))
            }
            Self::ChoiceStat { colors, amount } => {
                let amount = world.game().stat_value_in_world(
                    world,
                    *amount,
                    world.reference(source),
                    controller,
                );
                let amount = u32::try_from(amount.max(0)).capacity("mana representation exhausted");
                if amount == 0 {
                    Self::Fixed([0; 6])
                } else {
                    Self::Choice {
                        mask: colors,
                        amount,
                    }
                }
            }
            Self::ChosenColor => {
                let mut mana = [0; 6];
                if let Some(EntryChoiceValue::Color(color)) =
                    world.entry_value(world.reference(source))
                {
                    mana[color.index()] = 1;
                }
                Self::Fixed(mana)
            }
            Self::DistinctCreaturePowers => {
                // Mana resolves after the source is tapped; power and even creature
                // status can depend on that tap in either the live or projected world.
                let mut object = world.object(source).clone();
                object.tapped = true;
                let tapped = SourceTappedWorld {
                    parent: world,
                    source,
                    object,
                };
                let mut powers = Vec::new();
                for id in 0..tapped.game().state.objects.len() {
                    let id = id as CardId;
                    let object = tapped.object(id);
                    if object.zone == Zone::Battlefield
                        && object.controller == controller
                        && tapped.kinds(id) & 1 != 0
                    {
                        powers.push(tapped.power(id));
                    }
                }
                powers.sort_unstable();
                powers.dedup();
                let amount = u32::try_from(powers.len()).capacity("mana representation exhausted");
                if amount == 0 {
                    Self::Fixed([0; 6])
                } else {
                    Self::any_color(amount)
                }
            }
            Self::MonocoloredPermanentColors => {
                let mut mask = 0u8;
                for id in 0..world.game().state.objects.len() {
                    let id = id as CardId;
                    let object = world.object(id);
                    if object.zone == Zone::Battlefield && object.controller == controller {
                        let colors = world.colors(id);
                        if colors.count_ones() == 1 {
                            mask |= colors;
                        }
                    }
                }
                let mut mana = [0; 6];
                for (index, amount) in mana.iter_mut().enumerate().take(5) {
                    *amount = u32::from(mask & (1 << index) != 0);
                }
                Self::Fixed(mana)
            }
            Self::CountSubtype { color, subtype } => {
                let count = world
                    .game()
                    .state
                    .objects
                    .iter()
                    .enumerate()
                    .filter(|(id, _)| {
                        let object = world.object(*id as CardId);
                        object.zone == Zone::Battlefield
                            && object.controller == controller
                            && (world.subtypes(*id as CardId).has(object.card, subtype))
                    })
                    .count();
                let mut mana = [0; 6];
                mana[usize::from(color)] =
                    u32::try_from(count).capacity("mana representation exhausted");
                Self::Fixed(mana)
            }
            value => value,
        }
    }
}

/// Only the source tap changes in this borrowed production world. Every other
/// fact, including virtual departures, copies, counters and entry projections,
/// is read from the parent world. No events are emitted by a feasibility query.
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
struct SourceTappedWorld<'a, W: CharacteristicWorld + ?Sized> {
    parent: &'a W,
    source: CardId,
    object: Object,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl<W: CharacteristicWorld + ?Sized> CharacteristicWorld for SourceTappedWorld<'_, W> {
    fn departed_source_power(&self, source: ObjectRef) -> Option<i128> {
        self.parent.departed_source_power(source)
    }
    fn departed_source_subtypes(&self, source: ObjectRef) -> Option<(Card, GrantedSubtypes)> {
        self.parent.departed_source_subtypes(source)
    }
    fn game(&self) -> &Game {
        self.parent.game()
    }
    fn battle_protector(&self, object: ObjectRef) -> Option<Player> {
        self.parent.battle_protector(object)
    }
    fn object(&self, id: CardId) -> &Object {
        if id == self.source {
            &self.object
        } else {
            self.parent.object(id)
        }
    }
    fn temporary_copy_view(&self, object: ObjectRef) -> Option<TemporaryCopyView> {
        self.parent.temporary_copy_view(object)
    }
    fn life(&self, p: Player) -> i128 {
        self.parent.life(p)
    }
    fn exhaust_activated_this_turn(&self, player: Player) -> bool {
        self.parent.exhaust_activated_this_turn(player)
    }
    fn has_enduring_story(&self, player: Player) -> bool {
        self.parent.has_enduring_story(player)
    }
    fn gained_life(&self, p: Player) -> bool {
        self.parent.gained_life(p)
    }
    fn lost_life(&self, p: Player) -> bool {
        self.parent.lost_life(p)
    }
    fn attacking(&self, id: CardId) -> bool {
        self.parent.attacking(id)
    }
    fn static_sources(&self) -> &[CardId] {
        self.parent.static_sources()
    }
    fn trigger_multiplier_sources(&self) -> &[CardId] {
        self.parent.trigger_multiplier_sources()
    }
    fn attachment_sources(&self) -> &[CardId] {
        self.parent.attachment_sources()
    }
    fn controls(&self) -> Option<&continuous_control::ControlState> {
        self.parent.controls()
    }
    fn roles(&self) -> Option<&roles::RoleState> {
        self.parent.roles()
    }
    fn entry_values(&self) -> &[entry_choices::ChoiceRow] {
        self.parent.entry_values()
    }
    fn stored_recipe(&self, id: CardId) -> Option<CopyRecipe> {
        self.parent.stored_recipe(id)
    }
    fn ability_origin(&self, id: CardId) -> Option<u64> {
        self.parent.ability_origin(id)
    }
    fn named_counter_rows(&self) -> &[live_counters::NamedCounterRow] {
        self.parent.named_counter_rows()
    }
    fn counter_amounts(&self, id: CardId) -> CounterAmounts<'_> {
        self.parent.counter_amounts(id)
    }
    fn creatures_died(&self) -> u64 {
        self.parent.creatures_died()
    }
    fn card_left_graveyard(&self, p: Player) -> bool {
        self.parent.card_left_graveyard(p)
    }
    fn plus_one_placed_this_turn(&self, object: ObjectRef, p: Player) -> bool {
        self.parent.plus_one_placed_this_turn(object, p)
    }
    fn graveyard_borrowing_state(&self) -> &graveyard_borrowing::Rows {
        self.parent.graveyard_borrowing_state()
    }
    fn borrowed_slots_state(&self) -> &borrowed_activations::SlotRows {
        self.parent.borrowed_slots_state()
    }
    fn borrowed_slots(&self) -> &[borrowed_activations::SlotRow] {
        self.parent.borrowed_slots()
    }
    fn borrowed_activation_used(&self, source: ObjectRef, slot: u64) -> bool {
        self.parent.borrowed_activation_used(source, slot)
    }
    fn entered_this_turn(&self, source: ObjectRef) -> bool {
        self.parent.entered_this_turn(source)
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
fn power_after_source_tap<W: CharacteristicWorld + ?Sized>(world: &W, source: CardId) -> i128 {
    let mut object = world.object(source).clone();
    object.tapped = true;
    SourceTappedWorld {
        parent: world,
        source,
        object,
    }
    .power(source)
}

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn controlled_colors<W: CharacteristicWorld + ?Sized>(world: &W, actor: Player) -> u8 {
    (0..world.game().state.objects.len()).fold(0, |mask, id| {
        let id = id as CardId;
        let object = world.object(id);
        mask | if object.zone == Zone::Battlefield && object.controller == actor {
            world.colors(id) & 31
        } else {
            0
        }
    })
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub(super) fn graveyard_subtype<W: CharacteristicWorld + ?Sized>(
    world: &W,
    actor: Player,
    subtype: &str,
) -> bool {
    (0..world.game().state.objects.len()).any(|id| {
        let id = id as CardId;
        let object = world.object(id);
        object.zone == Zone::Graveyard
            && object.owner == actor
            && object.is_card()
            && world.subtypes(id).has(object.card, subtype)
    })
}
