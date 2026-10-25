//! Borrowed earlier-layer facts shared by actual and speculative mana worlds.
//! The overlay decides presence, control, counters and life; no cached ability
//! status from the real game is carried across a virtual sacrifice or payment.
use super::*;
use crate::cards::{LiveStatRecipients, LiveStaticEffect};

pub(super) trait CharacteristicWorld {
    fn power(&self, id: CardId) -> i128 {
        characteristic_query::CharacteristicQuery::in_world(self)
            .stats(id)
            .power
    }
    fn game(&self) -> &Game;
    fn static_sources(&self) -> &[CardId] {
        &self.game().state.static_sources
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
        // A copied linked ability does not inherit choices made for an earlier
        // ability instance, even when its printed definition is identical.
        self.ability_origin(object.card)
            .is_none()
            .then(|| self.raw_entry_value(object))
            .flatten()
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
    fn stored_recipe(&self, id: CardId) -> Option<CopyRecipe> {
        self.game().copy_recipe(self.reference(id))
    }
    fn is_aura(&self, id: CardId) -> bool {
        self.kinds(id) & 32 != 0 && self.subtypes(id).has(self.definition(id), "Aura")
    }
    fn is_equipment(&self, id: CardId) -> bool {
        self.kinds(id) & 16 != 0 && self.subtypes(id).has(self.definition(id), "Equipment")
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
    fn copied_keywords(&self, id: CardId) -> Keywords {
        let recipe = self.recipe(id);
        let keywords = recipe
            .overrides
            .map_or(Keywords::NONE, |value| value.keywords);
        if recipe.exceptions.sacrifice_abilities != 0 {
            keywords.union(Keywords::HASTE)
        } else {
            keywords
        }
    }
    fn legendary(&self, id: CardId) -> bool {
        self.recipe(id)
            .overrides
            .and_then(|value| value.legendary)
            .unwrap_or_else(|| self.definition(id).is_legendary())
    }
    fn intrinsic_mana_color(&self, id: CardId) -> Option<usize> {
        let card = self.definition(id);
        let color = card.mana_color()?;
        // CR305.6 belongs to the current basic land subtype. The same shortcut
        // also represents printed mana on Llanowar Elves, which survives losing Elf.
        if card.is_basic_land()
            && (self.kinds(id) & 2 == 0
                || !self.subtypes(id).has(
                    card,
                    ["Plains", "Island", "Swamp", "Mountain", "Forest"][color],
                ))
        {
            None
        } else {
            Some(color)
        }
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
    fn attacking(&self, id: CardId) -> bool;

    /// Effective string name in this actual or speculative object world.
    /// Name-changing effects belong here, before later characteristic layers.
    fn name(&self, id: CardId) -> &'static str {
        characteristic_layers::name(self, id)
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
    fn activation_prohibited(&self, actor: Player, id: CardId) -> bool {
        let target = self.object(id);
        target.zone == Zone::Battlefield
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
        if object.zone != Zone::Battlefield || self.kinds(id) & 1 == 0 {
            return Keywords::NONE;
        }
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
                    && self.counter_count(id, kind) >= minimum
                {
                    result = result.union(keywords);
                }
            }
        }
        result
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
        self.roles().is_some_and(|state| {
            state.ability_removal_sources.iter().any(|&source| {
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
        let Some(evaluation) = ability_evaluation::evaluate_if_needed(self) else {
            return true;
        };
        self.object(id)
            .card
            .static_ability_groups()
            .iter()
            .enumerate()
            .any(|(index, group)| {
                group.fragments.iter().any(&matches)
                    && evaluation.group_available(self.reference(id), index)
            })
    }
    fn printed_abilities_available(&self, id: CardId) -> bool {
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
        let game = self.game();
        let controller = self.object(source).controller;
        if value == StatValue::CreaturesDiedThisTurn {
            return i128::from(self.creatures_died());
        }
        if value == StatValue::CardLeftGraveyardThisTurn {
            return i128::from(self.card_left_graveyard(controller));
        }
        game.stat_value_with(
            value,
            self.reference(source),
            controller,
            &|| {
                game.state.objects.iter().enumerate().map(|(id, _)| {
                    let id = id as CardId;
                    (id, self.object(id))
                })
            },
            &|id| self.subtypes(id),
            &|id, _| self.kinds(id),
            &|id| self.name(id),
            &|id| self.colors(id),
            &|id| self.counter_amounts(id).has_any(),
            self.life(controller),
            game.state.attack_declared && self.attacking(source),
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
    fn land_mana_applies(
        &self,
        source: ObjectRef,
        id: CardId,
        recipients: crate::cards::LandManaRecipients,
    ) -> bool {
        let provider = self.object(source.card);
        let target = self.object(id);
        provider.zone == Zone::Battlefield
            && self.reference(source.card) == source
            && target.zone == Zone::Battlefield
            && self.kinds(id) & 2 != 0
            && match recipients {
                crate::cards::LandManaRecipients::YouControl => {
                    provider.controller == target.controller
                }
                crate::cards::LandManaRecipients::SourceChosenName => {
                    matches!(self.entry_value(source), Some(EntryChoiceValue::CardName(name)) if name.matches(self.name(id)))
                }
            }
    }
    fn granted_mana(&self, id: CardId) -> Vec<ability_order::GrantedManaAbility> {
        // Animation grants only keywords. It cannot remove a provider's printed
        // ability or change a mana grant, so only removal needs layer-six order.
        if self.has_ability_removal() {
            let evaluation = ability_evaluation::evaluate_abilities(self);
            return evaluation.result.objects[id as usize].mana.clone();
        }
        let mut result = Vec::new();
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
                    if let crate::cards::StaticAbilityFragment::LandMana {
                        recipients,
                        production,
                    } = *fragment
                        && self.land_mana_applies(self.reference(source), id, recipients)
                    {
                        result.push(ability_order::GrantedManaAbility {
                            source: self.reference(source),
                            production,
                            sacrifice: false,
                        });
                    }
                }
            }
        }
        result
    }
    fn attachment_link(&self, source: ObjectRef, recipient: ObjectRef) -> bool {
        let object = self.object(source.card);
        object.zone == Zone::Battlefield
            && self.reference(source.card) == source
            && self.object(recipient.card).zone == Zone::Battlefield
            && self.reference(recipient.card) == recipient
            && object.attached_to == Some(recipient)
    }
    fn attached(&self, source: ObjectRef, recipient: CardId) -> bool {
        self.attachment_link(source, self.reference(recipient))
            && (self.is_aura(source.card) || self.is_equipment(source.card))
            && self.kinds(source.card) & 1 == 0
            && self.kinds(recipient) & 1 != 0
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

impl ManaProduction {
    /// Evaluate against the actual game or a speculative funding overlay.
    pub(super) fn in_world<W: CharacteristicWorld + ?Sized>(
        self,
        world: &W,
        source: CardId,
    ) -> Self {
        match self {
            Self::Tagged { production, .. } => production.in_world(world, source),
            Self::SourcePower { color } => {
                let mut mana = [0; 6];
                mana[usize::from(color)] = u32::try_from(world.power(source).max(0))
                    .capacity("mana representation exhausted");
                Self::Fixed(mana)
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
            Self::MonocoloredPermanentColors => {
                let controller = world.object(source).controller;
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
                let controller = world.object(source).controller;
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
