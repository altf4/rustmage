//! Borrowed earlier-layer facts shared by actual and speculative mana worlds.
//! The overlay decides presence, control, counters and life; no cached ability
//! status from the real game is carried across a virtual sacrifice or payment.
use super::*;
use crate::cards::{LiveStatRecipients, LiveStaticEffect};

pub(super) trait CharacteristicWorld {
    fn game(&self) -> &Game;
    fn spell_has_convoke(&self, actor: Player, definition: Card) -> bool {
        definition.has_convoke()
            || definition.is_creature()
                && self.game().state.static_sources.iter().any(|&id| {
                    let object = self.object(id);
                    object.zone == Zone::Battlefield
                        && object.controller == actor
                        && object.card.static_ability_groups().iter().any(|group| {
                            group.fragments.iter().any(|fragment| {
                                matches!(
                                    fragment,
                                    crate::cards::StaticAbilityFragment::ConvokeCreatureSpells
                                )
                            })
                        })
                        && self.printed_abilities_available(id)
                })
    }
    fn object(&self, id: CardId) -> &Object;
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
    fn controlled_colors(&self, controller: Player) -> u8 {
        (0..self.game().state.objects.len())
            .map(|id| id as CardId)
            .filter(|&id| {
                self.object(id).zone == Zone::Battlefield
                    && self.object(id).controller == controller
            })
            .fold(0u8, |colors, id| colors | self.colors(id))
    }
    fn colors(&self, id: CardId) -> u8 {
        characteristic_layers::colors(self, id)
    }

    fn is_token(&self, id: CardId) -> bool {
        self.object(id).is_token()
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

    fn can_receive_counters(&self, id: CardId) -> bool {
        !self.game().state.attachment_sources.iter().any(|&source| {
            let provider = self.object(source);
            provider.zone == Zone::Battlefield
                && provider.card.static_ability_groups().iter().any(|group| {
                    group.fragments.iter().any(|fragment| {
                        matches!(
                            fragment,
                            crate::cards::StaticAbilityFragment::AttachedCounterProhibition
                        )
                    })
                })
                && self.attached(self.reference(source), id)
                && self.printed_abilities_available(source)
        })
    }
    fn activation_counter_cost_allowed(
        &self,
        source: CardId,
        definition: Card,
        index: u16,
        ability: &ActivatedAbility,
    ) -> bool {
        !(ability.cost.loyalty.is_some_and(|amount| amount > 0)
            || definition
                .activation_put_counter(index)
                .is_some_and(|(_, amount)| amount > 0))
            || (self.object(source).zone == ability.source_zone()
                && self.can_receive_counters(source))
    }

    fn graveyard_has_subtype(&self, actor: Player, subtype: &'static str) -> bool {
        (0..self.game().state.objects.len()).any(|index| {
            let id = index as CardId;
            let object = self.object(id);
            object.zone == Zone::Graveyard
                && object.owner == actor
                && !object.is_token()
                && self.subtypes(id).has(object.card, subtype)
        })
    }
    fn chosen_creature_type_applies(&self, source: ObjectRef, id: CardId) -> bool {
        let provider = self.object(source.card);
        let target = self.object(id);
        (provider.zone == Zone::Battlefield
            || (provider.zone == Zone::Command && provider.card.is_emblem()))
            && self.reference(source.card) == source
            && target.zone == Zone::Battlefield
            && self.kinds(id) & 1 != 0
            && provider.controller == target.controller
            && matches!(self.game().entry_value(source), Some(EntryChoiceValue::CreatureType(kind)) if self.subtypes(id).has(target.card, kind.name()))
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
        self.game().state.roles.as_ref().is_some_and(|state| {
            state.ability_removal_sources.iter().any(|&source| {
                self.object(source.card).zone == Zone::Battlefield
                    && self.reference(source.card) == source
            }) || state.ability_removal_effects.iter().any(|row| {
                self.object(row.recipient.card).zone == Zone::Battlefield
                    && self.reference(row.recipient.card) == row.recipient
                    && row
                        .expires_turn
                        .is_none_or(|turn| turn == self.game().state.turn)
            })
        })
    }
    fn printed_abilities_available(&self, id: CardId) -> bool {
        // Exact attached or resolution-created removal can remove printed abilities.
        // Unrelated sources need no whole-world layer-six evaluation. Keep the
        // full evaluator for candidates: competing effects may blank a provider.
        let recipient = self.reference(id);
        !self.game().state.roles.as_ref().is_some_and(|state| {
            state
                .ability_removal_sources
                .iter()
                .any(|&source| self.attachment_link(source, recipient))
                || state.ability_removal_effects.iter().any(|row| {
                    row.recipient == recipient
                        && self.object(id).zone == Zone::Battlefield
                        && row
                            .expires_turn
                            .is_none_or(|turn| turn == self.game().state.turn)
                })
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
        self.game().granted_subtypes_with(
            id,
            self.object(id),
            |source| Some(self.object(source)),
            |source, kind| self.counter_count(source, kind),
            |id, _| self.kinds(id),
            characteristic_layers::subtype_changes(self, id),
        )
    }
    fn value(&self, value: StatValue, source: CardId) -> i128 {
        let game = self.game();
        let controller = self.object(source).controller;
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
            &|id| self.attacking(id),
        )
    }
    /// Total intrinsic triggered occurrences in this actual/speculative world.
    /// Callers capture the number before costs; mana triggers resolve immediately.
    fn trigger_occurrences(&self, source: CardId, upper: bool) -> u64 {
        let object = self.object(source);
        if object.zone != Zone::Battlefield {
            return 1;
        }
        let subtypes = (!upper).then(|| self.subtypes(source));
        let mut amount = 1u64;
        for &id in &self.game().state.static_sources {
            let provider = self.object(id);
            if provider.zone != Zone::Battlefield
                || (!upper
                    && (provider.controller != object.controller
                        || !self.printed_abilities_available(id)))
            {
                continue;
            }
            for fragment in provider
                .card
                .static_ability_groups()
                .iter()
                .flat_map(|g| g.fragments)
            {
                if let crate::cards::StaticAbilityFragment::AdditionalSubtypeTriggers {
                    subtype,
                    other,
                } = *fragment
                    && (!other || self.reference(id) != self.reference(source))
                    && subtypes.is_none_or(|types| types.has(object.card, subtype))
                {
                    amount = amount
                        .checked_add(1)
                        .capacity("trigger multiplicity exhausted");
                }
            }
        }
        amount
    }
    /// Capture before the tap costs remove the land or its attached providers.
    /// The upper bound intentionally ignores ability removal and current types:
    /// sacrificing another provider can restore either during a funding branch.
    fn mana_tap_bonus(&self, id: CardId, upper: bool) -> Mana {
        let mut result = [0u32; 6];
        if self.object(id).zone != Zone::Battlefield || (!upper && self.kinds(id) & 2 == 0) {
            return result;
        }
        for &source in &self.game().state.attachment_sources {
            let reference = self.reference(source);
            if !self.attachment_link(reference, self.reference(id))
                || (!upper && !self.printed_abilities_available(source))
            {
                continue;
            }
            for group in self.object(source).card.static_ability_groups() {
                for fragment in group.fragments {
                    if let crate::cards::StaticAbilityFragment::AttachmentManaBonus(production) =
                        *fragment
                    {
                        let mut bonus = [0; 6];
                        production.in_world(self, source).visit(|mana| {
                            for index in 0..6 {
                                bonus[index] = bonus[index].max(mana[index]);
                            }
                            false
                        });
                        let occurrences = self.trigger_occurrences(source, upper);
                        for index in 0..6 {
                            let multiplied = u64::from(bonus[index])
                                .checked_mul(occurrences)
                                .and_then(|n| u32::try_from(n).ok())
                                .capacity("mana representation exhausted");
                            result[index] = result[index]
                                .checked_add(multiplied)
                                .capacity("mana representation exhausted");
                        }
                    }
                }
            }
        }
        result
    }
    /// The ability belongs to its provider, even when the tapped land belongs
    /// to an opponent. Capture multiplicity before any activation costs depart.
    fn basic_land_tap_mana_bonus(&self, id: CardId, upper: bool) -> u32 {
        let object = self.object(id);
        if object.zone != Zone::Battlefield
            || !object.card.is_basic_land()
            || (!upper && self.kinds(id) & 2 == 0)
        {
            return 0;
        }
        let mut result = 0u32;
        for &source in &self.game().state.static_sources {
            let provider = self.object(source);
            if provider.zone != Zone::Battlefield
                || (!upper && !self.printed_abilities_available(source))
            {
                continue;
            }
            for fragment in provider
                .card
                .static_ability_groups()
                .iter()
                .flat_map(|g| g.fragments)
            {
                if let crate::cards::StaticAbilityFragment::BasicLandTapManaBonus(amount) =
                    *fragment
                {
                    let amount = u64::from(amount)
                        .checked_mul(self.trigger_occurrences(source, upper))
                        .and_then(|n| u32::try_from(n).ok())
                        .capacity("mana representation exhausted");
                    result = result
                        .checked_add(amount)
                        .capacity("mana representation exhausted");
                }
            }
        }
        result
    }
    fn granted_mana(&self, id: CardId) -> Vec<ability_order::GrantedManaAbility> {
        // Animation grants only keywords. It cannot remove a provider's printed
        // ability or change a mana grant, so only removal needs layer-six order.
        if self.has_ability_removal() {
            let evaluation = ability_evaluation::evaluate_abilities(self);
            return evaluation.result.objects[id as usize].mana.clone();
        }
        let mut result = Vec::new();
        for &source in &self.game().state.attachment_sources {
            let reference = self.reference(source);
            if self.kinds(source) & 1 != 0 || !self.attachment_link(reference, self.reference(id)) {
                continue;
            }
            for group in self.object(source).card.static_ability_groups() {
                for fragment in group.fragments {
                    if let crate::cards::StaticAbilityFragment::AttachmentMana(production) =
                        *fragment
                    {
                        result.push(ability_order::GrantedManaAbility {
                            source: reference,
                            production,
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
                        || self.game().state.attachment_sources.iter().any(|&a| {
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
            Self::AnyColorChangingSource(amount) => Self::any_color(amount),
            Self::ChosenColor => {
                let mut mana = [0; 6];
                if let Some(EntryChoiceValue::Color(color)) =
                    world.game().entry_value(world.reference(source))
                {
                    mana[color.index()] = 1;
                }
                Self::Fixed(mana)
            }
            Self::ColorsAmongControlledPermanents => {
                let controller = world.object(source).controller;
                let colors = (0..world.game().state.objects.len())
                    .map(|id| id as CardId)
                    .filter(|&id| {
                        let object = world.object(id);
                        object.zone == Zone::Battlefield && object.controller == controller
                    })
                    .fold(0u8, |colors, id| colors | world.colors(id));
                Self::Fixed(std::array::from_fn(|index| {
                    u32::from(index < 5 && colors & (1 << index) != 0)
                }))
            }
            Self::ChoiceStat { colors, amount } => Self::Choice {
                mask: colors,
                amount: u32::try_from(world.value(*amount, source).max(0))
                    .capacity("mana representation exhausted"),
            },
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
