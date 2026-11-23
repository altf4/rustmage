//! Necessary mana conditions at priority. The quoted capacity is deliberately
//! generous: spending restrictions, life costs and many activation guards can
//! reduce the available mana. Unbounded or unrecognized funding paths abstain.
use super::*;
use crate::cards::{
    ActivationGuard, ActivationOrigin, BATTLE_KIND, BasicLandType, EntryChoiceValue,
    ManaRestriction, StaticAbilityFragment as F,
};
use mana_lots::{ManaRestrictionView as LotRestriction, Purpose, Quality};
use std::collections::HashMap;

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
// restricted mana would be needed to activate it.
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

/// Aggregate only restrictions whose spell-purpose inputs can be bounded here.
fn floating_spell_restrictions(game: &Game, actor: Player) -> Vec<(LotRestriction, [u128; 6])> {
    use LotRestriction::*;

    let mut totals = HashMap::<_, [u128; 6]>::new();
    for lot in game.mana_lots(actor) {
        let rule = match lot.quality.restriction {
            CreatureSpellOrActivation(kind) => CreatureSpell(kind),
            rule @ (AbilityActivation
            | ArtifactActivation
            | CreatureActivation
            | LandActivation
            | TurnFaceUp
            | CastFromExile
            | CreatureSpells
            | InstantSorcery
            | NoncreatureSpell
            | ArtifactSpell
            | ArtifactSpellOrActivation
            | ArtifactSpellOrArtifactActivation
            | ExceptNonArtifactSpells
            | PlaneswalkerSpell
            | CreatureSpell(_)
            | Unavailable) => rule,
            _ => continue,
        };
        for (total, amount) in totals.entry(rule).or_default().iter_mut().zip(lot.amounts) {
            *total += amount;
        }
    }
    // The occupied groups are bounded by fixed rules plus the finite creature
    // type catalogue, regardless of the size or provenance of the floating pool.
    totals.into_iter().collect()
}

pub(super) struct PaymentCapacity {
    // A subset counts every physical source that could produce any of its colors once.
    // Generously grant that source's whole largest output to the subset.
    subset: [u128; 64],
    sources: Vec<Source>,
    restrictions: Vec<(SpellRestriction, [u128; 64])>,
    floating_restrictions: Vec<(LotRestriction, [u128; 6])>,
    material_changing_costs: bool,
}

impl PaymentCapacity {
    pub(super) fn for_player(game: &Game, actor: Player) -> Option<Self> {
        let s = &game.state;
        let linked_returns = !s.exile_links.is_empty();
        if s.graveyard_borrowing.is_some()
            || s.self_granted_abilities.is_some()
            // A spell-cast grant sees a committed spell. It cannot fire inside payment.
            || s.temporary_abilities.as_ref().is_some_and(|rows| rows.iter().any(|grant| {
                !grant.granting_card.triggers().get(usize::from(grant.ability)).is_some_and(
                    |trigger| matches!(trigger.condition.event(), TriggerCondition::SpellCast { .. }),
                )
            }))
            || s.controls.is_some()
            || s.temporary_controls.is_some()
            || s.temporary_copies.is_some()
            || s.manifest.as_ref().is_some_and(|m| !m.permanents.is_empty())
            || s.copy_recipes.as_ref().is_some_and(|rows| rows.iter().any(|row| {
                let object = &s.objects[row.object.card as usize];
                if object.zone != Zone::Battlefield
                    || !object.is_copy_token()
                    || game.object_ref(row.object.card) != row.object
                {
                    return true;
                }
                // Base power/toughness cannot fund a supported mana mode. Require
                // the scanned public definition and every other copiable field to match.
                let mut expected = CopyRecipe::new(object.card);
                expected.exceptions.base = row.recipe.exceptions.base;
                row.recipe != expected
            }))
            || borrowed_activations::may_supply_borrowed_mana(game)
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
        if linked_returns && !fixed_exile_return_sources(game) {
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
                    // This grant turns on one printed slot at max speed. A nonmana
                    // activation needs priority and cannot run during payment.
                    if let F::SourceMaxSpeedActivation(slot) = fragment
                        && object
                            .card
                            .activations()
                            .get(usize::from(slot))
                            .is_some_and(|ability| ability.mana_production().is_none())
                    {
                        continue;
                    }
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
        let mut material_changing_costs = false;
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
            for (index, ability) in object.card.activations().iter().enumerate() {
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
                let changes_materials = ability.cost.sacrifice
                    || ability.cost.exile_self
                    || ability.cost.discard
                    || ability.cost.remove.is_some()
                    || ability.cost.loyalty.is_some()
                    || object.card.activation_put_counter(index as u16).is_some();
                if linked_returns && changes_materials {
                    return None;
                }
                material_changing_costs |= changes_materials;
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

        // The concrete pool already includes tagged mana and retained combat red.
        let pool = s.players[actor.index()].mana.map(u64::from);
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
            floating_restrictions: floating_spell_restrictions(game, actor),
            material_changing_costs,
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
        let mut excluded = [0; 6];
        if !self.floating_restrictions.is_empty() {
            let purpose = Purpose::Spell(mana_lots::Source {
                card: definition,
                mana_value: definition.fixed_mana_value(),
                kinds: definition.kind_mask(),
                subtypes: GrantedSubtypes::default(),
                foreign_stash: false,
                cast_from_hand: false,
                cast_from_exile: origin == Zone::Exile,
                spend_any_mana: false,
                linked_mana_source: None,
                activation_object: None,
                equip: false,
            });
            for &(restriction, amounts) in &self.floating_restrictions {
                // The capacity world excludes live subtype grants across zones.
                // Outside the hand, cast permissions can add creature subtypes.
                if matches!(restriction, LotRestriction::CreatureSpell(_))
                    && definition.is_creature()
                    && origin != Zone::Hand
                {
                    continue;
                }
                if !(Quality {
                    restriction,
                    ..Quality::ORDINARY
                })
                .eligible(purpose)
                {
                    for (total, amount) in excluded.iter_mut().zip(amounts) {
                        *total += amount;
                    }
                }
            }
        }
        let cover = |subset: &[u128; 64]| {
            Self::covers(colored, generic, |mask| {
                if excluded == [0; 6] {
                    return subset[mask];
                }
                let remove = excluded
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| mask & (1 << i) != 0)
                    .map(|(_, amount)| amount)
                    .sum();
                subset[mask].saturating_sub(remove)
            })
        };
        cover(&self.subset)
            && self
                .restrictions
                .iter()
                .all(|(kind, subset)| !kind.excludes(definition, origin) || cover(subset))
    }

    pub(super) fn permits(
        &self,
        colored: Mana,
        generic: u32,
        reserved_tap: Option<CardId>,
    ) -> bool {
        let reserved = reserved_tap.map_or(Source::default(), |id| self.sources[id as usize]);
        Self::covers(colored, generic, |mask| {
            self.subset[mask]
                - if usize::from(reserved.colors) & mask != 0 {
                    u128::from(reserved.maximum)
                } else {
                    0
                }
        })
    }

    fn covers(colored: Mana, generic: u32, upper: impl Fn(usize) -> u128) -> bool {
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

/// Certify declaration routes whose remaining funding actions cannot deploy,
/// return, or take control of material, or pay a cost that moves it. Live local
/// forms and existing crew still need to be counted by the material predicate.
pub(super) fn stable_optional_material_board(game: &Game, a: &Announcement) -> bool {
    let Some(source) = a.item.source else {
        return false;
    };
    let state = &game.state;
    if !a.pay_window
        || a.item.is_copy
        || !a.item.cast.is_some_and(|cast| {
            matches!(
                cast.method,
                CastMethod::Normal
                    | CastMethod::Airbend
                    | CastMethod::Flashback
                    | CastMethod::GrantedFlashback
                    | CastMethod::GrantedFixedFlashback(_)
                    | CastMethod::GrantedZeroFlashback
                    | CastMethod::Harmonize
                    | CastMethod::GrantedHarmonize
                    | CastMethod::Foretell
                    | CastMethod::Mayhem
                    | CastMethod::Plot
                    | CastMethod::Warp
            )
        })
        || a.limited_cast_permission.is_some()
        || a.origin_additional_cost.is_some()
        || a.origin_counter_cost.is_some()
        || a.additional_cost().is_some()
        || source.card.additional_cost().is_some()
        || source.card.kicker_additional_cost().is_some()
        || source.card.behold_cast().is_some()
        || a.cast_costs.as_ref().is_some_and(|costs| {
            costs
                .clauses
                .iter()
                .any(|row| row.clause != CastCostClause::Mana)
        })
        || !state.attachment_sources.is_empty()
        || state.roles.as_ref().is_some_and(|roles| {
            !roles.characteristic_sources.is_empty() || !roles.live_static.is_empty()
        })
    {
        return false;
    }
    PaymentCapacity::for_player(game, a.item.controller)
        .is_some_and(|capacity| !capacity.material_changing_costs)
}

/// An immediate linked return only fires when its exact host leaves. For a
/// tap-only funding window, certify hosts that cannot leave through creature,
/// Aura, Saga, battle, or loyalty state-based actions. The caller separately
/// excludes costs that could move permanents or change their counters.
fn fixed_exile_return_sources(game: &Game) -> bool {
    let state = &game.state;
    let mut animated = vec![false; state.objects.len()];
    if let Some(roles) = &state.roles {
        for row in &roles.animations {
            animated[row.object.card as usize] = true;
        }
    }
    state.exile_links.iter().all(|link| {
        let id = link.source.card;
        let host = &state.objects[id as usize];
        host.zone == Zone::Battlefield
            && game.object_ref(id) == link.source
            && !animated[id as usize]
            && host.attached_to.is_none()
            && host.attached_to_player.is_none()
            && host.card.kind_mask() & (1 | 64 | BATTLE_KIND) == 0
            && host.card.vehicle_stats().is_none()
            && host.card.saga_final_chapter().is_none()
            && !matches!(host.card.effect(), Effect::Aura(_))
            && !host.card.static_ability_groups().iter().any(|group| {
                group
                    .fragments
                    .iter()
                    .any(|row| matches!(row, F::SourceCounterCreature { .. }))
            })
    })
}
