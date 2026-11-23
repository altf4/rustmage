//! Necessary mana conditions at priority. The quoted capacity is deliberately
//! generous: spending restrictions, life costs and many activation guards can
//! reduce the available mana. Unbounded or unrecognized funding paths abstain.
use super::*;
use crate::cards::{
    ActivationGuard, ActivationOrigin, BasicLandType, EntryChoiceValue, ManaRestriction,
    StaticAbilityFragment as F,
};

#[derive(Clone, Copy, Default)]
struct Source {
    colors: u8,
    maximum: u64,
}

impl Source {
    fn mode(&mut self, colors: u8, amount: u64) {
        if amount != 0 {
            self.colors |= colors;
            self.maximum = self.maximum.max(amount);
        }
    }

    fn production(&mut self, production: ManaProduction) -> Option<()> {
        match production.base() {
            ManaProduction::Fixed(mana) => {
                let mut mask = 0;
                for (index, &amount) in mana.iter().enumerate() {
                    if amount != 0 {
                        mask |= 1 << index;
                    }
                }
                self.mode(mask, mana.into_iter().map(u64::from).sum());
            }
            ManaProduction::Choice { mask, amount }
            | ManaProduction::Combination { mask, amount } => {
                self.mode(mask, u64::from(amount));
            }
            ManaProduction::ChosenColor => self.mode(31, 1),
            ManaProduction::ChosenColorAmount(amount) => self.mode(31, u64::from(amount)),
            _ => return None,
        }
        Some(())
    }
}

// Each view excludes only outputs known to be unusable for a particular
// spell purpose. A filter land still contributes its whole output, even if
// restricted mana would be needed to activate it. Floating mana stays generous.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SpellRestriction {
    ActivationOnly,
    CastFromExile,
    CreatureSpell,
    InstantSorcery,
}
impl SpellRestriction {
    const ALL: [Self; 4] = [
        Self::ActivationOnly,
        Self::CastFromExile,
        Self::CreatureSpell,
        Self::InstantSorcery,
    ];

    fn of(production: ManaProduction) -> Option<Self> {
        let ManaProduction::Tagged { quality, .. } = production else {
            return None;
        };
        match quality.restriction {
            ManaRestriction::AbilityActivation
            | ManaRestriction::ArtifactActivation
            | ManaRestriction::CreatureActivation
            | ManaRestriction::LandActivation
            | ManaRestriction::TurnFaceUp => Some(Self::ActivationOnly),
            ManaRestriction::CastFromExile => Some(Self::CastFromExile),
            ManaRestriction::CreatureSpells
            | ManaRestriction::ChosenCreatureSpell
            | ManaRestriction::ChosenCreatureSpellOrActivation
            | ManaRestriction::CreatureSpellManaValueAtLeastOrX(_) => Some(Self::CreatureSpell),
            ManaRestriction::InstantSorcery => Some(Self::InstantSorcery),
            _ => None,
        }
    }

    fn excludes(self, definition: Card, origin: Zone) -> bool {
        match self {
            Self::ActivationOnly => true,
            Self::CastFromExile => origin != Zone::Exile,
            Self::CreatureSpell => !definition.is_creature(),
            Self::InstantSorcery => definition.kind_mask() & (4 | 8) == 0,
        }
    }
}

pub(super) struct PaymentCapacity {
    // A subset counts every physical source that could produce any of its colors once.
    // Generously grant that source's whole largest output to the subset.
    subset: [u128; 64],
    sources: Vec<Source>,
    restrictions: Vec<(SpellRestriction, [u128; 64])>,
}

impl PaymentCapacity {
    pub(super) fn for_player(game: &Game, actor: Player) -> Option<Self> {
        let s = &game.state;
        if !s.exile_links.is_empty()
            || s.borrowed_links.is_some()
            || s.graveyard_borrowing.is_some()
            || s.self_granted_abilities.is_some()
            || s.temporary_abilities.is_some()
            || s.controls.is_some()
            || s.temporary_controls.is_some()
            || s.copy_recipes.is_some()
            || s.temporary_copies.is_some()
            || s.manifest.as_ref().is_some_and(|m| !m.permanents.is_empty())
            // Earthbend only queues a delayed return; it cannot resolve while paying.
            // The zone-rules cache may be preallocated because of a card in an
            // unseen library. Check the live public providers below instead.
            || s.replacements.is_some()
            || s.players[actor.index()].mountain_mana_bonus != 0
            || s.roles.as_ref().is_some_and(|r| {
                !r.resolved_ability_modifications.is_empty()
                    || r.source_linked_effects.iter().any(|row| !matches!(row.kind,
                        SourceLinkedEffectKind::RemoveAbilities
                        | SourceLinkedEffectKind::RemoveAbilitiesUntilEndOfTurn
                        | SourceLinkedEffectKind::RemoveAbilitiesUntilNextTurn
                        | SourceLinkedEffectKind::PreventActivationsUntilUntapped
                        | SourceLinkedEffectKind::PreventUntap | SourceLinkedEffectKind::GrantHexproof
                        | SourceLinkedEffectKind::PreventDamageBy))
                    || !r.basic_land_types.is_empty()
                    // These forms change creature characteristics, never a land's
                    // intrinsic mana. Abort on newly acquired land characteristics.
                    || r.animations.iter().any(|row| {
                        ((row.land || row.additional_card_types & 2 != 0)
                            && !s.objects[row.object.card as usize].card.is_land())
                            || row.creature_types.is_some_and(|types| types.iter().any(|name| {
                                BasicLandType::ALL.into_iter().any(|kind| kind.name() == *name)
                            }))
                    })
                    || !r.melded_permanents.is_empty()
                    || !r.ability_removal_sources.is_empty()
                    || r.control_change_sources.is_some()
            })
        {
            return None;
        }

        let mut chosen = vec![0u8; s.objects.len()];
        if let Some(rows) = &s.entry_values {
            for row in rows.iter() {
                if let EntryChoiceValue::BasicLandType(kind) = row.value
                    && game.object_ref(row.object.card) == row.object
                {
                    chosen[row.object.card as usize] |= 1 << kind.index();
                }
            }
        }

        // Only prove sickness where neither the creature type nor permission to
        // tap can change during payment. Layered worlds keep all sick sources.
        let mut fixed_creature_taps = s.attachment_sources.is_empty()
            && s.named_counters.as_ref().is_none_or(|rows| {
                rows.iter().all(|row| {
                    let reference = row.object();
                    let object = &s.objects[reference.card as usize];
                    object.zone != Zone::Battlefield
                        || object.controller != actor
                        || game.object_ref(reference.card) != reference
                        || row.count(CounterKind::Haste) == 0
                })
            })
            && s.roles.as_ref().is_none_or(|r| {
                r.characteristic_sources.is_empty()
                    && r.animations.is_empty()
                    && r.keyword_effects.is_empty()
                    && r.live_static.is_empty()
            });
        let mut land_types = 0u8;
        for (id, object) in s.objects.iter().enumerate() {
            if !matches!(object.zone, Zone::Battlefield | Zone::Command) {
                continue;
            }
            if object.card.zone_replacement().is_some() {
                return None;
            }
            fixed_creature_taps &= !object
                .card
                .keyword_grants()
                .iter()
                .any(|grant| grant.keywords.contains(Keywords::HASTE))
                && !object
                    .card
                    .counter_keyword_grants()
                    .iter()
                    .any(|(_, grant)| grant.contains(Keywords::HASTE));
            for group in object.card.static_ability_groups() {
                for &fragment in group.fragments {
                    // Restrict the sickness proof to providers with no ability to
                    // grant keywords or activation-as-haste, under any condition.
                    fixed_creature_taps &= match fragment {
                        // An intrinsic keyword can only grant haste to its own source;
                        // the individual tap source checks its own keywords below.
                        F::IntrinsicKeywords(_) => true,
                        F::SourceConditionalKeywords { keywords, .. }
                        | F::SourceCounterCreature { keywords, .. }
                        | F::ControlledPermanentKeywords { keywords, .. } => {
                            !keywords.contains(Keywords::HASTE)
                        }
                        F::CostReduction(..)
                        | F::PlotCostReduction(..)
                        | F::ChosenBasicLandType
                        | F::BaseStatSetter(..)
                        | F::CannotBeBlockedByCreaturesWithLessPower
                        | F::StaticCastRestriction(..)
                        | F::AdditionalLandPlays(_)
                        | F::PlayLandFromGraveyard
                        | F::HarnessedTrigger(_)
                        | F::EntryTriggerSuppression(..)
                        | F::DeathBaronLord
                        | F::StaticModifier(..)
                        | F::ScalingModifier(..)
                        | F::CreatureGroupBonus { .. }
                        | F::CreatureGroupScaling { .. }
                        | F::ControlledCreatureBonusPerSubtype { .. }
                        | F::ControlledLegendaryCreatureBonus(_)
                        | F::ControlledCreatureScaling { .. }
                        | F::StartYourEngines => true,
                        _ => false,
                    };
                    // These effects cannot create, duplicate or untap a resource during
                    // funding. Mana-dependent stat recipes are intentionally unsupported.
                    if !matches!(
                        fragment,
                        F::CostReduction(..)
                            | F::PlotCostReduction(..)
                            | F::ChosenBasicLandType
                            | F::DeathBaronLord
                            | F::StaticModifier(..)
                            | F::ScalingModifier(..)
                            | F::IntrinsicKeywords(_)
                            | F::ControlledCreatureScaling { .. }
                            | F::ControlledLegendaryCreatureBonus(_)
                            | F::ControlledCreatureBonusPerSubtype { .. }
                            | F::CreatureGroupBonus { .. }
                            | F::CreatureGroupScaling { .. }
                            | F::SourceConditionalKeywords { .. }
                            | F::StartYourEngines
                            | F::CannotBeBlockedByCreaturesWithLessPower
                            | F::StaticCastRestriction(..)
                            | F::EntryTriggerSuppression(..)
                            | F::HarnessedTrigger(_)
                            | F::AdditionalLandPlays(_)
                            | F::PlayLandFromGraveyard
                            | F::SourceCounterCreature { .. }
                            | F::BaseStatSetter(..)
                            | F::ControlledPermanentKeywords { .. }
                    ) {
                        return None;
                    }
                }
            }
            if object.zone == Zone::Battlefield
                && object.controller == actor
                && object.card.is_land()
            {
                // ControlSubtypeAny can look at tapped lands as well.
                for color in object.card.printed_intrinsic_mana_colors() {
                    land_types |= 1 << color;
                }
                land_types |= chosen[id];
            }
        }

        let mut sources = vec![Source::default(); s.objects.len()];
        let mut histogram = [0u128; 64];
        let mut spell_histograms = [[0u128; 64]; SpellRestriction::ALL.len()];
        let mut has_restriction = [false; SpellRestriction::ALL.len()];
        for (id, object) in s.objects.iter().enumerate() {
            let controlled = object.zone == Zone::Battlefield && object.controller == actor;
            if !(controlled
                || (object.owner == actor
                    && matches!(
                        object.zone,
                        Zone::Battlefield | Zone::Hand | Zone::Graveyard
                    )))
            {
                // Never consult the identity of an opposing hand or library card.
                continue;
            }
            // Returning a double-faced card to hand or sacrificing it restores the
            // physical front. Its zone-activated mana rules can differ from this face.
            if object.owner == actor && object.zone == Zone::Battlefield && object.is_card() {
                let front = object
                    .card
                    .room_parent()
                    .or_else(|| object.card.split_parent())
                    .unwrap_or_else(|| object.card.front_face());
                if front != object.card
                    && front.activations().iter().any(|ability| {
                        ability.origin != ActivationOrigin::Battlefield
                            && ability.mana_production().is_some()
                    })
                {
                    return None;
                }
            }
            let source = &mut sources[id];
            let sick = fixed_creature_taps
                && object.summoning_sick
                && !object.cast_for_impending
                && object.card.is_creature()
                && object.card.static_ability_groups().iter().all(|group| group.fragments.iter().all(|f| {
                    matches!(*f, F::IntrinsicKeywords(keywords) if !keywords.contains(Keywords::HASTE))
                }))
                && !object
                    .card
                    .keywords()
                    .union(object.gained_keywords)
                    .contains(Keywords::HASTE);
            let can_tap = !object.tapped && !sick;
            if controlled && can_tap {
                for color in object
                    .card
                    .printed_intrinsic_mana_colors()
                    .chain(object.card.mana_color())
                {
                    source.mode(1 << color, 1);
                }
                source.mode(chosen[id], u64::from(chosen[id] != 0));
            }
            let mut spell_sources = [*source; SpellRestriction::ALL.len()];
            for ability in object.card.activations() {
                let Some(production) = ability.mana_production() else {
                    continue;
                };
                if ability.origin != ActivationOrigin::Battlefield {
                    // Payment costs can move an owned permanent to the hand or graveyard.
                    return None;
                }
                if !controlled {
                    continue;
                }
                // A tapped permanent can still activate a non-tap mana ability, e.g.
                // sacrificing an Eldrazi Spawn. Its production needs a different bound.
                if !ability.cost.tap {
                    return None;
                }
                if !can_tap {
                    continue;
                }
                // Extra prerequisites only reduce capacity. Effects beyond producing mana
                // can change what other sources may do during the payment window.
                if !ability.cost.alternatives.is_empty()
                    || !matches!(ability.program, [Instruction::AddMana(_)])
                    || ability.cost.additional.is_some_and(|cost| {
                        !matches!(cost.kind, EffectChoiceKind::TapPermanents(_))
                    })
                {
                    return None;
                }
                if let ActivationGuard::ControlSubtypeAny(types) = ability.guard
                    && !types.iter().any(|subtype| {
                        BasicLandType::ALL
                            .into_iter()
                            .find(|kind| kind.name() == *subtype)
                            .is_none_or(|kind| land_types & (1 << kind.index()) != 0)
                    })
                {
                    continue;
                }
                let mut mode = Source::default();
                mode.production(production)?;
                source.mode(mode.colors, mode.maximum);
                let restriction = SpellRestriction::of(production);
                for (i, &class) in SpellRestriction::ALL.iter().enumerate() {
                    if restriction == Some(class) {
                        has_restriction[i] = true;
                    } else {
                        spell_sources[i].mode(mode.colors, mode.maximum);
                    }
                }
            }
            histogram[source.colors as usize] += u128::from(source.maximum);
            for (histogram, source) in spell_histograms.iter_mut().zip(spell_sources) {
                histogram[source.colors as usize] += u128::from(source.maximum);
            }
        }

        let mut pool = s.players[actor.index()].mana.map(u64::from);
        pool[3] += u64::from(s.players[actor.index()].combat_red_mana);
        let subset = Self::subsets(histogram, pool);
        let restrictions = SpellRestriction::ALL
            .into_iter()
            .zip(spell_histograms)
            .zip(has_restriction)
            .filter(|&(_, active)| active)
            .map(|((kind, histogram), _)| (kind, Self::subsets(histogram, pool)))
            .collect();
        Some(Self {
            subset,
            sources,
            restrictions,
        })
    }

    fn subsets(mut histogram: [u128; 64], pool: [u64; 6]) -> [u128; 64] {
        let mut subset = [0u128; 64];
        // Six colors give a fixed-size subset-sum transform, independent of board size.
        for bit in 0..6 {
            for mask in 0..64 {
                if mask & (1 << bit) != 0 {
                    histogram[mask] += histogram[mask ^ (1 << bit)];
                    subset[mask] += u128::from(pool[bit]);
                }
            }
        }
        for (mask, upper) in subset.iter_mut().enumerate() {
            *upper += histogram[63] - histogram[63 ^ mask];
        }
        subset
    }

    pub(super) fn permits_spell(
        &self,
        colored: Mana,
        generic: u32,
        definition: Card,
        origin: Zone,
    ) -> bool {
        self.permits(colored, generic, None)
            && self.restrictions.iter().all(|(kind, subset)| {
                !kind.excludes(definition, origin)
                    || Self::covers(subset, colored, generic, Source::default())
            })
    }

    pub(super) fn permits(
        &self,
        colored: Mana,
        generic: u32,
        reserved_tap: Option<CardId>,
    ) -> bool {
        let reserved = reserved_tap.map_or(Source::default(), |id| self.sources[id as usize]);
        Self::covers(&self.subset, colored, generic, reserved)
    }

    fn covers(subset: &[u128; 64], colored: Mana, generic: u32, reserved: Source) -> bool {
        let upper = |mask: usize| {
            subset[mask]
                - if usize::from(reserved.colors) & mask != 0 {
                    u128::from(reserved.maximum)
                } else {
                    0
                }
        };
        let mut colors = 0;
        let mut total = u128::from(generic);
        for (bit, &amount) in colored.iter().enumerate() {
            total += u128::from(amount);
            if amount != 0 {
                colors |= 1usize << bit;
            }
        }
        if total > upper(63) {
            return false;
        }
        let mut mask = colors;
        while mask != 0 {
            let needed: u128 = colored
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, amount)| u128::from(*amount))
                .sum();
            if needed > upper(mask) {
                return false;
            }
            mask = (mask - 1) & colors;
        }
        true
    }
}
